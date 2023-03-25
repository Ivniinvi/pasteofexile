use std::fmt;
use std::time::Duration;

use git_version::git_version;
use shared::validation;
use worker::{Request, Result};

use crate::Error;

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

pub fn hex_lower(data: &[u8]) -> String {
    data.iter().map(|x| format!("{x:02x}")).collect()
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

pub fn to_path(id: &str) -> crate::Result<String> {
    validation::is_valid_id(id).ok().map_err(|msg| {
        tracing::warn!(id, msg, "invalid id, cannot convert to path");
        Error::InvalidId(msg)
    })?;

    // Invariants for the following unsafe code, should already be checked by the validation
    assert!(id.len() >= 3, "Id too short");
    assert!(id.is_ascii(), "Id not ascii");

    let mut result = String::with_capacity(2 + id.len());
    result.push_str(unsafe { id.get_unchecked(0..1) });
    result.push('/');
    result.push_str(unsafe { id.get_unchecked(1..2) });
    result.push('/');
    result.push_str(unsafe { id.get_unchecked(2..) });
    Ok(result)
}

pub fn to_link(p: &[app::Prefetch], rel: &str) -> String {
    p.iter()
        .map(|p| format!("<{}>;rel={};as={}", p.url(), rel, p.typ()))
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Copy, Clone, Debug)]
pub enum Cachability {
    Public,
    #[allow(dead_code)]
    Private,
    #[allow(dead_code)]
    NoCache,
}

impl fmt::Display for Cachability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => write!(f, "public"),
            Self::Private => write!(f, "private"),
            Self::NoCache => write!(f, "no-cache"),
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct CacheControl {
    pub cachability: Option<Cachability>,
    pub max_age: Option<Duration>,
    pub s_max_age: Option<Duration>,
}

impl CacheControl {
    pub fn cachability(mut self, cachability: Cachability) -> Self {
        self.cachability = Some(cachability);
        self
    }

    pub fn max_age(mut self, duration: Duration) -> Self {
        self.max_age = Some(duration);
        self
    }

    pub fn s_max_age(mut self, duration: Duration) -> Self {
        self.s_max_age = Some(duration);
        self
    }

    pub fn public(self) -> Self {
        self.cachability(Cachability::Public)
    }
}

impl fmt::Display for CacheControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut need_comma = false;
        macro_rules! w {
            ($e:expr, $fmt:expr) => {
                if let Some(v) = $e {
                    if need_comma {
                        write!(f, ", ")?;
                    }
                    write!(f, $fmt, v)?;
                    #[allow(unused_assignments)]
                    {
                        need_comma = true;
                    }
                }
            };
        }

        w!(self.cachability, "{}");
        w!(self.max_age.map(|d| d.as_secs()), "max-age={}");
        w!(self.s_max_age.map(|d| d.as_secs()), "s-max-age={}");

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct Etag<'a> {
    value: &'a str,
    weak: bool,
    git: bool,
}

impl<'a> Etag<'a> {
    pub fn strong(value: &'a str) -> Self {
        Self {
            value,
            weak: false,
            git: false,
        }
    }

    pub fn weak(value: &'a str) -> Self {
        Self {
            value,
            weak: true,
            git: false,
        }
    }

    pub fn git(mut self) -> Self {
        self.git = true;
        self
    }
}

impl<'a> fmt::Display for Etag<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.weak {
            write!(f, "W/")?;
        }
        write!(f, "\"")?;
        write!(f, "{}", self.value)?;
        if self.git {
            write!(f, ".{}", git_version!())?;
        }
        write!(f, "\"")?;
        Ok(())
    }
}

pub trait RequestExt: Sized {
    fn header(&self, name: &str) -> Option<String>;
    fn referrer(&self) -> Option<url::Url> {
        self.header("Referer")
            .and_then(|v| url::Url::parse(&v).ok())
    }

    fn cookie(&self, name: &str) -> Option<String>;
    fn session(&self) -> Option<String> {
        self.cookie("session")
    }
}

impl RequestExt for Request {
    fn header(&self, name: &str) -> Option<String> {
        self.headers().get(name).ok().flatten()
    }

    fn cookie(&self, name: &str) -> Option<String> {
        let cookie = self.headers().get("Cookie").unwrap()?;

        cookie
            .split(';')
            .filter_map(|part| part.split_once('='))
            .find(|(k, _)| name == k.trim())
            .map(|(_, v)| v.trim().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_path() {
        assert!(to_path("").is_err());
        assert!(to_path("a").is_err());
        assert!(to_path("aa").is_err());
        assert!(to_path("aaa").is_err());
        assert_eq!(to_path("aaaaa").unwrap(), "a/a/aaa");
    }

    #[test]
    fn test_cache_control() {
        assert_eq!(
            "public",
            CacheControl::default()
                .cachability(Cachability::Public)
                .to_string()
        );
        assert_eq!(
            "private",
            CacheControl::default()
                .cachability(Cachability::Private)
                .to_string()
        );
        assert_eq!(
            "no-cache",
            CacheControl::default()
                .cachability(Cachability::NoCache)
                .to_string()
        );
        assert_eq!(
            "s-max-age=123",
            CacheControl::default()
                .s_max_age(Duration::from_secs(123))
                .to_string()
        );
        assert_eq!(
            "max-age=121, s-max-age=123",
            CacheControl::default()
                .max_age(Duration::from_secs(121))
                .s_max_age(Duration::from_secs(123))
                .to_string()
        );
        assert_eq!(
            "public, max-age=121, s-max-age=123",
            CacheControl::default()
                .cachability(Cachability::Public)
                .max_age(Duration::from_secs(121))
                .s_max_age(Duration::from_secs(123))
                .to_string()
        );
    }
}
