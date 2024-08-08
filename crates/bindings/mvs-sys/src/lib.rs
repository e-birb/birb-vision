#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg(not(doctest))]
#![doc = include_str!("../README.md")]

include!("./mvs.rs");
include!(concat!(env!("OUT_DIR"), "/mvs_config.rs"));

impl MVS {
    /// Loads [`DYNAMIC_LIBRARY_NAME`]
    pub unsafe fn load() -> Result<Self, libloading::Error> {
        Self::new(DYNAMIC_LIBRARY_NAME)
    }
}

//#[cfg(test)]
//mod tests {
//    #[cfg(feature = "libloading")]
//    #[test]
//    fn test_load_and_enumerate() {
//        unsafe {
//            use super::*;
//
//            let mvs = MVS::load().unwrap();
//            MVSError::result_from_code(mvs.MV_CC_Initialize()).unwrap();
//
//            MVSError::result_from_code(mvs.MV_CC_EnumDevices(
//                //MV_GIGE_DEVICE | MV_USB_DEVICE,
//                MV_USB_DEVICE,
//                &mut std::mem::zeroed(),
//            )).expect("Failed to enumerate cameras");
//
//            assert_eq!(
//                MVSError::result_from_code(mvs.MV_CC_EnumDevices(
//                    MV_USB_DEVICE,
//                    std::ptr::null_mut(),
//                )).expect_err("Should have failed"),
//                MVSError::PARAMETER,
//            );
//
//            MVSError::result_from_code(mvs.MV_CC_Finalize()).unwrap();
//        }
//    }
//
//    #[test]
//    fn version() {
//        use super::*;
//
//        let number = 0x01020304;
//        let version = MVSVersion::from_u32(number);
//        assert_eq!(version.main, 1);
//        assert_eq!(version.sub, 2);
//        assert_eq!(version.rev, 3);
//        assert_eq!(version.test, 4);
//        assert_eq!(version.to_u32(), number);
//    }
//}
