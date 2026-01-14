use anyhow::Result;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;
use tar::Builder;
use walkdir::WalkDir;

use super::diff::{FileDiff, compare_directories};
use super::metadata::{Checksums, Metadata, ModifiedChecksum};
use crate::utils::compute_file_hash;

/// 生成补丁包
pub fn create_patch(source_dir: &Path, target_dir: &Path, output: &Path) -> Result<()> {
    println!("正在比较目录...");
    let diffs = compare_directories(source_dir, target_dir)?;

    if diffs.is_empty() {
        println!("两个目录完全相同，无需生成补丁包");
        return Ok(());
    }

    // 创建临时目录
    let temp_dir = std::env::temp_dir().join(format!("dft_patch_{}", std::process::id()));
    fs::create_dir_all(&temp_dir)?;

    let added_dir = temp_dir.join("added");
    let deleted_dir = temp_dir.join("deleted");
    let modified_dir = temp_dir.join("modified");

    fs::create_dir_all(&added_dir)?;
    fs::create_dir_all(&deleted_dir)?;
    fs::create_dir_all(&modified_dir)?;

    let mut checksums = Checksums::new();

    println!("正在处理文件差异...");
    for diff in &diffs {
        match diff {
            FileDiff::Added(path) => {
                process_added_file(path, target_dir, &added_dir, &mut checksums)?;
            }
            FileDiff::Deleted(path) => {
                process_deleted_file(path, &mut checksums);
            }
            FileDiff::Modified(path) => {
                process_modified_file(path, source_dir, target_dir, &modified_dir, &mut checksums)?;
            }
        }
    }

    // 创建元数据
    let metadata = Metadata::new();

    // 写入元数据和校验和文件
    write_metadata_files(&temp_dir, &metadata, &checksums)?;

    // 创建 tar.gz 包
    println!("正在创建补丁包...");
    create_tar_gz(&temp_dir, output)?;

    // 清理临时目录
    fs::remove_dir_all(&temp_dir)?;

    println!("补丁包已生成: {}", output.display());
    println!("  {}", checksums.summary());

    Ok(())
}

fn process_added_file(
    path: &Path,
    target_dir: &Path,
    added_dir: &Path,
    checksums: &mut Checksums,
) -> Result<()> {
    let source = target_dir.join(path);
    let dest = added_dir.join(path);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&source, &dest)?;

    let hash = compute_file_hash(&source)?;
    checksums
        .added
        .insert(path.to_string_lossy().to_string(), hash);
    println!("  + {}", path.display());

    Ok(())
}

fn process_deleted_file(path: &Path, checksums: &mut Checksums) {
    checksums.deleted.push(path.to_string_lossy().to_string());
    println!("  - {}", path.display());
}

fn process_modified_file(
    path: &Path,
    source_dir: &Path,
    target_dir: &Path,
    modified_dir: &Path,
    checksums: &mut Checksums,
) -> Result<()> {
    let source_file = source_dir.join(path);
    let target_file = target_dir.join(path);
    let dest = modified_dir.join(path);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    // 对于所有文件，都使用完整替换方式
    fs::copy(&target_file, &dest)?;

    let original_hash = compute_file_hash(&source_file)?;
    let modified_hash = compute_file_hash(&target_file)?;
    checksums.modified.insert(
        path.to_string_lossy().to_string(),
        ModifiedChecksum::new(original_hash, modified_hash),
    );
    println!("  * {}", path.display());

    Ok(())
}

fn write_metadata_files(temp_dir: &Path, metadata: &Metadata, checksums: &Checksums) -> Result<()> {
    let metadata_content = toml::to_string_pretty(metadata)?;
    fs::write(temp_dir.join("metadata.toml"), metadata_content)?;

    let checksums_content = toml::to_string_pretty(checksums)?;
    fs::write(temp_dir.join("checksums.toml"), checksums_content)?;

    Ok(())
}

pub(crate) fn create_tar_gz(source_dir: &Path, output: &Path) -> Result<()> {
    let file = File::create(output)?;
    let encoder = GzEncoder::new(BufWriter::new(file), Compression::default());
    let mut tar_builder = Builder::new(encoder);

    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative_path = path.strip_prefix(source_dir)?;

        if path.is_file() {
            tar_builder.append_path_with_name(path, relative_path)?;
        } else if path.is_dir() && path != source_dir {
            tar_builder.append_dir(relative_path, path)?;
        }
    }

    tar_builder.finish()?;
    Ok(())
}
