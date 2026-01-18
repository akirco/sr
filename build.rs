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

    if output.status.success() {
        let python_dir = String::from_utf8(output.stdout).expect("Invalid UTF-8");
        let libpython_src = format!(
            "{}/cpython-3.11.14-linux-x86_64-gnu/lib/libpython3.11.so.1.0",
            python_dir.trim()
        );
        let libpython_dst = target_dir.join("libpython3.11.so.1.0");
        let libpython_link = target_dir.join("libpython3.11.so");

        if fs::copy(&libpython_src, &libpython_dst).is_ok() {
            if !libpython_link.exists() {
                symlink(&libpython_dst, &libpython_link).ok();
            }
            println!("cargo:rustc-link-lib=python3.11");
            println!("cargo:rustc-link-search={}", target_dir.display());
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
        }
    }
}

fn symlink<P: AsRef<std::path::Path>, Q: AsRef<std::path::Path>>(
    original: P,
    link: Q,
) -> std::io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}
