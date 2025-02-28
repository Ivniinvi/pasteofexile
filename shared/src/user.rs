use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct User(String);

impl User {
    pub fn new_unchecked(user: String) -> Self {
        Self(user)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Usernames are case insensitive,
    /// returns a normalized version of the username (lowercase).
    pub fn normalized(&self) -> Self {
        Self(self.0.to_lowercase())
    }
}

impl std::ops::Deref for User {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl AsRef<str> for User {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<User> for String {
    fn from(user: User) -> Self {
        user.0
    }
}

impl From<&User> for User {
    fn from(user: &User) -> Self {
        user.clone()
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvalidUser {
    #[error("Username too long")]
    TooLong,
    #[error("Invalid Username")]
    Invalid,
}

impl FromStr for User {
    type Err = InvalidUser;

    fn from_str(username: &str) -> Result<Self, Self::Err> {
        let mut count = 0usize;
        for c in username.chars() {
            if matches!(c, '/' | ':') {
                return Err(Self::Err::Invalid);
            }
            count += 1;

            if count > 30 {
                return Err(Self::Err::TooLong);
            }
        }

        Ok(Self(username.into()))
    }
}

impl TryFrom<String> for User {
    type Error = InvalidUser;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
