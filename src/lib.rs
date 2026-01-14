//! # Bin Diff Tool
//!
//! 二进制文件增量更新工具库
//!
//! ## 功能
//!
//! - 支持对比两个目录，生成二进制文件差异补丁包
//! - 应用更新补丁包到目标目录，生成更新后的目录
//! - 支持大文件处理，内存占用低
//!
//! ## 使用示例
//!
//! ```no_run
//! use bin_diff_tool::patch::{create_patch, apply_patch, merge_patches, show_patch};
//! use std::path::Path;
//!
//! // 生成补丁包
//! create_patch(
//!     Path::new("old_version"),
//!     Path::new("new_version"),
//!     Path::new("patch.tgz")
//! ).unwrap();
//!
//! // 应用补丁包
//! apply_patch(
//!     Path::new("target_dir"),
//!     Path::new("patch.tgz")
//! ).unwrap();
//! ```

pub mod cli;
pub mod patch;
pub mod utils;

// 重新导出常用类型
pub use patch::{Checksums, FileDiff, Metadata, ModifiedChecksum};
pub use patch::{apply_patch, create_patch, merge_patches, show_patch};
