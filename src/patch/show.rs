use anyhow::Result;
use std::fs;
use std::path::Path;

use super::apply::{extract_patch, load_checksums};
use super::metadata::Metadata;
use crate::utils::is_text_file;

/// 显示补丁包内容
pub fn show_patch(patch_path: &Path) -> Result<()> {
    println!("补丁包: {}\n", patch_path.display());

    // 创建临时目录
    let temp_dir = std::env::temp_dir().join(format!("dft_show_{}", std::process::id()));
    fs::create_dir_all(&temp_dir)?;

    // 解压补丁包
    extract_patch(patch_path, &temp_dir)?;

    // 显示元数据
    show_metadata(&temp_dir)?;

    // 读取并显示校验和信息
    let checksums = load_checksums(&temp_dir)?;

    // 显示新增文件
    if !checksums.added.is_empty() {
        println!("=== 新增文件 ({}) ===", checksums.added.len());
        for path in checksums.added.keys() {
            println!("  + {}", path);
        }
        println!();
    }

    // 显示删除文件
    if !checksums.deleted.is_empty() {
        println!("=== 删除文件 ({}) ===", checksums.deleted.len());
        for path in &checksums.deleted {
            println!("  - {}", path);
        }
        println!();
    }

    // 显示修改文件
    if !checksums.modified.is_empty() {
        println!("=== 修改文件 ({}) ===", checksums.modified.len());
        for path in checksums.modified.keys() {
            println!("  * {}", path);
            show_text_file_preview(&temp_dir, path)?;
        }
    }

    // 清理临时目录
    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

fn show_metadata(temp_dir: &Path) -> Result<()> {
    let metadata_path = temp_dir.join("metadata.toml");
    if metadata_path.exists() {
        let metadata_content = fs::read_to_string(&metadata_path)?;
        let metadata: Metadata = toml::from_str(&metadata_content)?;
        println!("=== 元数据 ===");
        println!("版本: {}", metadata.version);
        println!("创建时间: {}", metadata.created_at);
        if let Some(desc) = &metadata.description {
            println!("描述: {}", desc);
        }
        println!();
    }
    Ok(())
}

fn show_text_file_preview(temp_dir: &Path, path: &str) -> Result<()> {
    let modified_file = temp_dir.join("modified").join(path);
    if modified_file.exists() && is_text_file(&modified_file) {
        let content = fs::read_to_string(&modified_file)?;
        println!("    --- 新内容 ---");
        for line in content.lines().take(20) {
            println!("    | {}", line);
        }
        if content.lines().count() > 20 {
            println!("    | ... (更多内容省略)");
        }
        println!();
    }
    Ok(())
}
