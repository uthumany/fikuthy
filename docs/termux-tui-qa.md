# Termux TUI QA findings

The PTY harness was run with `TERMUX_VERSION=0.118.0` and a Termux-style `PREFIX` across 40, 60, 80, 120, 160, and 220 columns. The compact 40-column capture preserved the UTHARNESS branding, reduced the header to a short command hint, used the compact `Ask UTHARNESS…` composer, and kept status content within the terminal frame. The 60-column capture used the Termux header, displayed the `↑↓ navigate · Enter select` hint, kept the full message composer within the frame, and rendered the status bar with truncation rather than horizontal overflow.

These are Linux PTY simulations with Termux environment variables, not native Android emulator or physical-device captures. Glyph rendering, soft-keyboard behavior, Android viewport resizing, and terminal emulator-specific behavior still require device testing outside this sandbox.
