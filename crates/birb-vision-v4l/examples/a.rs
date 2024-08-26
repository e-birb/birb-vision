use birb_vision_v4l::V4lDevice;



fn main() {
    let dev = V4lDevice::from_path("/dev/video0").unwrap();
}