use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::hash::{HashResult, compute_file_hash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileInfo {
    pub hash: HashResult,
    pub fsize: usize,
}

/// 获取目录下所有文件的相对路径和哈希值
pub fn scan_directory(dir: &Path) -> Result<HashMap<PathBuf, FileInfo>> {
    let mut files = HashMap::new();

    if !dir.exists() {
        return Ok(files);
    }

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let relative_path = path
            .strip_prefix(dir)
            .with_context(|| format!("无法获取相对路径: {:?}", path))?
            .to_path_buf();

        let hash = compute_file_hash(path)?;
        let fsize = path.metadata()?.len() as usize;
        files.insert(relative_path, FileInfo { hash, fsize });
    }

    Ok(files)
}

/// 判断文件是否为文本文件
pub fn is_text_file(path: &Path) -> bool {
    const TEXT_EXTENSIONS: &[&str] = &[
        "txt", "md", "json", "xml", "html", "css", "js", "ts", "py", "rs", "go", "java", "c", "h",
        "cpp", "hpp", "toml", "yaml", "yml", "ini", "cfg", "conf", "sh", "bat", "ps1", "sql",
    ];

    if let Some(ext) = path.extension()
        && let Some(ext_str) = ext.to_str()
    {
        return TEXT_EXTENSIONS.contains(&ext_str.to_lowercase().as_str());
    }

    // 尝试读取文件头部判断
    if let Ok(file) = File::open(path) {
        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 512];
        if let Ok(bytes_read) = reader.read(&mut buffer) {
            // 检查是否包含空字节，如果包含则可能是二进制文件
            return !buffer[..bytes_read].contains(&0);
        }
    }

    false
}
