# birb-vision [![Rust](https://github.com/e-birb/birb-vision/actions/workflows/rust.yml/badge.svg)](https://github.com/e-birb/birb-vision/actions/workflows/rust.yml)

<div align="center">
  <h3>A unified Rust interface for machine vision cameras</h3>
  <p>
    <em>One API to enumerate, control, and capture from any camera (webcam or industrial).</em>
  </p>
</div>

> [!IMPORTANT]
> This project is a work in progress. The core API is unstable, some backends are experimental or temporarily broken. It should not be treated as production-ready yet.

`birb-vision` provides a single, coherent API to interact with a wide range of camera systems: from USB webcams to GigE Vision industrial cameras. It abstracts away vendor-specific SDKs behind a common trait-based interface, making it easy to write portable vision applications.

## Philosophy

- **The `CameraDevice` trait** provides a consistent interface for enumeration, property control, and frame acquisition.
- **Preserve backend expressiveness**: core types are designed to be the common denominator without preventing backends from exposing richer capabilities through their own types.
- **Zero unnecessary copies**: frame data is delivered as borrowed buffers or `Arc`-locked memory, letting backends avoid copies where the backend allows it.

## Example

See [`basic.rs`](./crates/birb-vision/examples/basic.rs):
```rust
for (_, pkg) in all_backends().all_packages() {
  let ctx = pkg.build_backend()?;
  for info in ctx.enumerate(&ctx.default_transport_layers())? {
    let device = ctx.create(&info)?.unwrap();
    device.start_grabbing()?;
    let frame = device.get_one_frame(timeout).await?;
    frame.try_decode().unwrap()?.save("frame.png")?;
  }
}
```
which you can run it with:
```sh
cargo run --example basic --features "v4l"
```
or, on Windows:
```sh
cargo run --example basic --features "media-foundation,directshow"
```

### Full example

```sh
cargo run --example all_cameras --features v4l
```
or, on Windows:
```sh
cargo run --example all_cameras --features "media-foundation,directshow"
```

## Project Structure

The project is organized as a Cargo workspace with the following crates:

| Crate | Description | Status |
|---|---|---|
| [`birb-vision-core`](./crates/birb-vision-core/) | Core traits and types | ✅ Maintained |
| [`birb-vision`](./crates/birb-vision/) | Aggregator crate with `BackendRegistry`. Registers and discovers available backends via feature flags | ✅ Maintained |
| **Linux backends** | | |
| [`birb-vision-v4l`](./crates/birb-vision-v4l/) | Video4Linux2 backend (webcams, capture cards on Linux) | ✅ Maintained |
| **Windows backends** | | |
| [`birb-vision-directshow`](./crates/birb-vision-directshow/) | DirectShow backend (webcams, USB cameras on Windows) | ✅ Maintained |
| [`birb-vision-media-foundation`](./crates/birb-vision-media-foundation/) | Media Foundation backend (modern Windows capture API) | ✅ Maintained |
| **Industrial camera backends** | | |
| [`birb-vision-mvs`](./crates/birb-vision-mvs/) | Hikrobot MVS SDK backend (Hikrobot / MV series industrial cameras) | ✅ Maintained |
| [`birb-vision-daheng`](./crates/birb-vision-daheng/) | Daheng (MER/Galaxy) SDK backend (Daheng industrial cameras) | 🚧 Maintained, not working |
| [`birb-vision-icube`](./crates/birb-vision-icube/) | iCube SDK backend (iCube smart cameras) | ✅ Maintained |
| **Utility / experimental** | | |
| [`birb-vision-fake`](./crates/birb-vision-fake/) | Fake/mock camera backend for testing | ⚠️ Stub (not yet a real backend) |
| [`birb-vision-nest`](./crates/birb-vision-nest/) | Plugin/dynamic-loading system for external camera backends | ⚠️ Stale (temporarily broken) |
| [`birb-vision-nokhwa`](./crates/birb-vision-nokhwa/) | Wrapper around the `nokhwa` cross-platform camera library | ⚠️ Stale (uses outdated API) |
| [`birb-vision-media-uvc`](./crates/birb-vision-media-uvc/) | UVC (USB Video Class) backend | ⚠️ Stub |
| **FFI bindings** | | |
| [`mvs-sys`](./crates/bindings/mvs-sys/) | Raw FFI bindings for the Hikrobot MVS SDK | ✅ Maintained |
| [`daheng-sys`](./crates/bindings/daheng-sys/) | Raw FFI bindings for the Daheng Galaxy SDK (V1 & V2 APIs) | ✅ Maintained |

## Feature Flags

The aggregator crate [`birb-vision`](./crates/birb-vision/) uses Cargo feature flags to control which backends are compiled:

| Feature | Backend | Platform |
|---|---|---|
| `v4l` | Video4Linux (V4L2) | Linux |
| `directshow` | DirectShow | Windows |
| `media-foundation` | Media Foundation | Windows |
| `mvs` | Hikrobot MVS SDK | Windows / Linux (x86_64) |
| `daheng` | Daheng Galaxy SDK | Windows / Linux (x86_64) |
| `icube` | iCube SDK | Linux |

```sh
# Linux: use V4L2 for webcams + MVS for industrial cameras
cargo run --example all_cameras --features "v4l,mvs"

# Windows: DirectShow + Media Foundation + MVS
cargo run --example all_cameras --features "directshow,media-foundation,mvs"
```

## Architecture

- `birb-vision-core: Core traits & types (no platform deps)
  - `CameraDevice` trait: open, close, start/stop grabbing, read/write properties, set stream callback
  - `VisionContext` trait: enumerate devices, create device instances
  - `Node` / `Property` / `NodeId`: GenICam-inspired property tree
  - `Sample` / `PixelFormat`: Frame data types & format handling
- `birb-vision` (aggregator): Feature gating, `BackendRegistry`
  - `all_backends()` returns a registry of all compiled-in backends
- `birb-vision-*` (backends): Platform/SDK-specific implementations
  - Each implements `CameraDevice` + `VisionContext`
  - Registered via `birb-vision` feature flags

## Platform Support

| Platform | Backends |
|---|---|
| Linux | V4L2, MVS, Daheng, iCube |
| Windows | DirectShow, Media Foundation, MVS, Daheng |
| Cross-compile (Linux -> Windows) | DirectShow, Media Foundation (via `x86_64-pc-windows-gnu`) |

> [!NOTE]
> Wine does not currently support MediaFoundation camera capture and enumeration will result in an empty device list. DirectShow may work for some webcams but is not fully tested under Wine.

## Project Status

birb-vision is in **active development**. The core trait design is **unstable**, most backends implement the full API surface.

## License

This repository is licensed under the Apache License, Version 2.0. See
[`LICENSE`](./LICENSE).

Some crates expose or include vendor SDK interface material for interoperability
with industrial camera SDKs. Those SDKs are not redistributed here and must be
installed separately. See [`THIRD_PARTY_NOTICES.md`](./THIRD_PARTY_NOTICES.md)
for Hikrobot MVS and Daheng Galaxy SDK notes.
