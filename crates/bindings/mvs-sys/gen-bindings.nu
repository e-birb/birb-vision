# Generate bindings for the MVS SDK
#
# Note:
# - run `cargo [b]install bindgen-cli`


$env.FILE_PWD

if ($env.OS | find Windows) == null {
    panic $"This script is for Windows only as the Linux version of the SDK comes with no C headers \(just the runtime library\). Current OS: ($env.OS)"
}

if $env.MVCAM_COMMON_RUNENV == null {
    panic "MVCAM_COMMON_RUNENV environment variable is not set, maybe the MVS SDK is not installed?"
}

bindgen ...[
    $"($env.FILE_PWD)/ffi/mvs.h"

    # options
    --allowlist-item "MV_.*"
    --dynamic-loading MVS # called "dynamic_library_name" in bindgen::Builder
    --dynamic-link-require-all

    # clang args
    --
    -x c++
    -target x86_64-pc-windows-msvc
    -I $"($env.MVCAM_COMMON_RUNENV)/Includes"
] | save -f $"($env.FILE_PWD)/src/mvs.rs"