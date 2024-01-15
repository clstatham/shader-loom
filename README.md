# shader-loom

A simple, quick-and-dirty command line interpreter/debugger for GPU shaders.

Uses [naga](https://crates.io/crates/naga) to parse the shader source and then steps through the code sequentially.

## Cargo Features

- `wgsl`: Enables WGSL shader source input.
- `glsl`: Enables GLSL shader source input.

`wgsl` is enabled by default. Enable `glsl` by passing `--features glsl` to cargo.

## WIP

This crate is very incomplete. Use with caution!
