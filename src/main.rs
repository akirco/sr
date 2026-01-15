use anyhow::{Context, Result};
use clap::Parser;
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "sr")]
#[command(version = "0.2.0")]
struct Cli {
    #[arg(short, long)]
    input: Option<PathBuf>,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(short, long, default_value = "2.0")]
    scale: f32,
    #[arg(short, long)]
    model: Option<String>,
    #[arg(long, default_value = "0")]
    gpu_id: i32,
    #[arg(long)]
    cpu: bool,
    #[arg(long)]
    list_models: bool,
    #[arg(long)]
    model_path: Option<PathBuf>,
}

fn process_image(
    input: &Path,
    output: &Path,
    scale: f32,
    model: &str,
    gpu_id: i32,
    cpu: bool,
    model_path: Option<&Path>,
) -> Result<()> {
    Python::attach(|py| {
        let processor = py.import("image.processor")?;
        let model_path_str = model_path.map(|p| p.to_str().unwrap_or(""));
        let result = processor
            .call_method(
                "process_image",
                (
                    input.to_str().unwrap_or(""),
                    output.to_str().unwrap_or(""),
                    scale,
                    model,
                    gpu_id,
                    cpu,
                    model_path_str,
                ),
                None,
            )?
            .extract::<(bool, String)>()?;
        if result.0 {
            println!("{}", result.1);
            Ok(())
        } else {
            anyhow::bail!("处理失败: {}", result.1)
        }
    })
}

fn list_models() -> Result<()> {
    Python::attach(|py| {
        let processor = py.import("image.processor")?;
        let categories: String = processor
            .call_method0("get_model_categories_formatted")?
            .extract()?;

        println!("可用模型:\n");
        println!("{}", categories);

        println!("使用示例:");
        println!("  sr -i input.jpg -o output.webp --scale 2");
        println!("  sr -i input.jpg -o output.webp --model waifu2x_cunet_up2x");

        Ok(())
    })
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_models {
        list_models()?;
        return Ok(());
    }

    let input = cli.input.unwrap_or_else(|| {
        eprintln!("错误: 请指定输入文件 (-i/--input)");
        std::process::exit(1);
    });

    let output = cli.output.unwrap_or_else(|| {
        eprintln!("错误: 请指定输出文件 (-o/--output)");
        std::process::exit(1);
    });

    let model = cli.model.unwrap_or_else(|| {
        eprintln!("错误: 请指定模型名称 (--model)，使用 --list-models 查看所有模型");
        std::process::exit(1);
    });

    if !input.exists() {
        anyhow::bail!("输入文件不存在: {:?}", input);
    }

    process_image(
        &input,
        &output,
        cli.scale,
        &model,
        cli.gpu_id,
        cli.cpu,
        cli.model_path.as_deref(),
    )
    .context("图片处理失败")?;

    Ok(())
}
