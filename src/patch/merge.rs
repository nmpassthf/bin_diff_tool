use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use super::apply::{extract_patch, load_checksums};
use super::create::create_tar_gz;
use super::metadata::{Checksums, Metadata, ModifiedChecksum};

/// 合并两个补丁包
pub fn merge_patches(first: &Path, second: &Path, output: &Path) -> Result<()> {
    println!("正在合并补丁包...");

    // 创建临时目录
    let temp_dir = std::env::temp_dir().join(format!("dft_append_{}", std::process::id()));
    let first_dir = temp_dir.join("first");
    let second_dir = temp_dir.join("second");
    let merged_dir = temp_dir.join("merged");

    fs::create_dir_all(&first_dir)?;
    fs::create_dir_all(&second_dir)?;
    fs::create_dir_all(&merged_dir)?;

    // 解压两个补丁包
    extract_patch(first, &first_dir)?;
    extract_patch(second, &second_dir)?;

    // 读取两个补丁包的校验和
    let checksums1 = load_checksums(&first_dir)?;
    let checksums2 = load_checksums(&second_dir)?;

    // 合并校验和
    let merged_checksums = merge_checksums(&checksums1, &checksums2);

    // 创建合并后的目录结构
    setup_merged_directories(&merged_dir)?;

    // 复制文件
    copy_merged_files(&first_dir, &second_dir, &merged_dir, &merged_checksums)?;

    // 创建元数据
    let metadata = Metadata::new().with_description("合并补丁包");

    // 写入元数据和校验和
    write_merged_metadata(&merged_dir, &metadata, &merged_checksums)?;

    // 创建 tar.gz 包
    create_tar_gz(&merged_dir, output)?;

    // 清理临时目录
    fs::remove_dir_all(&temp_dir)?;

    println!("补丁包合并完成: {}", output.display());
    println!("  {}", merged_checksums.summary());

    Ok(())
}

fn merge_checksums(checksums1: &Checksums, checksums2: &Checksums) -> Checksums {
    let mut merged = Checksums::new();
    let second_deleted: HashSet<_> = checksums2.deleted.iter().collect();

    // 处理第一个补丁的新增文件
    merge_added_files(&mut merged, checksums1, checksums2, &second_deleted);

    // 处理第二个补丁的新增文件
    for (path, hash) in &checksums2.added {
        if !merged.added.contains_key(path) {
            merged.added.insert(path.clone(), hash.clone());
        }
    }

    // 处理修改文件
    merge_modified_files(&mut merged, checksums1, checksums2, &second_deleted);

    // 处理删除文件
    merge_deleted_files(&mut merged, checksums1, checksums2);

    merged
}

fn merge_added_files(
    merged: &mut Checksums,
    checksums1: &Checksums,
    checksums2: &Checksums,
    second_deleted: &HashSet<&String>,
) {
    for (path, hash) in &checksums1.added {
        if second_deleted.contains(path) {
            // 如果在第二个补丁中被删除，则不包含
            continue;
        }
        if let Some(second_modified) = checksums2.modified.get(path) {
            // 如果在第二个补丁中被修改，使用修改后的版本
            merged
                .added
                .insert(path.clone(), second_modified.modified.clone());
        } else if checksums2.added.contains_key(path) {
            // 如果在第二个补丁中也被添加，使用第二个版本
            merged
                .added
                .insert(path.clone(), checksums2.added.get(path).unwrap().clone());
        } else {
            merged.added.insert(path.clone(), hash.clone());
        }
    }
}

fn merge_modified_files(
    merged: &mut Checksums,
    checksums1: &Checksums,
    checksums2: &Checksums,
    second_deleted: &HashSet<&String>,
) {
    for (path, checksum) in &checksums1.modified {
        if second_deleted.contains(path) {
            // 修改后被删除，记录为删除
            merged.deleted.push(path.clone());
            continue;
        }
        if let Some(second_checksum) = checksums2.modified.get(path) {
            // 两次都被修改，合并为一次修改
            merged.modified.insert(
                path.clone(),
                ModifiedChecksum::new(checksum.original.clone(), second_checksum.modified.clone()),
            );
        } else {
            merged.modified.insert(path.clone(), checksum.clone());
        }
    }

    for (path, checksum) in &checksums2.modified {
        if !merged.modified.contains_key(path) && !merged.added.contains_key(path) {
            merged.modified.insert(path.clone(), checksum.clone());
        }
    }
}

fn merge_deleted_files(merged: &mut Checksums, checksums1: &Checksums, checksums2: &Checksums) {
    for path in &checksums1.deleted {
        if checksums2.added.contains_key(path) {
            // 删除后又添加，简化处理为添加
            continue;
        }
        if !merged.deleted.contains(path) {
            merged.deleted.push(path.clone());
        }
    }

    for path in &checksums2.deleted {
        if !merged.deleted.contains(path) && !checksums1.added.contains_key(path) {
            merged.deleted.push(path.clone());
        }
    }
}

fn setup_merged_directories(merged_dir: &Path) -> Result<()> {
    fs::create_dir_all(merged_dir.join("added"))?;
    fs::create_dir_all(merged_dir.join("modified"))?;
    fs::create_dir_all(merged_dir.join("deleted"))?;
    Ok(())
}

fn copy_merged_files(
    first_dir: &Path,
    second_dir: &Path,
    merged_dir: &Path,
    checksums: &Checksums,
) -> Result<()> {
    let merged_added = merged_dir.join("added");
    let merged_modified = merged_dir.join("modified");

    // 复制新增文件
    for path in checksums.added.keys() {
        let source = find_added_source(first_dir, second_dir, path);
        let dest = merged_added.join(path);

        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        if source.exists() {
            fs::copy(&source, &dest)?;
        }
    }

    // 复制修改文件
    for path in checksums.modified.keys() {
        let source = if second_dir.join("modified").join(path).exists() {
            second_dir.join("modified").join(path)
        } else {
            first_dir.join("modified").join(path)
        };

        let dest = merged_modified.join(path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        if source.exists() {
            fs::copy(&source, &dest)?;
        }
    }

    Ok(())
}

fn find_added_source(first_dir: &Path, second_dir: &Path, path: &str) -> std::path::PathBuf {
    if second_dir.join("added").join(path).exists() {
        second_dir.join("added").join(path)
    } else if second_dir.join("modified").join(path).exists() {
        second_dir.join("modified").join(path)
    } else {
        first_dir.join("added").join(path)
    }
}

fn write_merged_metadata(
    merged_dir: &Path,
    metadata: &Metadata,
    checksums: &Checksums,
) -> Result<()> {
    fs::write(
        merged_dir.join("metadata.toml"),
        toml::to_string_pretty(metadata)?,
    )?;
    fs::write(
        merged_dir.join("checksums.toml"),
        toml::to_string_pretty(checksums)?,
    )?;
    Ok(())
}
