# `birb-vision-nest`

```sh
cd example-plugin
mkdir build
cd build
cmake ..
cmake --build .
```
Now we can check the plugin with the following:
```sh
cd ../..
cargo run -p birb-vision-nest -- check example-plugin/build/libplugin.so
```