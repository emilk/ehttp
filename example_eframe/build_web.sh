#!/bin/bash
set -eu
script_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$script_path/.."

./example_eframe/setup_web.sh

CRATE_NAME="example_eframe"

OPEN=false
OPTIMIZE=false
BUILD=debug
BUILD_FLAGS=""
WASM_OPT_FLAGS="-O2 --fast-math"

while test $# -gt 0; do
  case "$1" in
    -h|--help)
      echo "build_demo_web.sh [--release] [--open]"
      echo "  --open: open the result in a browser"
      echo "  --release: Build with --release, and then run wasm-opt."
      exit 0
      ;;

    --open)
      shift
      OPEN=true
      ;;

    --release)
      shift
      OPTIMIZE=true
      BUILD="release"
      BUILD_FLAGS="--release"
      ;;

    *)
      break
      ;;
  esac
done

# This is required to enable the web_sys clipboard API which egui_web uses
# https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Clipboard.html
# https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html
export RUSTFLAGS=--cfg=web_sys_unstable_apis

FINAL_WASM_PATH=web_demo/${CRATE_NAME}_bg.wasm

# Clear output from old stuff:
rm -f "${FINAL_WASM_PATH}"

echo "Building rust…"

cargo build \
  -p ${CRATE_NAME} \
  ${BUILD_FLAGS} \
  --lib \
  --target wasm32-unknown-unknown

echo "Generating JS bindings for wasm…"
TARGET_NAME="${CRATE_NAME}.wasm"
wasm-bindgen "target/wasm32-unknown-unknown/$BUILD/$TARGET_NAME" \
  --out-dir web_demo --no-modules --no-typescript

# to get wasm-strip:  apt/brew/dnf install wabt
# wasm-strip "${FINAL_WASM_PATH}"

if [[ "${OPTIMIZE}" = true ]]; then
  echo "Optimizing wasm…"
  # to get wasm-opt:  apt/brew/dnf install binaryen
  wasm-opt "${FINAL_WASM_PATH}" $WASM_OPT_FLAGS -o "${FINAL_WASM_PATH}"
fi

echo "Finished ${FINAL_WASM_PATH}"

if [ "${OPEN}" = true ]; then
  if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux, ex: Fedora
    xdg-open http://localhost:8787/index.html
  elif [[ "$OSTYPE" == "msys" ]]; then
    # Windows
    start http://localhost:8787/index.html
  else
    # Darwin/MacOS, or something else
    open http://localhost:8787/index.html
  fi
fi
