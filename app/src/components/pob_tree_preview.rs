use itertools::Itertools;
use js_sys::{Object, Uint32Array};
use pob::TreeSpec;
use shared::model::data;
use sycamore::prelude::*;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, HtmlElement, HtmlObjectElement};

use crate::{
    build::Build,
    components::{PobColoredSelect, Popup, TreeNode},
    consts,
    utils::{from_ref, hooks::scoped_event_passive, reflect_set, IteratorExt},
};

#[derive(Debug)]
struct Tree<'build> {
    name: String,
    tree_url: String,
    svg_url: &'static str,
    spec: TreeSpec<'build>,
    nodes: &'build data::Nodes,
    overrides: Vec<Override<'build>>,
}

#[derive(Debug)]
struct Override<'build> {
    count: usize,
    name: &'build str,
    effect: &'build str,
}

#[component]
pub fn PobTreePreview<'a, G: Html>(cx: Scope<'a>, build: &'a Build) -> View<G> {
    let trees = build
        .trees()
        .filter_map(|(nodes, spec)| {
            let tree_url = get_tree_url(&spec)?;
            let svg_url = get_svg_url(&spec);
            let overrides = extract_overrides(&spec.overrides);
            Some(Tree {
                name: spec.title.unwrap_or("<Default>").to_owned(),
                tree_url,
                svg_url,
                spec,
                nodes,
                overrides,
            })
        })
        .collect::<Vec<_>>();

    if trees.is_empty() {
        return view! { cx, };
    }

    let trees = create_ref(cx, trees);
    let current_tree = create_signal(cx, trees.iter().find_or_first(|t| t.spec.active).unwrap());
    let tree_loaded = create_signal(cx, false);
    let node_ref = create_node_ref(cx);

    let current_svg = create_signal(cx, current_tree.get().svg_url);
    create_effect(cx, || {
        let new_svg = current_tree.get().svg_url;
        // Debounce the svg and reset loading state when it changed.
        if new_svg != *current_svg.get() {
            tree_loaded.set(false);
            current_svg.set(new_svg);
        }
    });

    let events = std::sync::Once::new();
    let attach = create_signal(cx, None);
    let popup = create_signal(cx, view! { cx, });
    let on_mouseover = move |event: web_sys::Event| {
        let target: HtmlElement = event.target().unwrap().unchecked_into();

        let dataset = target.dataset();
        match dataset.get("name") {
            Some(name) => {
                let stats = dataset
                    .get("stats")
                    .map(|s| s.split(";;").map(Into::into).collect())
                    .unwrap_or_default();
                let kind = dataset.get("kind");

                popup.set(view! { cx, TreeNode(name=name, stats=stats, kind=kind) });
                attach.set(Some(target.unchecked_into()));
            }
            None => {
                attach.set(None);
            }
        }
    };

    create_effect(cx, move || {
        let tree = current_tree.get();
        if !*tree_loaded.get() {
            return;
        }

        let obj = Object::new();
        reflect_set(&obj, "nodes", Uint32Array::from(tree.spec.nodes));
        reflect_set(&obj, "classId", tree.spec.class_id);
        reflect_set(&obj, "ascendancyId", tree.spec.ascendancy_id);
        reflect_set(
            &obj,
            "alternateAscendancyId",
            tree.spec.alternate_ascendancy_id,
        );

        from_ref::<web_sys::HtmlObjectElement>(node_ref)
            .content_window()
            .unwrap()
            .unchecked_into::<TreeObj>()
            .load(obj.into());

        let inner = from_ref::<HtmlObjectElement>(node_ref)
            .content_document()
            .unwrap()
            .unchecked_into();

        events.call_once(|| {
            scoped_event_passive(cx, inner, "mouseover", on_mouseover);
        });
    });

    // TODO: this updates the currently active tree, but it doesn't read from it
    // the select would need to be updated as well if the tree changes, kinda tricky...
    let select = render_select(cx, trees, move |index, tree| {
        current_tree.set(tree);
        build.active_tree().set(index);
    });

    let nodes = create_memo(cx, move || render_nodes(cx, &current_tree.get()));
    let tree_level = create_memo(cx, move || {
        let current_tree = current_tree.get();
        let (nodes, level) = resolve_level(current_tree.spec.nodes.len());
        let desc = format!("Level {level} ({nodes} passives)");
        view! { cx,
            a(href=current_tree.tree_url, rel="external", target="_blank",
            class="text-sky-500 dark:text-sky-400 hover:underline") {
                (desc)
            }
        }
    });

    view! { cx,
        Popup(attach=attach, parent=Some(node_ref)) { (&*popup.get()) }
        div(class="flex flex-wrap align-center") {
            div(class="h-9 max-w-full") { (select) }
            div(class="flex-1 text-right sm:mr-3 whitespace-nowrap") { (&*tree_level.get()) }
        }
        div(class="grid grid-cols-10 gap-3") {
            div(class="col-span-10 lg:col-span-7 h-[450px] md:h-[800px] cursor-move md:overflow-auto mt-2") {
                object(
                    ref=node_ref,
                    data=current_svg.get(),
                    class="h-full w-full bg-center bg-no-repeat touch-pan
                    transition-[background-image] duration-1000 will-change-[background-image]",
                    type="image/svg+xml",
                    on:load=|_: Event| { tree_loaded.set(true); },
                    on:mouseout=|_: Event| { attach.set(None) }
                ) {}
            }

            div(class="col-span-10 lg:col-span-3 flex flex-col gap-3 h-full relative") {
                div(class="flex flex-col gap-3 md:gap-6 h-full w-full lg:absolute overflow-y-auto") {
                    (*nodes.get())
                }
            }
        }
    }
}

fn resolve_level(allocated: usize) -> (usize, usize) {
    // TODO: needs auto-generated node information for ascendancies
    if allocated == 0 {
        return (0, 0);
    }

    // character start node
    let allocated = allocated - 1;

    // points count towards allocated but aren't available skill tree points
    let asc = match allocated {
        0..=38 => 0,
        39..=69 => 3, // 2 points + ascendancy start node
        70..=90 => 5,
        91..=98 => 7,
        _ => 9,
    };

    // TODO: check for bandits
    let bandits = match allocated {
        0..=21 => 0,
        _ => 2,
    };

    let quests = match allocated - asc - bandits {
        0..=11 => 0,
        12..=23 => 2,
        24..=34 => 3,
        35..=44 => 5,
        45..=49 => 6,
        50..=57 => 8,
        58..=64 => 11,
        65..=73 => 14,
        74..=80 => 17,
        81..=85 => 19,
        _ => 22,
    };

    (allocated - asc, 1 + allocated - asc - bandits - quests)
}

fn extract_overrides<'a>(overrides: &[pob::Override<'a>]) -> Vec<Override<'a>> {
    overrides
        .iter()
        .sorted_unstable_by_key(|k| (k.name, k.effect))
        .dedup_by_with_count(|a, b| (a.name, a.effect) == (b.name, b.effect))
        .map(|(count, o)| Override {
            count,
            name: o.name,
            effect: o.effect,
        })
        .collect()
}

fn render_nodes<G: GenericNode + Html>(cx: Scope, tree: &Tree<'_>) -> View<G> {
    let nodes = tree.nodes;

    if nodes.is_empty() {
        return view! { cx,
            div(class="text-stone-200 hidden lg:block text-center") {
                "No Keystones and Masteries"
            }
        };
    }

    let overrides = tree
        .overrides
        .iter()
        .map(|o| render_override(cx, o))
        .collect_view();

    let keystones = nodes
        .keystones
        .iter()
        .map(|node| render_keystone(cx, node))
        .collect_view();

    let masteries = nodes
        .masteries
        .iter()
        .map(|node| render_mastery(cx, node))
        .collect_view();

    view! { cx,
        div(class="grid grid-cols-fit-mastery gap-2 lg:gap-1 empty:hidden") { (overrides) }
        div(class="grid grid-cols-fit-keystone gap-2 lg:gap-1 empty:hidden") { (keystones) }
        div(class="grid grid-cols-fit-mastery gap-2 lg:gap-1 empty:hidden") { (masteries) }
    }
}

fn render_override<G: GenericNode + Html>(cx: Scope, r#override: &Override) -> View<G> {
    let name = r#override.name.to_owned();
    let effect = r#override.effect.to_owned();
    let count = if r#override.count > 1 {
        format!("(x{})", r#override.count)
    } else {
        String::new()
    };

    view! { cx,
        div(class="bg-slate-900 rounded-xl px-4 py-3") {
            div(class="mb-2 text-stone-200 text-sm md:text-base") {
                span() { (name) }
                span(class="text-xs ml-1") { (count) }
            }
            div(class="flex flex-col gap-2 pb-1 whitespace-pre-line text-xs md:text-sm text-slate-400") { (effect) }
        }
    }
}

fn render_keystone<G: GenericNode + Html>(cx: Scope, node: &data::Node) -> View<G> {
    let name = node.name.to_owned();
    let alt = name.clone();
    let stats = node.stats.iter().join("\n");

    let src = node
        .icon
        .as_deref()
        .map(crate::assets::item_image_url)
        .unwrap_or_default();

    view! { cx,
        div(class="bg-slate-900 rounded-xl px-4 py-3", title=stats) {
            div(class="text-stone-200 text-sm md:text-base flex items-center gap-2") {
                img(class="rounded-xl w-7 h-7", src=src, alt=alt, onerror=consts::IMG_ONERROR_HIDDEN, loading="lazy") {}
                span() { (name) }
            }
        }
    }
}

fn render_mastery<G: GenericNode + Html>(cx: Scope, node: &data::Node) -> View<G> {
    let name = node.name.to_owned();
    let alt = name.clone();
    let stats = node
        .stats
        .iter()
        .map(|stat| {
            let stat = stat.clone();
            view! { cx, li(class="leading-tight") { (stat) } }
        })
        .collect_view();

    let src = node
        .icon
        .as_deref()
        .map(crate::assets::item_image_url)
        .unwrap_or_default();

    view! { cx,
        div(class="bg-slate-900 rounded-xl px-4 py-3") {
            div(class="mb-2 text-stone-200 text-sm md:text-base flex items-center gap-2") {
                img(class="rounded-xl w-7 h-7", src=src, alt=alt, onerror=consts::IMG_ONERROR_HIDDEN, loading="lazy") {}
                span() { (name) }
            }
            ul(class="flex flex-col gap-2 pb-1 whitespace-pre-line text-xs md:text-sm text-slate-400") { (stats) }
        }
    }
}

fn render_select<'a, G: GenericNode + Html, F>(
    cx: Scope<'a>,
    trees: &'a [Tree],
    on_change: F,
) -> View<G>
where
    F: Fn(usize, &'a Tree) + 'a,
{
    if trees.len() <= 1 {
        return view! { cx, };
    }

    let options = trees.iter().map(|t| t.name.clone()).collect();
    let selected = trees.iter().position(|t| t.spec.active);
    let on_change = move |index| {
        if let Some(index) = index {
            on_change(index, &trees[index])
        }
    };

    view! { cx,
        PobColoredSelect(options=options, selected=selected, label="Select tree", on_change=on_change)
    }
}

fn get_tree_url(spec: &TreeSpec) -> Option<String> {
    spec.url
        .filter(|url| {
            url.starts_with("https://pathofexile.com")
                || url.starts_with("https://www.pathofexile.com")
        })
        .map(|url| url.to_owned())
}

fn get_svg_url(spec: &TreeSpec) -> &'static str {
    match spec.version {
        Some("3_15") => "/assets/3.15.svg",
        Some("3_16") => "/assets/3.16.svg",
        Some("3_17") => "/assets/3.17.svg",
        Some("3_18") => "/assets/3.18.svg",
        Some("3_19") => "/assets/3.19.svg",
        Some("3_20") => "/assets/3.20.svg",
        Some("3_21") => "/assets/3.21.svg",
        Some("3_22") => "/assets/3.22.svg",
        _ => "/assets/3.23.svg",
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    type TreeObj;

    #[wasm_bindgen(method, js_name=tree_load)]
    fn load(this: &TreeObj, data: JsValue);
}
