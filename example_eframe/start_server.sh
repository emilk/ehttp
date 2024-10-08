#!/bin/bash
set -eu
script_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$script_path/.."

# Starts a local web-server that serves the contents of the `doc/` folder,

echo "ensuring basic-http-server is installed…"
cargo install basic-http-server

echo "starting server…"
echo "serving at http://localhost:8787"

(cd web_demo && basic-http-server --addr 127.0.0.1:8787 .)
# (cd web_demo && python3 -m http.server 8787 --bind 127.0.0.1)
