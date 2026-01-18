use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use sr_bindings::{list_models, process_image};
use std::path::PathBuf;

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

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.list_models {
        let models = list_models().map_err(anyhow::Error::msg)?;
        println!("Available models:\n");
        println!("{}", models);
        println!("\nUsage examples:");
        println!("  sr -i input.jpg -o output.webp --scale 2");
        println!("  sr -i input.jpg -o output.webp --model waifu2x_cunet_up2x");
        return Ok(());
    }

    let input = cli.input.unwrap_or_else(|| {
        eprintln!("Error: Please specify input file (-i/--input)");
        std::process::exit(1);
    });

    let output = cli.output.unwrap_or_else(|| {
        eprintln!("Error: Please specify output file (-o/--output)");
        std::process::exit(1);
    });

    let model = cli.model.unwrap_or_else(|| {
        eprintln!(
            "Error: Please specify model name (--model), use --list-models to list all models"
        );
        std::process::exit(1);
    });

    if !input.exists() {
        anyhow::bail!("Input file not found: {:?}", input);
    }

    let spinner_style = ProgressStyle::with_template("{spinner:.cyan} {msg}")
        .unwrap()
        .tick_strings(&[
            "▹▹▹▹▹",
            "▸▹▹▹▹",
            "▹▸▹▹▹",
            "▹▹▸▹▹",
            "▹▹▹▸▹",
            "▹▹▹▹▸",
            "▪▪▪▪▪",
        ]);

    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style);
    pb.set_message("Processing image...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let (success, message) = process_image(
        input.to_str().unwrap_or(""),
        output.to_str().unwrap_or(""),
        cli.scale,
        &model,
        cli.gpu_id,
        cli.cpu,
        cli.model_path.as_ref().map(|p| p.to_str().unwrap_or("")),
    )
    .map_err(|e| anyhow::anyhow!("Image processing failed: {}", e))?;

    if success {
        let res = format!("Done! Time: {}", message);
        pb.finish_with_message(res);
    } else {
        pb.finish_with_message("Failed");
        anyhow::bail!("Processing failed: {}", message);
    }

    Ok(())
}
