//! mc_updater — 将补丁包应用到本地 Minecraft mods 目录
//!
//! 功能说明
//!
//! - 默认在当前工作目录查找补丁包 `update.tgz`。
//! - 将补丁应用到 `./.minecraft/versions/NeoForge/mods` 目录下。
//! - 如果目标目录不存在，会尝试创建该目录（递归创建父目录）。
//! - 使用库函数 `bin_diff_tool::patch::apply_patch` 实际执行解压与文件变更。
//! - 在错误或补丁缺失时打印清晰的错误信息并以非零退出码退出。
//! - 运行结束前会等待一个按键以便在交互式环境下查看输出。
//!
//! 使用方法
//!
//! 1. 将补丁包命名为 `update.tgz` 放在当前目录。
//! 2. 运行：
//!
//! ```text
//! cargo run --bin mc_updater
//! ```
//!
//! 退出码
//!
//! - `0`：补丁成功应用且程序正常退出。
//! - 非 `0`：发生错误（例如补丁不存在、无法创建目标目录、应用补丁失败等）。
//!
//! 注意与故障排查
//!
//! - 补丁包应符合本仓库补丁格式（包含 `checksums.toml`、`added/modified/` 目录等），否则
//!   `apply_patch` 可能报错。
//! - 如果遇到权限问题，请确认当前用户对目标目录具有写权限。
//! - 如果补丁应用过程中出现校验和不匹配，程序会打印警告但仍继续应用（由 `apply_patch` 控制）。
//!
//! 无需额外命令行参数。本文件是一个小型交互式工具，适用于本地手动更新场景。
use anyhow::{Context, Result};
use std::io::{self, Read, Write};
use std::path::PathBuf;

fn wait_for_key() {
    print!("按任意键继续...");
    let _ = io::stdout().flush();
    let mut b = [0u8];
    let _ = io::stdin().read(&mut b);
}

fn run() -> Result<()> {
    let cwd = std::env::current_dir()?;

    let patch_path: PathBuf = cwd.join("update.tgz");
    if !patch_path.exists() {
        eprintln!("补丁文件未找到: {}", patch_path.display());
        eprintln!("请把 update.tgz 放在当前目录后重试。");
        wait_for_key();
        std::process::exit(1);
    }

    let target_dir: PathBuf = cwd.join(".minecraft/versions/NeoForge/mods");
    if !target_dir.exists() {
        println!("目标目录不存在，正在创建: {}", target_dir.display());
        std::fs::create_dir_all(&target_dir)
            .with_context(|| format!("无法创建目标目录: {}", target_dir.display()))?;
    }

    println!("使用补丁: {}", patch_path.display());
    println!("目标目录: {}", target_dir.display());

    bin_diff_tool::patch::apply_patch(&target_dir, &patch_path)
        .with_context(|| format!("应用补丁失败: {}", patch_path.display()))?;

    wait_for_key();
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("错误: {:#}", err);
        wait_for_key();
        std::process::exit(1);
    }
}
