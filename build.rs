use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).unwrap();
    let output = Command::new("uv")
        .args(["python", "dir"])
        .output()
        .expect("Failed to execute command");
    let python_dir;
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).expect("Output not valid UTF-8");

        python_dir = format!(
            "{}/cpython-3.11.14-linux-x86_64-gnu/lib/libpython3.11.so.1.0",
            stdout.trim()
        );
        fs::copy(python_dir, target_dir.join("libpython3.11.so.1.0")).ok();

        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("错误详情: {}", stderr);
    }
}
