# Safe MVS SDK wrapper for Rust

This crate provides a safe wrapper around the MVS SDK, a machine vision library provided by [Hikrobotics / Hikvision](https://www.hikrobotics.com/en) to interact with their cameras.  
The goal of this wrapper is to abstract the usage of the API provide a high level interface. The exposed functionality tries to make the usage intuitive and safe, but you should **read the official SDK documentation**[^1] to understand the underlying concepts.

This crates specifically targets MVS cameras, if you need a more generic support for [GenICam](https://www.emva.org/standards-technology/genicam/) devices, [`cameleon`](https://crates.io/crates/cameleon) may be worth checking out.

## Features

Crate features:
- `birb-vision` (default): implement `birb-vision` traits

## Usage

The usage of this crate begins with the creation of a [`MVSContext`], which is responsible for loading and initializing the MVS library.

From a context, you can enumerate, access and open devices ([`MVSDevice`]).

## Example

```rust no_run
use birb_vision_mvs::prelude::*;

// Initialize a context.
// You can call this function multiple times, but `MVSContext::current()`
// might be more suitable to avoid multiple initializations.
let cx = MVSContext::new(None)
    .expect("Failed to initialize a MVS context");

println!("MVS SDK version: {}", cx.sdk_version());

// Of course we need to enumerate the available devices first.
// This will give us a list of device info, which we can use to create
// a device handles.
let devices = cx
    .enumerate_devices([TransportLayerType::Usb])
    .expect("Failed to enumerate devices");

println!("Found {} MVS devices", devices.len());

for device_info in devices {
    println!("{:?}", device_info);

    let device = device_info
        .into_device(true)
        .expect("Failed to create a device handle");

    if device.open(AccessMode::Exclusive, None).is_ok() {
        println!("Device opened successfully");
    } else {
        println!("Failed to open device");
    }
}
```
See [`MVSContext::current`](crate::MVSContext::current) for a better context handling.

## Multithreading

<!-- TODO link issue -->

[`MVSContext`] is thread-safe, but [`MVSDevice`] is not!
Devices cannot be shared or sent between threads and this is enforced at compile time so you don't have to worry. You can just send the [`DeviceInfo`] and create a new device handle in the other thread:
```rust no_run
use birb_vision_mvs::prelude::*;

let devices = MVSContext::new(None)
    .expect("Failed to initialize a MVS context")
    .enumerate_devices([TransportLayerType::Usb])
    .expect("Failed to enumerate devices");

std::thread::spawn(move || {
    for device_info in devices {
        let _device = device_info
            .into_device(true)
            .unwrap();
    }
});
```

Obviously you will get a runtime error if you try to open the same device in multiple threads using [`Exclusive`] access mode, but no undefined behavior will occur.

## Versioning Policy <!-- scheme -->

When a new version of this crate is released, the **major** version will match the required version of the MVS SDK. The **minor** version will be incremented for new features, and the **patch** version will be incremented for bug fixes.

## Notes

> This is currently not an official Hikvision product, nor is it endorsed by Hikvision. I hope that they will approve this project and provide first party support for this crate in the future.

> The Hikrobot MVS SDK is not redistributed by this crate. Users must install the official SDK separately and comply with its license terms. See the repository `THIRD_PARTY_NOTICES.md` for details about the generated `mvs-sys` bindings.

> ⚠️**Warning**⚠️: Version `~4` of the MVS SDK [^1] is required, use `mvs-sys` directly for a more generic support.

> ⚠️**Warning**⚠️: **DO NOT USE MULTIPLE VERSIONS OF THIS CRATE** in your binaries. Doing so might initialize and finalyze the MVS SDK multiple times, which may result in **unexpected behavior**. [`cargo tree`](https://doc.rust-lang.org/cargo/commands/cargo-tree.html) might turn useful for you.  
> See issue [^2] for more info.

<!-- References: -->
[`MVSContext`]: crate::MVSContext
[`MVSDevice`]: crate::MVSDevice
[`DeviceInfo`]: crate::device::DeviceInfo
[`Exclusive`]: crate::device::AccessMode::Exclusive

<!-- Footnotes: -->
[^1]: <https://www.hikrobotics.com/en/machinevision/service/download>
[^2]: TODO link issue
