#!/usr/bin/env bash
set -e
SRC="$(cd "$(dirname "$0")/.." && pwd)"
UV_PY="$HOME/.local/share/uv/python/cpython-3.11-linux-x86_64-gnu"
OUT="/tmp/sr-portable"

rm -rf "$OUT"
mkdir -p "$OUT/lib"

cp "$SRC/target/release/sr" "$OUT/"
cp "$SRC/target/release/libpython3.11.so.1.0" "$OUT/lib/"
cp -a "$SRC/target/release/libpython3.11.so" "$OUT/lib/"
cp -r "$UV_PY/lib/python3.11" "$OUT/lib/"
cp -r "$SRC/.venv/lib/python3.11/site-packages/sr_vulkan"* "$OUT/lib/python3.11/site-packages/"

rm -rf "$OUT/lib/python3.11"/{test,tkinter,idlelib,ensurepip,lib2to3,distutils,pydoc_data,venv}
rm -rf "$OUT/lib/python3.11/site-packages"/{pip,pip-*,setuptools*,_distutils_hack,distutils-precedence.pth}
rm -f "$OUT/lib/python3.11/site-packages"/{_virtualenv.pth,_virtualenv.py}
find "$OUT" -name "__pycache__" -type d -exec rm -rf {} + 2>/dev/null
find "$OUT" -name "*.pyc" -delete 2>/dev/null

cat > "$OUT/run.sh" << 'RUNEOF'
#!/usr/bin/env bash
DIR="$(cd "$(dirname "$0")" && pwd)"
export LD_LIBRARY_PATH="$DIR/lib:$LD_LIBRARY_PATH"
export PYTHONHOME="$DIR"
exec "$DIR/sr" "$@"
RUNEOF
chmod +x "$OUT/run.sh"

tar czf "$SRC/sr-portable.tar.gz" -C "$OUT" .
echo "Done: $(du -sh "$SRC/sr-portable.tar.gz")"
