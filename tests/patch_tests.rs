use anyhow::Result;
use bin_diff_tool::patch::{
    apply_patch, compare_directories, create_patch, merge_patches, show_patch,
};
use bin_diff_tool::utils::{compute_file_hash, is_text_file, scan_directory};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;
use walkdir::WalkDir;

static PATCH_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn patch_lock() -> std::sync::MutexGuard<'static, ()> {
    PATCH_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn write_file(root: &Path, relative: &str, contents: &[u8]) -> PathBuf {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, contents).unwrap();
    path
}

fn copy_dir(source: &Path, dest: &Path) {
    for entry in WalkDir::new(source).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(source).unwrap();
            let target = dest.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(entry.path(), &target).unwrap();
        }
    }
}

#[test]
fn compute_file_hash_matches_expected_value() -> Result<()> {
    let dir = TempDir::new()?;
    let file = write_file(dir.path(), "hash.txt", b"hello world");

    let hash = compute_file_hash(&file)?;

    assert_eq!(
        hash,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
    Ok(())
}

#[test]
fn detect_text_and_binary_files() -> Result<()> {
    let dir = TempDir::new()?;
    let text_file = write_file(dir.path(), "note.md", b"# hello");
    let binary_file = write_file(dir.path(), "image.bin", &[0, 1, 2, 0]);

    assert!(is_text_file(&text_file));
    assert!(!is_text_file(&binary_file));
    Ok(())
}

#[test]
fn scan_directory_collects_relative_paths_and_hashes() -> Result<()> {
    let dir = TempDir::new()?;
    let file_a = write_file(dir.path(), "a.txt", b"one");
    let file_b = write_file(dir.path(), "nested/b.txt", b"two");

    let files = scan_directory(dir.path())?;

    assert_eq!(files.len(), 2);
    assert_eq!(
        &files
            .get(&file_a.strip_prefix(dir.path())?.to_path_buf())
            .unwrap()
            .hash,
        &compute_file_hash(&file_a)?
    );
    assert_eq!(
        &files
            .get(&file_b.strip_prefix(dir.path())?.to_path_buf())
            .unwrap()
            .hash,
        &compute_file_hash(&file_b)?
    );
    Ok(())
}

#[test]
fn compare_directories_finds_added_deleted_modified() -> Result<()> {
    let source = TempDir::new()?;
    let target = TempDir::new()?;

    write_file(source.path(), "same.txt", b"same");
    write_file(source.path(), "removed.txt", b"old");
    write_file(source.path(), "changed.txt", b"before");

    write_file(target.path(), "same.txt", b"same");
    write_file(target.path(), "added.txt", b"new");
    write_file(target.path(), "changed.txt", b"after");

    let diffs = compare_directories(source.path(), target.path())?;
    let diff_set: HashSet<(String, String)> = diffs
        .into_iter()
        .map(|d| {
            (
                d.symbol().to_string(),
                d.path().to_string_lossy().to_string(),
            )
        })
        .collect();

    assert_eq!(diff_set.len(), 3);
    assert!(diff_set.contains(&("+".to_string(), "added.txt".to_string())));
    assert!(diff_set.contains(&("-".to_string(), "removed.txt".to_string())));
    assert!(diff_set.contains(&("*".to_string(), "changed.txt".to_string())));
    Ok(())
}

#[test]
fn create_and_apply_patch_produces_expected_directory() -> Result<()> {
    let _guard = patch_lock();

    let source = TempDir::new()?;
    let target = TempDir::new()?;
    let patch_dir = TempDir::new()?;
    let output = patch_dir.path().join("patch.tgz");

    write_file(source.path(), "keep.txt", b"same");
    write_file(source.path(), "remove.txt", b"old");
    write_file(source.path(), "change.txt", b"v1");

    write_file(target.path(), "keep.txt", b"same");
    write_file(target.path(), "change.txt", b"v2");
    write_file(target.path(), "add/new.txt", b"new file");

    create_patch(source.path(), target.path(), &output)?;
    assert!(output.exists());

    let apply_dir = TempDir::new()?;
    copy_dir(source.path(), apply_dir.path());

    apply_patch(apply_dir.path(), &output)?;

    let expected = scan_directory(target.path())?;
    let actual = scan_directory(apply_dir.path())?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn merge_patches_applies_changes_from_both_inputs() -> Result<()> {
    let _guard = patch_lock();

    let base = TempDir::new()?;
    let mid = TempDir::new()?;
    let final_dir = TempDir::new()?;
    let patch_dir = TempDir::new()?;
    let patch_one = patch_dir.path().join("one.tgz");
    let patch_two = patch_dir.path().join("two.tgz");
    let merged_patch = patch_dir.path().join("merged.tgz");

    write_file(base.path(), "stay.txt", b"base");
    write_file(base.path(), "edit.txt", b"v1");
    write_file(base.path(), "drop.txt", b"remove me");

    write_file(mid.path(), "stay.txt", b"base");
    write_file(mid.path(), "edit.txt", b"v2");
    write_file(mid.path(), "new_mid.txt", b"mid add");

    write_file(final_dir.path(), "stay.txt", b"base");
    write_file(final_dir.path(), "edit.txt", b"v3");
    write_file(final_dir.path(), "new_mid.txt", b"mid add updated");
    write_file(final_dir.path(), "final_only.txt", b"final add");

    create_patch(base.path(), mid.path(), &patch_one)?;
    create_patch(mid.path(), final_dir.path(), &patch_two)?;
    merge_patches(&patch_one, &patch_two, &merged_patch)?;

    let apply_dir = TempDir::new()?;
    copy_dir(base.path(), apply_dir.path());
    apply_patch(apply_dir.path(), &merged_patch)?;

    let expected = scan_directory(final_dir.path())?;
    let actual = scan_directory(apply_dir.path())?;
    assert_eq!(expected, actual);
    Ok(())
}

#[test]
fn show_patch_can_inspect_generated_patch() -> Result<()> {
    let _guard = patch_lock();

    let source = TempDir::new()?;
    let target = TempDir::new()?;
    let patch = source.path().join("display.tgz");

    write_file(source.path(), "file.txt", b"old");
    write_file(target.path(), "file.txt", b"new");

    create_patch(source.path(), target.path(), &patch)?;

    // Should succeed without panicking or failing.
    show_patch(&patch)?;
    Ok(())
}

#[test]
#[should_panic]
fn apply_patch_panics_on_corrupted_archive() {
    let _guard = patch_lock();

    let target = TempDir::new().unwrap();
    let corrupted_patch = TempDir::new().unwrap().path().join("bad.tgz");
    fs::write(&corrupted_patch, b"not a tar.gz").unwrap();

    // unwrap to force panic when apply_patch returns Err
    apply_patch(target.path(), &corrupted_patch).unwrap();
}

#[test]
#[should_panic]
fn merge_patches_panics_with_invalid_inputs() {
    let _guard = patch_lock();

    let patch_dir = TempDir::new().unwrap();
    let first = patch_dir.path().join("one.tgz");
    let second = patch_dir.path().join("two.tgz");
    let output = patch_dir.path().join("merged.tgz");

    fs::write(&first, b"garbage").unwrap();
    fs::write(&second, b"garbage").unwrap();

    merge_patches(&first, &second, &output).unwrap();
}
