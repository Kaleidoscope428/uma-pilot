# UmaPilot Milestone 01 — Verification Report

Date: 2026-11-24  
Workspace: `E:\A.PRJ\UmaPilot`

## 1. Build environment

- OS: Windows x64.
- Rust: `rustc 1.100.0 (0ef1bc386 2026-11-09)`.
- Cargo: `cargo 1.100.0 (3acc79c0c 2026-11-05)`.
- rustfmt: `1.11.0-stable`; Clippy: `0.1.100`.
- Toolchain: `stable-x86_64-pc-windows-msvc`.
- Linker: Visual Studio 2022 Build Tools, MSVC `14.44.35207`, `Hostx64\x64\link.exe`.
- Windows SDK: `10.0.26100.0`.

## 2. Defects found and fixed

- Repaired invalid Rust float literals (`.99`, `.85`) that prevented parsing.
- Corrected `windows 0.58` API imports: `ClientToScreen`, `SetProcessDPIAware`, and `PrintWindow`/`PW_CLIENTONLY` are exposed from different generated modules than originally used.
- Corrected `HWND` conversion to/from raw pointers and GDI null-handle checks.
- Added missing TOML type inference, eframe `egui` path, and supported egui colour constant.
- Removed an unused dependency and resolved all `-D warnings` Clippy findings.
- Hardened click flow: a Test Click now captures again, matches again, verifies client-area bounds, validates the window, and checks foreground status before `SendInput`. The one-click confirmation is consumed before worker dispatch.

## 3. Executed commands and results

All commands ran from a Visual Studio x64 environment, initialised through `vcvars64.bat`.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --all-targets -- -D warnings` | Passed |
| `cargo test` | Passed: 2 passed, 0 failed |
| `cargo build --release` | Passed |
| `scripts\verify.ps1 -Release` | Passed |

## 4. Automated coverage

- `vision::tests::finds_embedded_template`: verifies exact template location and confidence matching.
- `vision::tests::rejects_different_image`: verifies a non-matching image is rejected at the threshold.
- The release EXE passed a startup smoke test: it remained running for three seconds before the verifier intentionally stopped it.

## 5. Distribution output

- EXE: `E:\A.PRJ\UmaPilot\dist\UmaPilot.exe`
- Configuration: `E:\A.PRJ\UmaPilot\dist\config\default.toml`
- Template folder: `E:\A.PRJ\UmaPilot\dist\assets\templates`

## 6. Windows GUI functions verified

- Native x64 release build and GUI process startup were verified on Windows.
- The code path for window enumeration, client-coordinate conversion, GDI bitmap conversion, template matching, dry-run, foreground validation, and `SendInput` compiled with the Windows/MSVC bindings.
- No real input was sent during verification.

## 7. Not verified and why

- Uma Musume was not available; no claim is made that its game window was captured or clicked.
- `assets\templates` contained only `.gitkeep`, so no original PNG template load, real screenshot match, false-positive measurement, scale-mismatch corpus test, or timing benchmark could be executed.
- No DirectX/Unity test application was available. `PrintWindow` is therefore not proven to capture Unity/DX content on this machine. Black frames are rejected and block input; Windows Graphics Capture is not implemented in this milestone.
- A physical real click was deliberately not issued. The code requires an explicit user confirmation and foreground target before it can do so.

## 8. Remaining risks before Milestone 02

- `PrintWindow` can fail or return black for GPU/protected windows; Windows Graphics Capture is the appropriate future fallback.
- Normalized correlation is exhaustive and may be slow for large screenshots/templates. It is dispatched off the UI thread, but needs profiling against real templates.
- Matching is not scale invariant; templates must match the captured UI scale.
- The application only permits a real click when the selected window is foreground, which is safe but requires manual focus from the user.
