# UmaPilot — Milestone 01

Windows desktop utility for selecting a game window, capturing its client area, finding PNG templates, and performing one explicitly confirmed click. It is intentionally local-only: no injection, hooks, workflows, or automatic repetition.

## Build and run (Windows 10/11 x64)

Install [Rust stable with the MSVC toolchain](https://rustup.rs/) and the Visual Studio **Desktop development with C++** workload, then run:

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
.\target\release\uma-pilot.exe
```

Put `.png` templates under `assets/templates`, then select the game window, capture it, choose a template, and press **Find Template**. A result is accepted only at or above the configured threshold (default 0.85). The preview draws the accepted result in green. **Open Screenshot PNG** supports offline matching without a running game; offline images can never be clicked.

## Safe click workflow

`Test Click` defaults to dry-run and reports the exact screen coordinates without emitting input. To send a single click, turn on **Real Click**, tick **I confirm one real click**, and press the button. Immediately before either action, UmaPilot validates the target window and uses the latest accepted match. It clicks the bounding-box centre once through `SendInput`; it never loops.

## Capture behaviour

Capture is of the selected window's client area, DPI-aware and independent of a fixed game resolution. It uses `PrintWindow` and falls back to a window DC when necessary. A nearly black result is rejected as a capture failure instead of being saved or presented as success. Minimized/disappeared windows are rejected.

## Tests

The vision module has unit tests for an embedded match and a non-match. On non-Windows hosts the application UI can compile only when platform dependencies support it, but discovery, capture, and input deliberately report `Unsupported`; validate the release build on Windows.
