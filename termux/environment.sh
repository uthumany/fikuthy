#!/data/data/com.termux/files/usr/bin/sh
# FIKUTHY Termux path contract. This file is sourced by package scripts and
# can be sourced by users who want to inspect the resolved directories.
export FIKUTHY_TERMUX=1
export FIKUTHY_PREFIX="${PREFIX:-/data/data/com.termux/files/usr}"
export FIKUTHY_BIN_DIR="$FIKUTHY_PREFIX/bin"
export FIKUTHY_LIB_DIR="$FIKUTHY_PREFIX/lib/fikuthy"
export FIKUTHY_SHARE_DIR="$FIKUTHY_PREFIX/share/fikuthy"
export FIKUTHY_CONFIG_DIR="${HOME:-.}/.config/fikuthy"
export FIKUTHY_DATA_DIR="${HOME:-.}/.local/share/fikuthy"
export FIKUTHY_CACHE_DIR="${HOME:-.}/.cache/fikuthy"
export FIKUTHY_HOME="$FIKUTHY_DATA_DIR"
