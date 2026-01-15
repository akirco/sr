use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().nth(3).unwrap(); // 找到 target/debug 或 target/release

    fs::copy(
        "/home/neil/.local/share/uv/python/cpython-3.11.14-linux-x86_64-gnu/lib/libpython3.11.so.1.0",
        target_dir.join("libpython3.11.so.1.0"),
    )
    .ok();

    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
}
