cmake_minimum_required(VERSION 3.10)

PROJECT(birb-vision-example-plugin LANGUAGES CXX)

# we use cmrc to include sample images
include(FetchContent)
FetchContent_Declare(
    cmrc
    GIT_REPOSITORY
    https://github.com/vector-of-bool/cmrc
    GIT_TAG
    2.0.1
)
FetchContent_MakeAvailable(cmrc)

# we use stb to load images
FetchContent_Declare(
    stb
    GIT_REPOSITORY
    https://github.com/nothings/stb
    GIT_TAG
    master
)
FetchContent_MakeAvailable(stb)

add_library(
    plugin
    SHARED
    src/plugin.cpp
)

target_include_directories(
    plugin
    PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../interfaces/v0
    # stb is a header only library
    ${stb_SOURCE_DIR}
)