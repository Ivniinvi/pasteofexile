use serde::{Deserialize, Serialize};
use shared::{
    model::{ListPaste, PasteMetadata},
    PasteId, User,
};

use crate::{
    crypto::Sha1,
    request_context::{Env, FromEnv},
    Result,
};

mod pastebin;
mod r2;
mod utils;

pub(crate) use utils::{strip_prefix, to_path_r2, to_prefix_r2};

#[derive(Debug, Deserialize, Serialize)]
pub struct StoredPaste {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PasteMetadata>,
    #[serde(default)]
    pub last_modified: u64,
    pub entity_id: String,
    pub content: String,
}

pub struct Storage {
    r2: r2::R2Storage,
}

impl FromEnv for Storage {
    fn from_env(env: &Env) -> Option<Self> {
        Some(Self {
            r2: r2::R2Storage::from_env(env)?,
        })
    }
}

impl Storage {
    pub async fn get(&self, id: &PasteId) -> Result<Option<StoredPaste>> {
        if pastebin::could_be_pastebin_id(id) {
            tracing::info!("fetching from pastebin.com");
            return pastebin::get(id).await;
        }

        self.r2.get(id).await
    }

    pub async fn delete(&self, id: &PasteId) -> Result<()> {
        self.r2.delete(id).await
    }

    pub async fn put(
        &self,
        id: &PasteId,
        sha1: &Sha1,
        data: &[u8],
        metadata: Option<&PasteMetadata>,
    ) -> Result<()> {
        self.r2.put(id, sha1, data, metadata).await
    }

    pub async fn list(&self, user: &User) -> Result<Vec<ListPaste>> {
        self.r2.list(user).await
    }
}
