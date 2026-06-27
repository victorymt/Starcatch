#!/usr/bin/env bash
set -euo pipefail

# Starcatch 安装脚本
# ================
# 编译 Rust CLI 并安装到 ~/.local/bin/starcatch。
# 可选编译 TUI 和生成 shell 补全。
#
# 用法:
#   ./install.sh                          # 只装 CLI
#   INSTALL_TUI=1 ./install.sh            # CLI + TUI
#   INSTALL_COMPLETIONS=1 ./install.sh    # CLI + bash 补全
#   INSTALL_TUI=1 INSTALL_COMPLETIONS=1 ./install.sh   # 全部
#
# 前置依赖:
#   - Rust 工具链 (cargo)

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

# ── TUI (optional) ───────────────────────────────────────────────────
if [ "${INSTALL_TUI:-0}" = "1" ]; then
    echo ""
    echo "[3/3] Building TUI…"
    cargo build --release -p starcatch-tui
    cp "target/release/starcatch-tui" "${INSTALL_DIR}/starcatch-tui"
    echo "       → ${INSTALL_DIR}/starcatch-tui"
else
    echo ""
    echo "[3/3] TUI skipped (set INSTALL_TUI=1 to build it)"
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
