use anyhow::{Result, anyhow};
use clap::Parser;

use bin_diff_tool::cli::{Cli, Commands};
use bin_diff_tool::patch::{apply_patch, create_patch, merge_patches, show_patch};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Diff {
            source_dir,
            target_dir,
            output,
        } => {
            if !source_dir.exists() {
                return Err(anyhow!("源目录不存在: {:?}", source_dir));
            }
            if !target_dir.exists() {
                return Err(anyhow!("目标目录不存在: {:?}", target_dir));
            }
            create_patch(&source_dir, &target_dir, &output)?;
        }
        Commands::Apply { target_dir, patch } => {
            if !target_dir.exists() {
                return Err(anyhow!("目标目录不存在: {:?}", target_dir));
            }
            if !patch.exists() {
                return Err(anyhow!("补丁包不存在: {:?}", patch));
            }
            apply_patch(&target_dir, &patch)?;
        }
        Commands::Append {
            first_patch,
            second_patch,
            output,
        } => {
            if !first_patch.exists() {
                return Err(anyhow!("第一个补丁包不存在: {:?}", first_patch));
            }
            if !second_patch.exists() {
                return Err(anyhow!("第二个补丁包不存在: {:?}", second_patch));
            }
            merge_patches(&first_patch, &second_patch, &output)?;
        }
        Commands::Show { patch } => {
            if !patch.exists() {
                return Err(anyhow!("补丁包不存在: {:?}", patch));
            }
            show_patch(&patch)?;
        }
    }

    Ok(())
}
