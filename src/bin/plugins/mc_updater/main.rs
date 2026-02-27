//! mc_updater — 将补丁包应用到本地 Minecraft mods 目录
//!
//! 功能说明
//!
//! - 从命令行参数中读取一个或多个补丁包（`.tgz`），按补丁内 `metadata.toml` 中的 `created_at` 时间排序后依次合并。
//! - 将最终合并得到的补丁应用到 `./.minecraft/versions/NeoForge/mods` 目录下。
//! - 如果目标目录不存在，程序会报错并提示用户确认当前工作目录是否正确。
//! - 合并多个补丁时，会在系统临时目录中创建中间文件用于过渡合并。
//! - 使用库函数 `bin_diff_tool::patch::apply_patch` 实际执行解压与文件变更。
//! - 在错误或补丁缺失时打印清晰的错误信息并以非零退出码退出。
//! - 运行结束前会等待一个按键以便在交互式环境下查看输出。
//!
//! 使用方法
//!
//! 1. 将一个或多个补丁包拖入程序，或在命令行中作为参数传入：
//!
//! ```text
//! mc_updater patch1.tgz patch2.tgz ...
//! ```
//!
//! 2. 程序会自动按补丁创建时间顺序合并并应用。
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
use anyhow::{Context, Result, bail};
use bin_diff_tool::merge_patches;
use chrono::DateTime;
use chrono::Utc;
use std::io::{self, Read, Write};
use std::path::Path;
use std::path::PathBuf;

use flate2::read::GzDecoder;
use tar::Archive;
use toml::Table;

struct Patch {
    path: PathBuf,
    created_at: DateTime<Utc>,
}

fn wait_for_key() {
    print!("按回车退出...");
    let _ = io::stdout().flush();
    let mut b = [0u8];
    let _ = io::stdin().read(&mut b);
}

fn check_mod_folder<T>(relative_path: T) -> Result<PathBuf>
where
    T: AsRef<Path>,
{
    let cwd = std::env::current_dir()?;
    let target_dir: PathBuf = cwd.join(relative_path);

    if !target_dir.exists() {
        bail!(
            "目标目录不存在: {}, 你真的在 minecraft 目录下吗？",
            target_dir.display()
        );
    }

    Ok(target_dir)
}

fn create_patch_list(args: Vec<PathBuf>) -> Result<Vec<Patch>> {
    let mut patches: Vec<Patch> = Vec::new();

    for patch in args {
        if !patch.exists() {
            bail!("补丁文件未找到: {}", patch.display())
        }

        let date = parse_create_time(&patch)?;
        patches.push(Patch {
            path: patch,
            created_at: date,
        });
    }

    Ok(patches)
}

fn parse_create_time(patch: &Path) -> Result<DateTime<Utc>> {
    let file = std::fs::File::open(patch).context("文件打开失败")?;
    let gz = GzDecoder::new(file);
    let mut tar = Archive::new(gz);

    for entry in tar.entries()? {
        let mut entry = entry?;

        if entry
            .path()?
            .file_name()
            .is_none_or(|name| name != "metadata.toml")
        {
            continue;
        }

        let mut s = String::new();
        entry.read_to_string(&mut s)?;

        let table: Table = s.parse()?;

        let time_str = table
            .get("created_at")
            .and_then(|v| v.as_str())
            .context("缺失 created_at 字段")?;

        return time_str.parse::<DateTime<Utc>>().context("非法的时间格式");
    }

    bail!("数据包失效：无法找到 metadata.toml")
}

fn create_merge_tgz(patches: &[Patch], temp_dir: &Path) -> Result<PathBuf> {
    let (first, rest) = patches.split_first().context("补丁文件未找到")?;
    let mut current = first.path.clone();

    for (i, patch) in rest.iter().enumerate() {
        let output = temp_dir.join(format!("merge_{}.tgz", i));
        merge_patches(&current, &patch.path, &output)?;
        current = output;
    }

    Ok(current)
}

fn run() -> Result<()> {
    let target_dir = check_mod_folder(".minecraft/versions/NeoForge/mods")?;

    let args: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();

    if args.is_empty() {
        bail!("请拖入补丁包文件")
    }

    let mut patches = create_patch_list(args)?;

    patches.sort_by_key(|p| p.created_at);

    let merge_dir = std::env::temp_dir().join(format!("mc_updater_{}", std::process::id()));
    std::fs::create_dir_all(&merge_dir)?;

    let merge_tgz = create_merge_tgz(&patches, &merge_dir)?;

    bin_diff_tool::patch::apply_patch(&target_dir, &merge_tgz)
        .with_context(|| format!("应用补丁失败: {}", merge_tgz.display()))?;

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
