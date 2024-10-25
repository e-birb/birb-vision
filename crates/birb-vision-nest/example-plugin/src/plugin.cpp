#include <iostream>
#include <vector>
#include <string>
#include <sstream>
#include <thread>

// The interface we are implementing
#define BIRB_VISION_INTERFACE_IMPLEMENTATION
#include <birb-vision-nest/interface.h>

#define STB_IMAGE_IMPLEMENTATION
#include <stb_image.h>

using namespace std;

extern "C" {
    // ================================================================
    //                        INITIALIZATION
    // ================================================================

    // ================================================================
    //                        DEVICE DISCOVERY
    // ================================================================

    struct TransportLayerList* supported_transport_layers(Logger logger) {
        logger(Info, "The example plugin is enumerating transport layers");
        return (struct TransportLayerList*)1;
    }

    void transport_layer_list_free(
        struct TransportLayerList* list
    ) {}

    const char* transport_layer_list_get(
        const struct TransportLayerList* list,
        int32_t index
    ) {
        switch (index)
        {
        case 0:
            return "file";
        case 1:
            return "bazinga";
        default:
            return nullptr;
        }
    }

    struct DeviceInfoList* discover_devices(
        Logger logger,
        const char** transport_layers
    ) {
        vector<string> transport_layers_vec;
        if (transport_layers != nullptr) {
            for (const char** p_layer = transport_layers; *p_layer != nullptr; p_layer++) {
                transport_layers_vec.push_back(string(*p_layer));
            }
        }

        stringstream ss;
        ss << "The example plugin is discovering devices using" << transport_layers_vec.size() << " transport layers:" << endl;
        for (const string& layer : transport_layers_vec) {
            ss << "  - " << layer << endl;
        }
        logger(Info, ss.str().c_str());

        return (struct DeviceInfoList*)1;
    }

    void device_info_list_free(
        struct DeviceInfoList* list
    ) {}

    struct DeviceInfo* device_info_list_get(
        const struct DeviceInfoList* list,
        int32_t index
    ) {
        return (struct DeviceInfo*)1;
    }

    const char* device_info_display_name(
        const struct DeviceInfo* device_info
    ) {
        return "Example Device";
    }

    const char* device_info_field_name(
        const struct DeviceInfo* device_info,
        int32_t index
    ) {
        if (index == 0) {
            return "Serial Number";
        } else {
            return nullptr;
        }
    }

    const char* device_info_field_value(
        const struct DeviceInfo* device_info,
        int32_t index
    ) {
        if (index == 0) {
            return "123456";
        } else {
            return nullptr;
        }
    }

    // ================================================================
    //                            DEVICE
    // ================================================================

    struct Device {
        FrameCallback callback;
        thread thread;

        Device() {
        }
    };

    struct Device* open_device(
        const struct DeviceInfo* device_info
    ) {
        return new Device();
    }

    void device_close(
        struct Device* device
    ) {
        if (device != nullptr) {
            delete device;
        }
    }

    struct DeviceInfo* device_info(
        const struct Device* device
    ) {
        return (struct DeviceInfo*)1;
    }

    void device_start_grabbing(
        struct Device* device
    ) {
        device->thread = thread([device]() {
            for (int i = 0; i < 100; i++) {
                this_thread::sleep_for(chrono::milliseconds(100));
                cout << "Grabbing frame " << i << endl;
                if (device->callback != nullptr) {
                    // TODO ...
                }
            }
        });
    }

    void device_stop_grabbing(
        struct Device* device
    ) {
        if (device->thread.joinable()) {
            device->thread.join();
        }
    }

    void device_grab(
        struct Device* device
    ) {}

    void device_set_stream_callback(
        struct Device* device,
        FrameCallback callback,
        void* user_data
    ) {
        device->callback = callback;
    }

    // ================================================================
    //                       DEVICE CONTROL
    // ================================================================

    struct ControlList* device_controls(
        const struct Device* device
    ) {
        return (struct ControlList*)1;
    }

    void control_list_free(
        struct ControlList* list
    ) {}
}


class PluginGuard {
public:
    PluginGuard() {
        std::cout << "The example plugin is loading" << std::endl;
    }

    ~PluginGuard() {
        std::cout << "The example plugin is unloading" << std::endl;
    }
};

PluginGuard _plugin_guard;