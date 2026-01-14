use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::utils::scan_directory;

/// 文件差异类型
#[derive(Debug)]
pub enum FileDiff {
    Added(PathBuf),
    Deleted(PathBuf),
    Modified(PathBuf),
}

impl FileDiff {
    pub fn path(&self) -> &PathBuf {
        match self {
            FileDiff::Added(p) | FileDiff::Deleted(p) | FileDiff::Modified(p) => p,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            FileDiff::Added(_) => "+",
            FileDiff::Deleted(_) => "-",
            FileDiff::Modified(_) => "*",
        }
    }
}

/// 比较两个目录并返回差异
pub fn compare_directories(source_dir: &Path, target_dir: &Path) -> Result<Vec<FileDiff>> {
    let source_files = scan_directory(source_dir)?;
    let target_files = scan_directory(target_dir)?;

    let mut diffs = Vec::new();

    // 检查新增和修改的文件
    for (path, target_hash) in &target_files {
        if let Some(source_hash) = source_files.get(path) {
            if source_hash != target_hash {
                diffs.push(FileDiff::Modified(path.clone()));
            }
        } else {
            diffs.push(FileDiff::Added(path.clone()));
        }
    }

    // 检查删除的文件
    for path in source_files.keys() {
        if !target_files.contains_key(path) {
            diffs.push(FileDiff::Deleted(path.clone()));
        }
    }

    Ok(diffs)
}
