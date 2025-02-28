use ::pob::{Config, Keystone, PathOfBuilding, PathOfBuildingExt, Stat};

use crate::pob::{self, Element};

static AMBER_50: &str = "dark:text-amber-50 text-slate-800";

pub fn core_stats(pob: &impl PathOfBuilding) -> Vec<Element<'_>> {
    let mut elements = Vec::with_capacity(5);

    Element::new("Life")
        .color("text-rose-500")
        .stat_int(pob.stat_parse(Stat::LifeUnreserved))
        .stat_percent_if(
            !pob.has_keystone(Keystone::ChaosInoculation),
            pob.stat(Stat::LifeInc),
        )
        .add_to(&mut elements);

    if pob.stat_at_least(Stat::EnergyShield, 10.0) {
        Element::new("ES")
            .title("Energy Shield")
            .color("text-cyan-200")
            .stat_int(pob.stat_parse(Stat::EnergyShield))
            .stat_percent_if(pob::is_hybrid(pob), pob.stat(Stat::EnergyShieldInc))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::Ward, 100.0) {
        Element::new("Ward")
            .color("text-amber-500")
            .stat_int(pob.stat_parse(Stat::Ward))
            .add_to(&mut elements);
    }

    Element::new("Mana")
        .color("text-blue-400")
        .stat_int(pob.stat_parse(Stat::ManaUnreserved))
        .stat_percent_if(
            pob.has_keystone(Keystone::MindOverMatter),
            pob.stat(Stat::ManaInc),
        )
        .add_to(&mut elements);

    if pob.stat_at_least(Stat::Strength, 500.0) {
        Element::new("Str")
            .color("text-rose-500")
            .stat_int(pob.stat_parse(Stat::Strength))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::Dexterity, 500.0) {
        Element::new("Dex")
            .color("text-lime-400")
            .stat_int(pob.stat_parse(Stat::Dexterity))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::Intelligence, 500.0) {
        Element::new("Int")
            .color("text-blue-400")
            .stat_int(pob.stat_parse(Stat::Intelligence))
            .add_to(&mut elements);
    }

    if let Some(ehp) = pob
        .stat_parse(Stat::TotalEhp)
        .filter(|&ehp: &f32| ehp.is_finite())
    {
        Element::new("eHP")
            .title("Total effective Health Pool")
            .color(AMBER_50)
            .stat_int(Some(ehp))
            .hover(pob::formatted_max_hit(pob))
            .add_to(&mut elements);
    } else {
        Element::new("Pool")
            .title("Total Health Pool includes Life, ES, Ward, Mana")
            .color(AMBER_50)
            .stat_int(Some(pob::hp_pool(pob) as f32))
            .add_to(&mut elements);
    }

    elements
}

pub fn defense(pob: &impl PathOfBuilding) -> Vec<Element<'_>> {
    let mut elements = Vec::with_capacity(5);

    Element::new("Resistances")
        .push_percent(
            "text-orange-500 dark:text-orange-400",
            pob.stat_parse(Stat::FireResistance).unwrap_or(-60.0),
        )
        .push_percent(
            "text-blue-400",
            pob.stat_parse(Stat::ColdResistance).unwrap_or(-60.0),
        )
        .push_percent(
            "text-yellow-600 dark:text-yellow-300",
            pob.stat_parse(Stat::LightningResistance).unwrap_or(-60.0),
        )
        .push_percent(
            "text-fuchsia-500",
            pob.stat_parse(Stat::ChaosResistance).unwrap_or(-60.0),
        )
        .add_to(&mut elements);

    if pob.stat_at_least(Stat::MeleeEvadeChance, 20.0) {
        Element::new("Evade")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::MeleeEvadeChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::PhysicalDamageReduction, 10.0)
        && pob.config(Config::EnemeyHit).is_some()
    {
        Element::new("PDR")
            .title("Physical Damage Reduction")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::PhysicalDamageReduction))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::SpellSuppressionChance, 30.0) {
        Element::new("Supp")
            .title("Spell Suppression")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::SpellSuppressionChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::AttackDodgeChance, 20.0) {
        Element::new("Dodge")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::AttackDodgeChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::SpellDodgeChance, 10.0) {
        Element::new("Spell Dodge")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::SpellDodgeChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::BlockChance, 30.0) {
        Element::new("Block")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::BlockChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::SpellBlockChance, 10.0) {
        Element::new("Spell Block")
            .color(AMBER_50)
            .stat_percent(pob.stat(Stat::SpellBlockChance))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::Armour, 5000.0) {
        Element::new("Armour")
            .color(AMBER_50)
            .stat_int(pob.stat_parse(Stat::Armour))
            .add_to(&mut elements);
    }

    if pob.stat_at_least(Stat::Evasion, 5000.0) {
        Element::new("Evasion")
            .color(AMBER_50)
            .stat_int(pob.stat_parse(Stat::Evasion))
            .add_to(&mut elements);
    }

    elements
}

pub fn offense(pob: &impl PathOfBuilding) -> Vec<Element<'_>> {
    let mut elements = Vec::with_capacity(5);

    // TODO: real minion support
    let is_minion = pob.minion_stat(Stat::CombinedDps).is_some();

    let dps = pob
        .stat_parse(Stat::FullDps)
        .filter(|&dps: &f32| dps.is_finite() && dps > 0.0)
        .or_else(|| match is_minion {
            true => pob.minion_stat_parse(Stat::CombinedDps),
            false => pob.stat_parse(Stat::CombinedDps),
        });

    Element::new("DPS")
        .color(AMBER_50)
        .stat_int(dps)
        .add_to(&mut elements);

    let speed = if is_minion {
        pob.minion_stat_parse(Stat::Speed)
    } else {
        pob.stat_parse(Stat::Speed)
    };

    if speed > Some(0.001) {
        // TODO: this is cast rate for spells
        Element::new("Speed")
            .color(AMBER_50)
            .stat_float(speed)
            .add_to(&mut elements);
    }

    Element::new("Hit Rate")
        .color(AMBER_50)
        .stat_float(pob.stat_parse(Stat::HitRate))
        .add_to(&mut elements);

    Element::new("Hit Chance")
        .color(AMBER_50)
        .stat_percent(pob.stat(Stat::HitChance))
        .add_to(&mut elements);

    if pob::is_crit(pob) {
        Element::new("Crit Chance")
            .color(AMBER_50)
            .stat_percent_float(pob.stat_parse(Stat::CritChance))
            .add_to(&mut elements);

        if pob.stat_at_least(Stat::CritMultiplier, 1.0) {
            Element::new("Crit Multi")
                .color(AMBER_50)
                .stat_percent_int(pob.stat_parse(Stat::CritMultiplier).map(|v: f32| v * 100.0))
                .add_to(&mut elements);
        }
    }

    elements
}

pub fn config(pob: &impl PathOfBuilding) -> Vec<Element<'_>> {
    let mut configs = Vec::with_capacity(5);

    let boss = pob.config(Config::Boss);
    if boss.is_true() {
        configs.push("Boss".to_owned());
    } else if let Some(boss) = boss.string() {
        configs.push(boss.to_owned());
    }

    if pob.config(Config::Focused).is_true() {
        configs.push("Focused".to_owned());
    }

    macro_rules! effect {
        ($enable:expr, $effect:expr, $def:expr, $format:expr) => {
            if pob.config($enable).is_true() {
                let effect = pob.config($effect).number().unwrap_or($def);
                configs.push(format!($format, effect));
            }
        };
    }

    effect!(Config::EnemyShocked, Config::ShockEffect, 15.0, "{}% Shock");
    effect!(
        Config::EnemyScorched,
        Config::ScorchedEffect,
        10.0,
        "{}% Scorch"
    );
    effect!(
        Config::EnemyBrittled,
        Config::BrittleEffect,
        2.0,
        "{}% Brittle"
    );
    effect!(Config::EnemySapped, Config::SapEffect, 6.0, "{}% Sap");

    if pob.config(Config::CoveredInAsh).is_true() {
        configs.push("Covered in Ash".into());
    }

    if pob.config(Config::FrenzyCharges).is_true() {
        if let Some(amount) = pob.config(Config::FrenzyChargesAmount).number() {
            configs.push(format!("{}x Frenzy", amount as i32));
        } else {
            configs.push("Frenzy".into());
        }
    }

    if pob.config(Config::PowerCharges).is_true() {
        if let Some(amount) = pob.config(Config::PowerChargesAmount).number() {
            configs.push(format!("{}x Power", amount as i32));
        } else {
            configs.push("Power".into());
        }
    }

    if let Some(amount) = pob.config(Config::WitherStacks).number() {
        if amount > 0.0 {
            configs.push(format!("{}x Wither", amount as i32));
        }
    }

    let custom_mods = pob
        .config(Config::CustomMods)
        .string()
        .filter(|s| !s.trim().is_empty());
    if custom_mods.is_some() {
        configs.push("Custom Mods".to_owned());
    }

    if configs.is_empty() {
        configs.push("None".to_owned());
    }

    let element = Element::new("Config")
        .color(AMBER_50)
        .stat_str(Some(configs.join(", ")))
        .hover(custom_mods);

    vec![element]
}

pub fn choices(pob: &impl PathOfBuilding) -> Vec<Element<'_>> {
    let mut elements = Vec::with_capacity(2);

    let bandit = pob
        .bandit()
        .map(|bandit| bandit.name())
        .unwrap_or("Kill All");

    Element::new("Bandit")
        .color(AMBER_50)
        .stat_str(Some(bandit))
        .add_to(&mut elements);

    let pantheons = [
        pob.pantheon_major_god().map(|god| god.name()),
        pob.pantheon_minor_god().map(|god| god.name()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    if !pantheons.is_empty() {
        Element::new("Pantheon")
            .color(AMBER_50)
            .stat_str(Some(pantheons.join(", ")))
            .add_to(&mut elements);
    }

    elements
}
