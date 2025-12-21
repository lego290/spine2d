# Changelog

This project follows a pragmatic changelog format during early development.
Version numbers follow SemVer, but the public API is expected to change rapidly until `1.0`.

## Unreleased

- TBD

## 0.1.0

Initial experimental release.

Highlights:
- Pure Rust Spine 4.3 runtime core (`spine2d`) with JSON parsing and renderer-agnostic draw output.
- Native wgpu integration crate (`spine2d-wgpu`) with a runnable viewer example.
- wasm32 demo crate (`spine2d-web`, not published) for `wasm32-unknown-unknown` validation.
- Oracle-driven parity workflow against upstream `spine-runtimes` (pinned by commit) to avoid “approximate” behaviour.

