#!/bin/bash
set -eu
script_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
cd "$script_path/.."

cargo doc -p ehttp --lib --no-deps --all-features --open

# cargo watch -c -x 'doc -p ehttp --lib --no-deps --all-features'
