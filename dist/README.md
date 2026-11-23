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

Or run all automated checks in a configured Windows shell:

```powershell
.\scripts\verify.ps1 -Release
```

The verified portable layout is under `dist/`. Start `dist\UmaPilot.exe` with `dist` as the working directory so it can find `config\default.toml` and `assets\templates`.

Put `.png` templates under `assets/templates`, then select the game window, capture it, choose a template, and press **Find Template**. A result is accepted only at or above the configured threshold (default 0.85). The preview draws the accepted result in green. **Open Screenshot PNG** supports offline matching without a running game; offline images can never be clicked.

## Safe click workflow

`Test Click` defaults to dry-run and reports the exact screen coordinates without emitting input. To send a single click, turn on **Real Click**, tick **I confirm one real click**, and press the button. Immediately before either action, UmaPilot captures the selected client area again, runs matching again, validates the match bounds and target window, then requires that target to be foreground. It clicks the bounding-box centre once through `SendInput`; it never loops.

## Capture behaviour

Capture is of the selected window's client area, DPI-aware and independent of a fixed game resolution. It uses `PrintWindow` and falls back to a window DC when necessary. A nearly black result is rejected as a capture failure instead of being saved or presented as success. Minimized/disappeared windows are rejected. Some DirectX/Unity applications may return an empty or black `PrintWindow` frame; UmaPilot reports that failure and does not click. Windows Graphics Capture has not yet been enabled as a fallback in this milestone.

## Tests

The vision module has unit tests for an embedded match and a non-match. `scripts/verify.ps1 -Release` executes formatting, Clippy, tests, and a release build in a Visual Studio x64 environment.

Template matching is position-based and does not scale templates. Capture a template at the game UI scale being tested; a DPI/resolution change can otherwise produce a valid `NOT FOUND` result rather than a click.

## Common problems

- **`link.exe` reports “extra operand”**: start from a Visual Studio Developer PowerShell, or use `scripts/verify.ps1`; it initializes `vcvars64.bat` before Cargo.
- **Capture fails with a black frame**: the target may be minimized, protected, or not renderable through `PrintWindow`. Restore it and keep it visible; the application deliberately refuses to treat black frames as success.
- **Real Click is blocked**: select a window/template, put the target window in the foreground, then explicitly confirm the single click. Offline PNGs and stale matches are never used to click.
