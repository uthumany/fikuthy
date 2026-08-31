# Visual QA findings

Date: 2026-08-27

## Compact 40x18

The compact TUI keeps the UTHARNESS identity, focus-mode header, prompt, and bottom status visible. Content is reduced to short rows and the composer remains usable. The compact screenshot shows deliberate truncation of long secondary text rather than horizontal overflow. The top banner is rendered as compact text rather than the full large logo.

## Wide 120x36

The wide TUI displays the full colored UTHY banner, onboarding tips, conversation rows, timestamps, inline tool cards, cyan composer border, and telemetry status line. Header, banner, composer, and status remain fixed while the conversation occupies the middle region. No clipping or horizontal overflow was observed in the inspected screenshot.

## Scope limitation

These images are Linux PTY simulations. They are evidence for Ink layout behavior at the tested dimensions, not proof of behavior in every terminal emulator, Android device, iOS terminal, SSH transport, or accessibility configuration.
