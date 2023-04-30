use shared::PasteId;

use crate::{
    request_context::RequestContext,
    route::{Api, Route},
    Response,
};

struct CacheEntryInner {
    cache: Cache,
    key: web_sys::Request,
    ctx: worker::Context,
}

pub struct CacheEntry {
    inner: Option<CacheEntryInner>,
}

impl CacheEntry {
    pub async fn load(&self) -> Option<Response> {
        let inner = self.inner.as_ref()?;

        let cache = inner.cache.open().await;

        cache
            .get(&inner.key, true)
            .await
            .expect("cache api")
            .map(Response::from_cache)
    }

    pub async fn store(self, mut response: Response) -> Response {
        if self.inner.is_none() || !response.is_cacheable() {
            return response;
        }

        let CacheEntryInner { cache, key, ctx } = self.inner.unwrap(); // checked for none above
        let for_cache = response.for_cache();

        ctx.wait_until(async move {
            tracing::debug!("--> caching response in {cache}");
            let cache = cache.open().await;
            let r = cache.put(&key, for_cache).await;
            debug_assert!(r.is_ok(), "failed to cache response: {r:?}");
            tracing::debug!("<-- response cached in {cache:?}");
        });

        response
            .header("X-Pobbin-Cache", &cache.to_string())
            .header("Cf-Cache-Status", "MISS")
    }
}

impl From<&RequestContext> for CacheEntry {
    fn from(value: &RequestContext) -> Self {
        if value.req().method() != worker::Method::Get {
            return Self { inner: None };
        }

        let cache = Cache::select(value);
        // This can only fail if the body was already consumed,
        // but we're cloning a get request here.
        // This may also fail for URLs with credentials (-> MDN) on Firefox only,
        // we're not running on Firefox, also we should never have credentials in URLs at
        // this point.
        let key = value.req().inner().clone().expect("clone request");
        let ctx = value.ctx().clone();

        Self {
            inner: Some(CacheEntryInner { cache, key, ctx }),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Cache {
    /// Default cache, used when no other cache is selected
    Default,
    /// The owned cache is used for all responses which are specific
    /// to the owner of the resource
    Owned,
}

impl Cache {
    pub fn select(rctx: &RequestContext) -> Self {
        let session = rctx.session();

        match rctx.route() {
            Route::App(app::Route::User(user))
            | Route::Api(Api::Get(crate::route::GetEndpoints::User(user))) => {
                if Some(user) == session.map(|s| &s.name) {
                    Cache::Owned
                } else {
                    Cache::Default
                }
            }
            _ => Cache::Default,
        }
    }

    pub async fn open(&self) -> worker::Cache {
        match self {
            Self::Default => worker::Cache::default(),
            Self::Owned => worker::Cache::open("owned").await,
        }
    }
}

impl std::fmt::Display for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub(crate) fn on_paste_change(rctx: &RequestContext, id: PasteId) {
    let url = rctx.url().unwrap();
    rctx.ctx().wait_until(on_paste_change_async(url, id));
}

pub(crate) async fn on_paste_change_async(mut url: url::Url, id: PasteId) {
    url.set_path("");
    url.set_query(None);
    url.set_fragment(None);
    let prefix = url.to_string();

    let cache_default = Cache::Default.open().await;
    let cache_owned = Cache::Owned.open().await;

    macro_rules! clear {
        ($e:expr) => {{
            let r = format!("{prefix}{}", $e.trim_start_matches('/'));
            let _ = cache_default.delete(&r, true).await;
            let _ = cache_owned.delete(&r, true).await;
        }};
    }

    tracing::info!("resetting cached URLs for {id}");
    clear!(id.to_url());
    clear!(id.to_raw_url());
    clear!(id.to_json_url());
    clear!(id.to_pob_load_url());

    if let PasteId::UserPaste(up) = id {
        clear!(up.to_pob_long_load_url());
        clear!(up.to_paste_edit_url());
        clear!(up.to_user_url());
        clear!(up.to_user_api_url());
    }
    tracing::info!("done resetting caches");
}
