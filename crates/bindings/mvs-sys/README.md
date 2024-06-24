# MVS Bindings

## Dynamic loading

This crate supports dynamic loading of the MVS library through the `libloading` feature flag.
This is useful if you want to make your application work on system even if a suitable MVS library is not available.

```rust
let mvs = MVS::load().expect("Failed to load MVS library");

MVSError::result_from_code(mvs.MV_CC_Initialize()).expect("Failed to initialize camera sdk");

// ...

println!("Found {} devices", device_list.nDeviceNum);

MVSError::result_from_code(mvs.MV_CC_Finalize()).expect("Failed to finalize camera sdk");
```

See [examples/enum-devices.rs](../examples/enum-devices.rs) for a complete example.