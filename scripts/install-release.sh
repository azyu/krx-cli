#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
install_root="${HOME}/.local"
install_bin_dir="${install_root}/bin"

cargo install \
  --path "$repo_root/crates/cli" \
  --root "$install_root" \
  --force \
  --bin krx

mkdir -p "$install_bin_dir"
printf 'installed %s\n' "${install_bin_dir}/krx"
