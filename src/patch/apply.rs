use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use tar::Archive;
use walkdir::WalkDir;

use super::metadata::Checksums;
use crate::utils::compute_file_hash;

/// 应用补丁包
pub fn apply_patch(target_dir: &Path, patch_path: &Path) -> Result<()> {
    println!("正在解压补丁包...");

    // 创建临时目录
    let temp_dir = std::env::temp_dir().join(format!("dft_apply_{}", std::process::id()));
    fs::create_dir_all(&temp_dir)?;

    // 解压补丁包
    extract_patch(patch_path, &temp_dir)?;

    // 读取校验和信息
    let checksums = load_checksums(&temp_dir)?;

    println!("正在应用补丁...");

    // 删除文件
    apply_deletions(target_dir, &checksums)?;

    // 添加新文件
    apply_additions(target_dir, &temp_dir, &checksums)?;

    // 应用修改
    apply_modifications(target_dir, &temp_dir, &checksums)?;

    // 清理临时目录
    fs::remove_dir_all(&temp_dir)?;

    println!("补丁应用完成!");
    Ok(())
}

pub(crate) fn extract_patch(patch_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = File::open(patch_path)?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let mut archive = Archive::new(decoder);
    archive.unpack(dest_dir)?;
    Ok(())
}

pub(crate) fn load_checksums(temp_dir: &Path) -> Result<Checksums> {
    let checksums_path = temp_dir.join("checksums.toml");
    let checksums_content =
        fs::read_to_string(&checksums_path).with_context(|| "无法读取 checksums.toml")?;
    let checksums: Checksums =
        toml::from_str(&checksums_content).with_context(|| "无法解析 checksums.toml")?;
    Ok(checksums)
}

fn apply_deletions(target_dir: &Path, checksums: &Checksums) -> Result<()> {
    for deleted_file in &checksums.deleted {
        let target_path = target_dir.join(deleted_file);
        if target_path.exists() {
            fs::remove_file(&target_path)?;
            println!("  - {}", deleted_file);

            // 清理空目录
            if let Some(parent) = target_path.parent() {
                let _ = fs::remove_dir(parent); // 忽略错误，目录可能非空
            }
        }
    }
    Ok(())
}

fn apply_additions(target_dir: &Path, temp_dir: &Path, _checksums: &Checksums) -> Result<()> {
    let added_dir = temp_dir.join("added");
    if !added_dir.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(&added_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let relative_path = entry.path().strip_prefix(&added_dir)?;
            let target_path = target_dir.join(relative_path);

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target_path)?;
            println!("  + {}", relative_path.display());
        }
    }
    Ok(())
}

fn apply_modifications(target_dir: &Path, temp_dir: &Path, checksums: &Checksums) -> Result<()> {
    let modified_dir = temp_dir.join("modified");
    if !modified_dir.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(&modified_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            let relative_path = entry.path().strip_prefix(&modified_dir)?;
            let target_path = target_dir.join(relative_path);

            // 验证原始文件校验和
            verify_original_checksum(&target_path, relative_path, checksums)?;

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target_path)?;
            println!("  * {}", relative_path.display());
        }
    }
    Ok(())
}

fn verify_original_checksum(
    target_path: &Path,
    relative_path: &Path,
    checksums: &Checksums,
) -> Result<()> {
    let relative_str = relative_path.to_string_lossy().to_string();
    if let Some(checksum) = checksums.modified.get(&relative_str)
        && target_path.exists()
    {
        let current_hash = compute_file_hash(target_path)?;
        if current_hash != checksum.original {
            println!(
                "  ! 警告: {} 的校验和不匹配，可能已被修改",
                relative_path.display()
            );
        }
    }
    Ok(())
}
