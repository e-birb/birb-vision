use mvs_sys::{MVS, MV_OK, MV_USB_DEVICE};

fn main() {
    unsafe {
        let mv = MVS::load().unwrap();

        assert_eq!(mv.MV_CC_Initialize(), MV_OK as _);
        println!("Successfully initialized!");

        let mut dev_list: mvs_sys::MV_CC_DEVICE_INFO_LIST = std::mem::zeroed();

        assert_eq!(mv.MV_CC_EnumDevices(MV_USB_DEVICE, &mut dev_list), MV_OK as _);

        for i in 0..dev_list.nDeviceNum {
            let dev_info = dev_list.pDeviceInfo[i as usize];
            println!("Device: {:?}", dev_info);
        }

        assert_eq!(mv.MV_CC_Finalize(), MV_OK as _);
        println!("finalized!");
    }
}