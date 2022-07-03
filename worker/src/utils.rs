use worker::wasm_bindgen::JsCast;
use worker::worker_sys::WorkerGlobalScope;
use worker::{js_sys, worker_sys, Env, Request, Response, Result};

macro_rules! if_debug {
    ($debug:expr, $otherwise:expr) => {{
        #[cfg(feature = "debug")] { $debug }
        #[cfg(not(feature = "debug"))] { $otherwise }
    }};
    ({ $($debug:tt)* }, { $($otherwise:expr)* }) => {{
        #[cfg(feature = "debug")] { $(debug)* }
        #[cfg(not(feature = "debug"))] { $(otherwise)* }
    }};
    { $debug:expr } => {{
        #[cfg(feature = "debug")] { $debug }
    }};
}
pub(crate) use if_debug;

pub fn b64_encode<T: AsRef<[u8]>>(input: T) -> String {
    base64::encode_config(input, base64::URL_SAFE_NO_PAD)
}

pub fn b64_decode<T: AsRef<[u8]>>(input: T) -> crate::Result<Vec<u8>> {
    Ok(base64::decode_config(input, base64::URL_SAFE_NO_PAD)?)
}

pub fn hex(data: &[u8]) -> String {
    data.iter().map(|x| format!("{:02X}", x)).collect()
}

pub fn btoa(s: &str) -> Result<String> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    Ok(worker.btoa(s)?)
}

pub fn basic_auth(username: &str, password: &str) -> Result<String> {
    let mut s = username.to_owned();
    s.push(':');
    s.push_str(password);

    let mut result = "Basic ".to_owned();
    result.push_str(&btoa(&s)?);
    Ok(result)
}

pub fn random_string<const N: usize>() -> Result<String> {
    let random = crate::crypto::get_random_values::<N>()?;
    Ok(b64_encode(random))
}

pub fn hash_to_short_id(hash: &[u8], bytes: usize) -> Result<String> {
    hash.get(0..bytes)
        .map(b64_encode)
        .ok_or_else(|| "Hash too small for id".into())
}

pub fn is_valid_id(id: &str) -> bool {
    id.len() >= 5
        && id.len() < 90
        && id
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-'))
}

pub fn to_path(id: &str) -> Result<String> {
    if !is_valid_id(id) {
        return Err("invalid id".into());
    }
    let mut result = String::with_capacity(4 + id.len());
    result.push_str(unsafe { id.get_unchecked(0..1) });
    result.push('/');
    result.push_str(unsafe { id.get_unchecked(1..2) });
    result.push('/');
    result.push_str(unsafe { id.get_unchecked(2..) });
    Ok(result)
}

pub trait ResponseExt: Sized {
    fn redirect2(target: &str) -> crate::Result<Self>;

    fn cache_for(self, ttl: u32) -> crate::Result<Self> {
        self.with_header("Cache-Control", &format!("max-age={}", ttl))
    }
    fn with_content_type(self, content_type: &str) -> crate::Result<Self> {
        self.with_header("Content-Type", content_type)
    }
    fn with_etag(self, entity_id: &str) -> crate::Result<Self> {
        let entity_id = format!("\"{}\"", entity_id.trim_matches('"'));
        self.with_header("Etag", &entity_id)
    }
    fn with_state_cookie(self, state: &str) -> crate::Result<Self> {
        self.append_header(
            "Set-Cookie",
            &format!("state={state}; Max-Age=600; Secure; Same-Site=Lax; Path=/"),
        )
    }
    fn with_delete_state_cookie(self) -> crate::Result<Self> {
        self.append_header(
            "Set-Cookie",
            "state=none; Max-Age=0; Secure; Same-Site=Lax; Path=/",
        )
    }
    fn with_new_session(self, session: &str) -> crate::Result<Self> {
        self.append_header(
            "Set-Cookie",
            &format!("session={session}; Max-Age=1209600; Secure; SameSite=Lax; Path=/"),
        )
    }

    fn dup_headers(self) -> Self;
    fn append_header(self, name: &str, value: &str) -> crate::Result<Self>;
    fn with_header(self, name: &str, value: &str) -> crate::Result<Self>;

    fn cloned(self) -> crate::Result<(Self, Self)>;
}

impl ResponseExt for Response {
    fn redirect2(target: &str) -> crate::Result<Self> {
        Self::empty()?
            .with_status(307)
            .with_header("Location", target)
    }

    fn dup_headers(self) -> Self {
        let headers = self.headers().clone();
        self.with_headers(headers)
    }

    fn append_header(mut self, name: &str, value: &str) -> crate::Result<Self> {
        self.headers_mut().append(name, value)?;
        Ok(self)
    }

    fn with_header(mut self, name: &str, value: &str) -> crate::Result<Self> {
        self.headers_mut().set(name, value)?;
        Ok(self)
    }

    fn cloned(self) -> crate::Result<(Self, Self)> {
        let status_code = self.status_code();
        let headers = self.headers().clone();

        let response1: worker_sys::Response = self.into();
        let response2 = response1.clone()?;

        let body1 = worker::ResponseBody::Stream(response1);
        let body2 = worker::ResponseBody::Stream(response2);

        let response1 = worker::Response::from_body(body1)?
            .with_status(status_code)
            .with_headers(headers.clone());
        let response2 = worker::Response::from_body(body2)?
            .with_status(status_code)
            .with_headers(headers);

        Ok((response1, response2))
    }
}

pub trait RequestExt: Sized {
    fn cookie(&self, name: &str) -> Option<String>;
    fn session(&self) -> Option<String> {
        self.cookie("session")
    }
}

impl RequestExt for Request {
    fn cookie(&self, name: &str) -> Option<String> {
        let cookie = self.headers().get("Cookie").unwrap()?;

        cookie
            .split(';')
            .filter_map(|part| part.split_once('='))
            .find(|(k, _)| name == k.trim())
            .map(|(_, v)| v.trim().to_owned())
    }
}

pub trait EnvExt: Sized {
    fn storage(&self) -> crate::Result<crate::storage::DefaultStorage>;
    fn oauth(&self) -> Result<crate::poe_api::Oauth>;
    fn dangerous(&self) -> Result<crate::dangerous::Dangerous>;
}

impl EnvExt for Env {
    fn storage(&self) -> crate::Result<crate::storage::DefaultStorage> {
        crate::storage::DefaultStorage::from_env(self)
    }

    fn oauth(&self) -> Result<crate::poe_api::Oauth> {
        Ok(crate::poe_api::Oauth::new(
            self.var(crate::consts::ENV_OAUTH_CLIENT_ID)?.to_string(),
            self.var(crate::consts::ENV_OAUTH_CLIENT_SECRET)?
                .to_string(),
        ))
    }

    fn dangerous(&self) -> Result<crate::dangerous::Dangerous> {
        let secret = self.var(crate::consts::ENV_SECRET_KEY)?.to_string();
        Ok(crate::dangerous::Dangerous::new(secret.into_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_id() {
        assert!(!is_valid_id(""));
        assert!(!is_valid_id("a"));
        assert!(!is_valid_id("abcd"));
        assert!(!is_valid_id(
            "abcdefghijklmnopqrstuvwxyz123456789012345678901234567890\
            abcdefghijklmnopqrstuvwxyz123456789012345678901234567890"
        ));
        assert!(is_valid_id("abcde"));
        assert!(is_valid_id("AZ09az-_"));
        assert!(is_valid_id("-AZ09az-_"));
    }
}
