## sr -Image super-resolution CLI tool built with Rust and PyO3

> depends on [https://github.com/tonquer/sr-vulkan](https://github.com/tonquer/sr-vulkan)

```sh
git clone https://github.com/akirco/sr
cd sr
uv sync
cargo build -r
ln -s "$(pwd)/target/release/sr" ~/.local/bin/sr
```

```sh
sr -h
Usage: sr [OPTIONS]

Options:
  -i, --input <INPUT>
  -o, --output <OUTPUT>
  -s, --scale <SCALE>            [default: 2.0]
  -m, --model <MODEL>            [default: REALESRGAN_X4PLUS_UP4X]
      --gpu-id <GPU_ID>          [default: 0]
      --cpu
      --list-models
      --model-path <MODEL_PATH>
  -h, --help                     Print help
  -V, --version                  Print version
```
