use anyhow::Result;
use clap::Parser;
use sr_bindings::{list_models, process_image};
use std::io::{BufRead, BufReader, Write};
use std::os::fd::FromRawFd;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

struct ProgressState {
    done: AtomicBool,
    progress: AtomicU64,
}

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
    #[arg(short, long, default_value = "REALESRGAN_X4PLUS_UP4X")]
    model: Option<String>,
    #[arg(long, default_value = "0")]
    gpu_id: i32,
    #[arg(long)]
    cpu: bool,
    #[arg(long)]
    list_models: bool,
    #[arg(long)]
    model_path: Option<PathBuf>,
    #[arg(short, long)]
    verbose: bool,
}

struct StderrCapture {
    saved_fd: i32,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl StderrCapture {
    fn start(state: Arc<ProgressState>) -> Option<Self> {
        let mut fds: [i32; 2] = [0, 0];
        unsafe {
            if libc::pipe(fds.as_mut_ptr()) != 0 {
                return None;
            }
        }
        let read_fd = fds[0];
        let write_fd = fds[1];

        let saved_fd = unsafe { libc::dup(2) };
        if saved_fd < 0 {
            unsafe {
                libc::close(read_fd);
                libc::close(write_fd);
            }
            return None;
        }

        unsafe {
            libc::dup2(write_fd, 2);
            libc::close(write_fd);
        }

        let thread = std::thread::spawn(move || {
            let file = unsafe { std::fs::File::from_raw_fd(read_fd) };
            for line in BufReader::new(file).lines().map_while(Result::ok) {
                if let Some(pct) = line.strip_suffix('%') {
                    if let Ok(val) = pct.trim().parse::<f64>() {
                        state
                            .progress
                            .store((val * 100.0) as u64, Ordering::Relaxed);
                    }
                }
            }
        });

        Some(StderrCapture {
            saved_fd,
            thread: Some(thread),
        })
    }
}

impl Drop for StderrCapture {
    fn drop(&mut self) {
        unsafe {
            libc::dup2(self.saved_fd, 2);
            libc::close(self.saved_fd);
        }
        if let Some(t) = self.thread.take() {
            t.join().ok();
        }
    }
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

    let input_path = input.display().to_string();
    let output_path = output.display().to_string();

    let state = Arc::new(ProgressState {
        done: AtomicBool::new(false),
        progress: AtomicU64::new(u64::MAX),
    });

    let _capture: Option<StderrCapture> = if cli.verbose {
        None
    } else {
        StderrCapture::start(state.clone())
    };

    let process_state = state.clone();
    let process = std::thread::spawn(move || {
        let result = process_image(
            input.to_str().unwrap_or(""),
            output.to_str().unwrap_or(""),
            cli.scale,
            &model,
            cli.gpu_id,
            cli.cpu,
            cli.model_path.as_ref().map(|p| p.to_str().unwrap_or("")),
        );
        process_state.done.store(true, Ordering::Relaxed);
        result
    });

    let spinner = ['-', '\\', '|', '/'];
    let mut idx = 0;

    while !state.done.load(Ordering::Relaxed) {
        let p = state.progress.load(Ordering::Relaxed);
        if p == u64::MAX {
            print!("\r\x1b[KProcessing... {}", spinner[idx % 4]);
        } else {
            let filled = ((p as f64 / 10000.0) * 40.0) as usize;
            print!(
                "\r\x1b[K[{}{}] {}%",
                "█".repeat(filled),
                "░".repeat(40 - filled),
                p / 100
            );
        }
        std::io::stdout().flush().ok();
        idx += 1;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    print!("\r\x1b[K");
    std::io::stdout().flush().ok();

    let (success, message) = process
        .join()
        .unwrap()
        .map_err(|e| anyhow::anyhow!("Image processing failed: {}", e))?;

    if success {
        println!("==> {} -> {}", input_path, output_path);
        println!("Done! Time: {}", message);
    } else {
        eprintln!("Failed: {}", message);
        anyhow::bail!("Processing failed: {}", message);
    }

    Ok(())
}
