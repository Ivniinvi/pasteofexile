use serde::{Deserialize, Serialize};

mod paste;

pub use paste::*;

#[derive(Debug)]
pub struct ListPaste {
    pub name: String, // TODO: this should be a PasteId I think
    pub metadata: Option<PasteMetadata>,
    pub last_modified: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Paste {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PasteMetadata>,
    #[serde(default, skip_serializing_if = "crate::utils::is_zero")]
    pub last_modified: u64,
    pub entity_id: String,
    pub content: String,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct PasteMetadata {
    pub title: String,
    pub ascendancy_or_class: String,
    pub version: Option<String>,
    pub main_skill_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PasteSummary {
    pub id: String,
    pub user: Option<crate::User>,
    pub title: String,
    pub ascendancy_or_class: String,
    pub version: String,
    pub main_skill_name: String,
    pub last_modified: u64,
}

impl PasteSummary {
    pub fn to_url(&self) -> String {
        if let Some(ref user) = self.user {
            format!("/u/{user}/{}", self.id)
        } else {
            format!("/{}", self.id)
        }
    }
}
