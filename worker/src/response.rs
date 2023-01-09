use std::time::Duration;

use serde::{Deserialize, Serialize};
use shared::{model::PasteId, User};

pub use self::ResponseError::{ApiError, AppError};
use crate::{
    storage::StoredPaste,
    utils::{CacheControl, Etag},
};

pub type Result = std::result::Result<Response, ResponseError>;

#[derive(Debug)]
pub enum ResponseError {
    ApiError(crate::Error),
    AppError(crate::Error),
}

impl ResponseError {
    pub fn inner(&self) -> &crate::Error {
        match self {
            Self::ApiError(err) => err,
            Self::AppError(err) => err,
        }
    }
}

// maybe this should be an enum but this might be annoying
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Meta {
    /// The user the resource is scoped to.
    ///
    /// This is e.g. filled when listing pastes from a specific
    /// user and also when viewing a user scoped paste.
    pub user_id: Option<User>,
    /// The stringified form of a `PasteId`.
    ///
    /// For example `abc` or `user:abc`.
    /// This uniquely identifies a paste.
    pub paste_id: Option<String>,
    pub ascendancy_or_class: Option<String>,
    pub main_skill_name: Option<String>,
    pub version: Option<String>,
    pub last_modified: Option<u64>,
}

impl Meta {
    pub fn paste(id: impl Into<PasteId>, pmeta: impl PartialMeta) -> Self {
        let (user_id, paste_id) = match id.into() {
            PasteId::Paste(paste_id) => (None, Some(paste_id)),
            PasteId::UserPaste(up) => {
                let paste_id = up.to_string();
                (Some(up.user), Some(paste_id))
            }
        };

        let mut meta = Meta {
            user_id,
            paste_id,
            ..Default::default()
        };
        pmeta.merge_with(&mut meta);
        meta
    }

    pub fn list(id: impl Into<User>) -> Self {
        Meta {
            user_id: Some(id.into()),
            ..Default::default()
        }
    }
}

pub trait PartialMeta {
    fn merge_with(self, meta: &mut Meta);
}

impl PartialMeta for &StoredPaste {
    fn merge_with(self, meta: &mut Meta) {
        let Some(ref this) = self.metadata else { return; };
        meta.ascendancy_or_class = Some(this.ascendancy_or_class.clone());
        meta.main_skill_name = this.main_skill_name.clone();
        meta.version = this.version.clone();
        meta.last_modified = Some(self.last_modified);
    }
}

impl PartialMeta for shared::model::Paste {
    fn merge_with(self, meta: &mut Meta) {
        let Some(this) = self.metadata else { return; };
        meta.ascendancy_or_class = Some(this.ascendancy_or_class);
        meta.main_skill_name = this.main_skill_name;
        meta.version = this.version;
        meta.last_modified = Some(self.last_modified);
    }
}

impl PartialMeta for &shared::model::Paste {
    fn merge_with(self, meta: &mut Meta) {
        let Some(ref this) = self.metadata else { return; };
        meta.ascendancy_or_class = Some(this.ascendancy_or_class.clone());
        meta.main_skill_name = this.main_skill_name.clone();
        meta.version = this.version.clone();
        meta.last_modified = Some(self.last_modified);
    }
}

impl PartialMeta for shared::model::PasteMetadata {
    fn merge_with(self, meta: &mut Meta) {
        meta.ascendancy_or_class = Some(self.ascendancy_or_class);
        meta.main_skill_name = self.main_skill_name;
        meta.version = self.version;
        meta.last_modified = Some(js_sys::Date::new_0().get_time() as u64);
    }
}

pub struct Response {
    status_code: u16,
    // maybe this should be `http::HeaderMap`.
    headers: worker::Headers,
    body: worker::ResponseBody,
    meta: Option<Meta>,
}

impl Response {
    pub fn ok() -> Self {
        Self::status(200)
    }

    pub fn not_found() -> Self {
        Self::status(404)
    }

    pub fn status(status_code: u16) -> Self {
        Self {
            status_code,
            headers: worker::Headers::new(),
            body: worker::ResponseBody::Empty,
            meta: None,
        }
    }

    pub fn redirect_temp(location: &str) -> Self {
        Self::status(307).header("Location", location)
    }

    pub fn redirect_perm(location: &str) -> Self {
        Self::status(301).header("Location", location)
    }
}

impl Response {
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = worker::ResponseBody::Body(body.into());
        self
    }

    pub fn json(self, body: &impl Serialize) -> Self {
        self.content_type("application/json")
            .body(serde_json::to_vec(body).unwrap()) // TODO: unwrap
    }

    pub fn html(self, body: impl Into<String>) -> Self {
        self.content_type("text/html")
            .body(body.into().into_bytes())
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        if !value.is_empty() {
            let r = self.headers.set(name, value);
            debug_assert!(r.is_ok());
        }
        self
    }

    pub fn append_header(mut self, name: &str, value: &str) -> Self {
        if !value.is_empty() {
            let r = self.headers.append(name, value);
            debug_assert!(r.is_ok());
        }
        self
    }

    pub fn content_type(self, content_type: &str) -> Self {
        self.header("Content-Type", content_type)
    }

    pub fn etag<'a>(self, etag: impl Into<Option<Etag<'a>>>) -> Self {
        let Some(etag) = etag.into() else { return self };
        self.header("Etag", &etag.to_string())
    }

    pub fn state_cookie(self, state: &str) -> Self {
        self.append_header(
            "Set-Cookie",
            &format!("state={state}; Max-Age=600; Secure; Same-Site=Lax; Path=/"),
        )
    }

    pub fn delete_state_cookie(self) -> Self {
        self.append_header(
            "Set-Cookie",
            "state=none; Max-Age=0; Secure; Same-Site=Lax; Path=/",
        )
    }

    pub fn new_session(self, session: &str) -> Self {
        self.append_header(
            "Set-Cookie",
            &format!("session={session}; Max-Age=1209600; Secure; SameSite=Lax; Path=/"),
        )
    }

    pub fn cache(self, cache_control: CacheControl) -> Self {
        self.header("Cache-Control", &cache_control.to_string())
    }

    pub fn cache_for(self, ttl: Duration) -> Self {
        self.cache(CacheControl::default().public().max_age(ttl))
    }

    pub fn result<T>(self) -> std::result::Result<Self, T> {
        Ok(self)
    }
}

// Methods related to metadata.
impl Response {
    pub fn meta(mut self, meta: impl Into<Option<Meta>>) -> Self {
        self.meta = meta.into();
        self
    }

    pub fn meta_paste(mut self, id: impl Into<PasteId>, pmeta: impl PartialMeta) -> Self {
        self.meta = Some(Meta::paste(id, pmeta));
        self
    }

    pub fn meta_list(mut self, id: impl Into<User>) -> Self {
        self.meta = Some(Meta::list(id));
        self
    }
}

// Accessors
impl Response {
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn get_meta(&self) -> Option<&Meta> {
        self.meta.as_ref()
    }

    pub fn get_header(&self, name: &str) -> Option<String> {
        self.headers.get(name).ok().flatten()
    }
}

// Utility Methods that do not modify the response.
impl Response {
    /// Whether the response's status code is 2xx.
    pub fn is_2xx(&self) -> bool {
        self.status_code >= 200 && self.status_code < 300
    }

    /// Whether the response currently can be cached.
    ///
    /// A response without caching headers is not cacheable,
    /// becaue it wouldn't be cached for any duration.
    ///
    /// Also returns false when the response has status code 206
    /// or contains the `Vary: *` header.
    ///
    /// See also: https://developers.cloudflare.com/workers/runtime-apis/cache/#parameters
    pub fn is_cacheable(&self) -> bool {
        if self.status_code == 206 {
            return false;
        }
        if self.headers.get("Vary").unwrap().as_deref() == Some("*") {
            return false;
        }
        ["Cache-Control", "ETag", "Expires", "Last-Modified"]
            .into_iter()
            .any(|hn| self.headers.has(hn).unwrap())
    }

    /// Whether the response was created from the cache.
    pub fn was_cached(&self) -> bool {
        self.get_header("Cf-Cache-Status").as_deref() == Some("HIT")
    }
}

// Cache related methods.
impl Response {
    pub fn from_cache(r: worker::Response) -> Self {
        let mut response: Self = r.into();

        let meta = response.headers.get("X-Response-Meta").ok().flatten();
        response.meta = meta.and_then(|m| serde_json::from_str(&m).ok());

        let _ = response.headers.delete("X-Response-Meta");
        let _ = response.headers.set("Cf-Cache-Status", "HIT");

        response
    }

    pub fn for_cache(&self) -> worker::Response {
        let body = clone_body(&self.body);
        let mut response = worker::Response::from_body(body)
            .unwrap()
            .with_status(self.status_code())
            .with_headers(self.headers.clone());

        if let Some(ref meta) = self.meta {
            let meta = serde_json::to_string(meta).unwrap();
            response
                .headers_mut()
                .set("X-Response-Meta", &meta)
                .unwrap();
        }

        response
    }
}

impl Clone for Response {
    fn clone(&self) -> Self {
        Self {
            status_code: self.status_code,
            // Cloning headers maybe sucks here, but is also good
            // because headers can actually be read only, so cloning get's rid of that limitation
            headers: self.headers.clone(),
            body: clone_body(&self.body),
            meta: self.meta.clone(),
        }
    }
}

impl From<Response> for worker::Response {
    fn from(r: Response) -> worker::Response {
        worker::Response::from_body(r.body)
            .unwrap()
            .with_status(r.status_code)
            .with_headers(r.headers)
    }
}

impl From<worker::Response> for Response {
    fn from(wr: worker::Response) -> Response {
        let headers = wr.headers().clone();

        let mut r = Response::status(wr.status_code());
        r.headers = headers;
        r.body = worker::ResponseBody::Stream(worker::worker_sys::Response::from(wr));

        r
    }
}

fn clone_body(rb: &worker::ResponseBody) -> worker::ResponseBody {
    match &rb {
        worker::ResponseBody::Empty => worker::ResponseBody::Empty,
        worker::ResponseBody::Body(v) => worker::ResponseBody::Body(v.clone()),
        worker::ResponseBody::Stream(s) => {
            worker::ResponseBody::Stream(s.clone().expect("response body already used?"))
        }
    }
}
