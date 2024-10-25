#pragma once

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

#define BIRB_VISION_NEST_INTERFACE_VERSION "0.1.0"

// ================================================================
//                         DOCUMENTATION
// ================================================================

/*

# Introduction

This header defines the interface to be implemented by the (external) backends
of the Birb Vision Nest library.

# Implementation Guide

The implementor shall export all the functions defined in this header and
satisfy the requirements of each function as defined in the respective
documentation.

*/

// ================================================================
//                          TYPE DEFS
// ================================================================

typedef uint32_t PixelType;

// ================================
//              Basic
// ================================

static const PixelType PIXEL_TYPE_UNKNOWN        = 0x00000000;
static const PixelType PIXEL_TYPE_PACKED_FLAG    = 0x80000000;
static const PixelType PIXEL_TYPE_SIGNED_FLAG    = 0x40000000;

// ================================
//       Uncompressed formats
// ================================ 

// ================
//       Mono
// ================

static const PixelType PIXEL_TYPE_MONO_UNKNOWN   = 0x00000100;
static const PixelType PIXEL_TYPE_MONO_1         = 0x00000001 | PIXEL_TYPE_MONO_UNKNOWN;
static const PixelType PIXEL_TYPE_MONO_2         = 0x00000002 | PIXEL_TYPE_MONO_UNKNOWN;
static const PixelType PIXEL_TYPE_MONO_4         = 0x00000004 | PIXEL_TYPE_MONO_UNKNOWN;
static const PixelType PIXEL_TYPE_MONO_6         = 0x00000006 | PIXEL_TYPE_MONO_UNKNOWN;
static const PixelType PIXEL_TYPE_MONO_8         = 0x00000008 | PIXEL_TYPE_MONO_UNKNOWN;
static const PixelType PIXEL_TYPE_MONO_10        = 0x0000000A | PIXEL_TYPE_MONO_UNKNOWN;
static const PixelType PIXEL_TYPE_MONO_12        = 0x0000000C | PIXEL_TYPE_MONO_UNKNOWN;
static const PixelType PIXEL_TYPE_MONO_16        = 0x00000010 | PIXEL_TYPE_MONO_UNKNOWN;
static const PixelType PIXEL_TYPE_MONO_32        = 0x00000020 | PIXEL_TYPE_MONO_UNKNOWN;

// ================
//       RGB
// ================

static const PixelType PIXEL_TYPE_RGB_UNKNOWN    = 0x00000200;
static const PixelType PIXEL_TYPE_RGB_888        = 0x00000001 | PIXEL_TYPE_RGB_UNKNOWN;
static const PixelType PIXEL_TYPE_BGR_UNKNOWN    = 0x00000300;
static const PixelType PIXEL_TYPE_BGR_888        = 0x00000001 | PIXEL_TYPE_BGR_UNKNOWN;
// ...
static const PixelType PIXEL_TYPE_RGBA_UNKNOWN   = 0x00000400;
static const PixelType PIXEL_TYPE_RGBA_8888      = 0x00000001 | PIXEL_TYPE_RGBA_UNKNOWN;
// ...
static const PixelType PIXEL_TYPE_ARGB_UNKNOWN   = 0x00000500;
static const PixelType PIXEL_TYPE_ARGB_8888      = 0x00000001 | PIXEL_TYPE_ARGB_UNKNOWN;

// ================
//       YUV
// ================

// static const PixelType PIXEL_TYPE_YUV_UNKNOWN    = 0x00040000;
// static const PixelType PIXEL_TYPE_YUV_422        = 0x00040001;
// static const PixelType PIXEL_TYPE_YUV_444        = 0x00040002;
// static const PixelType PIXEL_TYPE_YUV_411        = 0x00040003;
// static const PixelType PIXEL_TYPE_YUV_420        = 0x00040004;
// static const PixelType PIXEL_TYPE_YUV_400        = 0x00040005;
// static const PixelType PIXEL_TYPE_YUV_422P       = 0x00040006;
// static const PixelType PIXEL_TYPE_YUV_420P       = 0x00040007;
// static const PixelType PIXEL_TYPE_YUV_444P       = 0x00040008;
// static const PixelType PIXEL_TYPE_YUV_411P       = 0x00040009;
// static const PixelType PIXEL_TYPE_YUV_420SP      = 0x0004000A;
// static const PixelType PIXEL_TYPE_YUV_422SP      = 0x0004000B;
// static const PixelType PIXEL_TYPE_YUV_400P       = 0x0004000C;
// static const PixelType PIXEL_TYPE_YUV_420_8      = 0x0004000D;
// static const PixelType PIXEL_TYPE_YUV_422_8      = 0x0004000E;
// static const PixelType PIXEL_TYPE_YUV_444_8      = 0x0004000F;
// static const PixelType PIXEL_TYPE_YUV_420_10     = 0x00040010;
// // ...

// ================================
//  COMPRESSED / ENCODED / STREAM
// ================================
// TODO choose an option above

static const PixelType PIXEL_TYPE_JPEG_UNKNOWN   = 0x00080000;
static const PixelType PIXEL_TYPE_MJPEG          = 0x00000001 | PIXEL_TYPE_JPEG_UNKNOWN;


// ================================
//              ...
// ================================

struct FlatSampleLayout {
    /// Pixel type of the sample.
    PixelType pixel_type;
    /// Offset of the first row/column
    uint32_t offset;

    /// Width of the sample in pixels.
    uint32_t width;

    /// Height of the sample in pixels.
    uint32_t height;

    /// Stride of the sample in bytes.
    ///
    /// If the stride is negative, the image is flipped.
    int32_t stride;

    /// Whether the sample is stored in row-major order.
    bool row_major;
};

/// A flat representation of a sample.
struct FlatSample {
    /// Pointer to the sample data.
    ///
    /// The data is owned by the caller and must not be freed.
    uint8_t* data;

    /// Size of the sample data in bytes.
    uint64_t data_size;

    /// FourCC code of the sample.
    ///
    /// If not set, the `pixel_type` in `layout` should be used to determine the format.
    char FourCC[4];

    /// Layout of the sample data.
    struct FlatSampleLayout layout;
};

enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
};

// define our "logger" function
typedef void(*Logger)(uint8_t level, const char* message);

// ================================================================
//                        INITIALIZATION
// ================================================================

//! The `version` function shall not be changed in future versions of the interface as it is used to determine the version of the interface
//! at runtime.

#define BIRB_VISION_IMPLEMENT_VERSION_FUNCTION \
    const char* birb_vision_nest_interface_version() { \
        return BIRB_VISION_NEST_INTERFACE_VERSION; \
    }

/// Get the version of the interface.
///
/// This function shall return `BIRB_VISION_NEST_INTERFACE_VERSION` without any side effects.
const char* birb_vision_nest_interface_version();

// TODO maybe this is too unsafe since the library could be loaded different times
// say from different version of the higher level crates
//void initialize();
//void shutdown();

// ================================================================
//                        DEVICE DISCOVERY
// ================================================================

// ================================
//          Transport Layer
// ================================

struct TransportLayerList;

/// Get a list of supported transport layers.
///
/// To free the list, use `transport_layer_list_free`.
///
/// @return A list of supported transport layers or `NULL`. The list must be freed using `transport_layer_list_free`.
struct TransportLayerList* supported_transport_layers(Logger logger);

/// Free a list of transport layers.
/// The list must be freed exactly once.
///
/// @param list The list to free.
void transport_layer_list_free(
    struct TransportLayerList* list
);

/// Get an item from the list of transport layers.
///
/// If the index is out of bounds, the function will return `NULL`. This can be used to determine the length of the list.
/// The returned string is owned by the list and must not be freed.
///
/// @param list The list of transport layers.
/// @param index The index of the item to get.
/// @return A string representing the transport layer or `NULL`.
const char* transport_layer_list_get(
    const struct TransportLayerList* list,
    int32_t index
);

// ================================
//      Device Info (Discovery)
// ================================

struct DeviceInfo;
struct DeviceInfoList;

struct DeviceInfoList* discover_devices(
    const char** transport_layers
);

void device_info_list_free(
    struct DeviceInfoList* list
);

struct DeviceInfo* device_info_list_get(
    const struct DeviceInfoList* list,
    int32_t index
);

const char* device_info_display_name(
    const struct DeviceInfo* device_info
);

const char* device_info_field_name(
    const struct DeviceInfo* device_info,
    int32_t index
);

const char* device_info_field_value(
    const struct DeviceInfo* device_info,
    int32_t index
);

// ================================================================
//                            DEVICE
// ================================================================

struct Device;

struct Device* open_device(
    const struct DeviceInfo* device_info
);

void device_close(
    struct Device* device
);

struct DeviceInfo* device_info(
    const struct Device* device
);

void device_start_grabbing(
    struct Device* device
);

void device_stop_grabbing(
    struct Device* device
);

void device_grab(
    struct Device* device
);

typedef void (*FrameCallback)(struct FlatSample* frame, void* user_data);

void set_stream_callback(
    struct Device* device,
    FrameCallback callback,
    void* user_data
);

void on_stream_callback_dropped(
    struct Device* device,
    FrameCallback callback,
    void* user_data
);

#ifdef __cplusplus
}
#endif