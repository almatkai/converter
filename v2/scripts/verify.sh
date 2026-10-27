#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
export HOME="$PWD/.qa/home" TMPDIR="$PWD/.qa/tmp"
mkdir -p "$HOME" "$TMPDIR" .qa/v1 .qa/schema-tools
python3 - <<'PY'
from pathlib import Path
import shutil
for source,destination in [('qa/fixtures/original','.qa/v1'),('qa/toolchain','.qa/schema-tools')]:
 for f in Path(source).rglob('*'):
  if f.is_file():
   target=Path(destination)/f.relative_to(source);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(f,target)
PY
npm ci --ignore-scripts --prefix .qa/v1 --cache "$PWD/.qa/npm-cache"
npm ci --ignore-scripts --prefix .qa/schema-tools --cache "$PWD/.qa/npm-cache"
npm --prefix .qa/v1 test
npm --prefix .qa/v1 run typecheck
node scripts/oracle.cjs
node scripts/ui-oracle.cjs
python3 scripts/host-patch.py
python3 scripts/host-fixtures.py
# rustup toolchains are preinstalled prerequisites; this script installs none.
# Preserve rustup's configured home after replacing HOME for QA processes.
export RUSTUP_HOME="${RUSTUP_HOME:?set RUSTUP_HOME to the installed toolchain directory}"
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo +1.85.0 test --locked
cargo +1.85.0 clippy --locked --all-targets -- -D warnings
cargo build --locked
python3 scripts/process-qa.py
node scripts/host-qa.cjs
python3 scripts/notices.py
CARGO_HOME="$PWD/.qa/fresh-cargo-home" CARGO_TARGET_DIR="$PWD/.qa/fresh-target" cargo build --release --locked
python3 scripts/package.py
python3 scripts/audit.py
