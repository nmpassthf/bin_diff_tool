use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::HashResult;

/// 补丁包元数据
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub version: String,
    pub created_at: String,
    pub source_version: Option<String>,
    pub target_version: Option<String>,
    pub description: Option<String>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            source_version: None,
            target_version: None,
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

/// 文件校验和信息
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Checksums {
    pub added: HashMap<String, HashResult>,
    pub modified: HashMap<String, ModifiedChecksum>,
    pub deleted: Vec<String>,
}

impl Checksums {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.deleted.is_empty()
    }

    pub fn summary(&self) -> String {
        format!(
            "新增: {} 个文件, 删除: {} 个文件, 修改: {} 个文件",
            self.added.len(),
            self.deleted.len(),
            self.modified.len()
        )
    }
}

/// 修改文件的校验和
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifiedChecksum {
    pub original: HashResult,
    pub modified: HashResult,
}

impl ModifiedChecksum {
    pub fn new(original: HashResult, modified: HashResult) -> Self {
        Self { original, modified }
    }
}
