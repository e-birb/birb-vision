# Daheng Galaxy SDK Bindings

Raw Rust FFI bindings for the Daheng Galaxy SDK.

This crate contains Daheng Galaxy SDK C headers under `ffi/`. They are included
with Daheng's permission for this project. The SDK libraries are not included;
users must install the Daheng Galaxy SDK separately and comply with its license
terms.

The crate generates Rust bindings at build time using `bindgen` and dynamically
loads the vendor library at runtime.

See the repository `THIRD_PARTY_NOTICES.md` for the full third-party notice.

## Dynamic loading

The crate tries to load the platform-specific Daheng Galaxy SDK library, then
selects the supported API version.
