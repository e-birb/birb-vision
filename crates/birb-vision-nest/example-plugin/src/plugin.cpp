#include <iostream>

// The interface we are implementing
#include <birb-vision-nest/interface.h>

#define STB_IMAGE_IMPLEMENTATION
#include <stb_image.h>

extern "C" {
    BIRB_VISION_IMPLEMENT_VERSION_FUNCTION

    struct TransportLayerList* supported_transport_layers(Logger logger) {
        logger(Info, "The example plugin is enumerating transport layers");
        return nullptr;
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