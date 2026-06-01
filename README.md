# birb-vision
 A comprehensive Rust library designed for machine vision applications.

`birb-vision` provides a unified interface to interact with various camera systems such as webcams and industrial cameras (including MVS, iCube, Daheng and more). The library aims to simplify the process of camera enumeration, control, and image acquisition, to build robust vision-based applications.

## Usage

```sh
cargo run --example all_cameras --features v4l
```
or, on Windows:
```sh
cargo run --example all_cameras --features media-foundation,directshow
```

To test the windows implementations on Linux, you can use Wine:
```sh
cargo build \
  --example all_cameras \
  --features media-foundation,directshow \
  --target x86_64-pc-windows-gnu
wine target/x86_64-pc-windows-gnu/debug/examples/all_cameras.exe
```
For 32-bit Windows you can use the `i686-pc-windows-gnu` target instead.

> [!NOTE]
> Note that Wine does not currently support DirectShow camera capture so it
> will not show any cameras.

## Crates

See #7

(NOTE: not upd to date)

- [`birb-vision`](./crates/birb-vision/): the core crate
- **interfaces**: some provided interfaces
  - [`birb-vision-fake`](./crates/birb-vision-icube/): fake cameras for testing
  - [`birb-vision-icube`](./crates/birb-vision-icube/): the interface for the iCube cameras
  - [`birb-vision-mvs`](./crates/birb-vision-mvs/): the interface for the MVS cameras
  - [`birb-vision-nokhwa`](./crates/birb-vision-nokhwa/): the interface for the `nokhwa` crate
- [`birb-vision-bundle`](./crates/birb-vision-bundle/): wraps all the interfaces into a single crate
- [`bindings/`](./crates/bindings/): bindings for some camera interfaces
  - [`mvs-sys`](./crates/bindings/mvs-sys): sys crate for the MVS SDK