use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// 二进制文件增量更新工具
#[derive(Parser)]
#[command(name = "dft")]
#[command(about = "二进制文件增量更新工具", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 对比两个目录，生成补丁包
    Diff {
        /// 源目录 (旧版本)
        source_dir: PathBuf,
        /// 目标目录 (新版本)
        target_dir: PathBuf,
        /// 输出补丁包路径
        #[arg(short, long)]
        output: PathBuf,
    },
    /// 应用补丁包到目标目录
    Apply {
        /// 目标目录
        target_dir: PathBuf,
        /// 补丁包路径
        #[arg(short, long)]
        patch: PathBuf,
    },
    /// 合并两个补丁包
    Append {
        /// 第一个补丁包 (较早版本)
        first_patch: PathBuf,
        /// 第二个补丁包 (较新版本)
        second_patch: PathBuf,
        /// 输出合并后的补丁包路径
        #[arg(short, long)]
        output: PathBuf,
    },
    /// 显示补丁包内容
    Show {
        /// 补丁包路径
        patch: PathBuf,
    },
}
