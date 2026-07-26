#!/bin/sh
set -eu

PROJECT_NAME="falcon-system-utils"
PROJECT_DIR="$(mktemp -d)/${PROJECT_NAME}"
BIN_DIR="${HOME}/.local/bin"

git clone "https://github.com/hansolo1000falcon/${PROJECT_NAME}.git" "${PROJECT_DIR}"
cargo build --release --manifest-path "${PROJECT_DIR}/Cargo.toml"

mkdir -p "${BIN_DIR}"
cp "${PROJECT_DIR}/target/release/${PROJECT_NAME}" "${BIN_DIR}/${PROJECT_NAME}"
mv "${BIN_DIR}/${PROJECT_NAME}" "${BIN_DIR}/fsysutils"
rm -rf "${PROJECT_DIR}"

echo "Installed fsysutils to ${BIN_DIR}"
