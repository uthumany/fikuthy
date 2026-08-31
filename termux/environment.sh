#!/data/data/com.termux/files/usr/bin/sh
# UTHARNESS Termux path contract. This file is sourced by package scripts and
# can be sourced by users who want to inspect the resolved directories.
export UTHARNESS_TERMUX=1
export UTHARNESS_PREFIX="${PREFIX:-/data/data/com.termux/files/usr}"
export UTHARNESS_BIN_DIR="$UTHARNESS_PREFIX/bin"
export UTHARNESS_LIB_DIR="$UTHARNESS_PREFIX/lib/utharness"
export UTHARNESS_SHARE_DIR="$UTHARNESS_PREFIX/share/utharness"
export UTHARNESS_CONFIG_DIR="${HOME:-.}/.config/utharness"
export UTHARNESS_DATA_DIR="${HOME:-.}/.local/share/utharness"
export UTHARNESS_CACHE_DIR="${HOME:-.}/.cache/utharness"
export UTHARNESS_HOME="$UTHARNESS_DATA_DIR"
