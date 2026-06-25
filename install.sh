#!/usr/bin/env bash
set -euo pipefail

# Starcatch 安装脚本
# ================
# 编译 Rust CLI 并安装到 ~/.local/bin/starcatch。
# 可选编译 Qt 6 GUI 和生成 shell 补全。
#
# 用法:
#   ./install.sh                          # 只装 CLI
#   INSTALL_GUI=1 ./install.sh            # CLI + Qt GUI
#   INSTALL_COMPLETIONS=1 ./install.sh    # CLI + bash 补全
#   INSTALL_GUI=1 INSTALL_COMPLETIONS=1 ./install.sh   # 全部
#
# 前置依赖:
#   - Rust 工具链 (cargo)
#   - Qt 6 (可选，仅当 INSTALL_GUI=1 时需要 cmake + Qt6 库)

INSTALL_DIR="${HOME}/.local/bin"
DATA_DIR="${HOME}/.local/share/starcatch"
BIN_NAME="starcatch"

echo "=== Starcatch Install ==="

# ── Rust toolchain check ─────────────────────────────────────────────
if ! command -v cargo &>/dev/null; then
    echo "ERROR: cargo not found. Install Rust from https://rustup.rs"
    exit 1
fi

# ── Build & install CLI ──────────────────────────────────────────────
echo ""
echo "[1/3] Building Rust CLI (release)…"
cargo build --release

echo "[2/3] Installing binary…"
mkdir -p "${INSTALL_DIR}"
cp "target/release/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
echo "       → ${INSTALL_DIR}/${BIN_NAME}"

# ── Create data directory ────────────────────────────────────────────
mkdir -p "${DATA_DIR}"
echo "       → ${DATA_DIR} (data directory)"

# ── PATH reminder ────────────────────────────────────────────────────
if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo ""
    echo "[!] ${INSTALL_DIR} is not in your PATH."
    echo "    Add this to your ~/.bashrc or ~/.zshrc:"
    echo ""
    echo "    export PATH=\"${INSTALL_DIR}:\$PATH\""
fi

# ── Qt GUI (optional) ────────────────────────────────────────────────
if [ "${INSTALL_GUI:-0}" = "1" ]; then
    echo ""
    echo "[3/3] Building Qt GUI…"
    cd "$(dirname "$0")/qt"
    cmake -B build
    cmake --build build
    echo "       → qt/build/starcatch-qt"
else
    echo ""
    echo "[3/3] Qt GUI skipped (set INSTALL_GUI=1 to build it)"
fi

# ── Shell completions (optional) ─────────────────────────────────────
if [ "${INSTALL_COMPLETIONS:-0}" = "1" ]; then
    echo ""
    echo "[+] Generating shell completions…"
    COMP_DIR="${HOME}/.local/share/bash-completion/completions"
    mkdir -p "${COMP_DIR}"
    "${INSTALL_DIR}/${BIN_NAME}" completions bash > "${COMP_DIR}/${BIN_NAME}" 2>/dev/null || true
    echo "       → ${COMP_DIR}/${BIN_NAME} (bash)"
fi

echo ""
echo "Done. Run '${BIN_NAME} --help' to verify."
