use std::{marker::PhantomData, ops::{Deref, RangeInclusive}, sync::Mutex};

use icube_sdk_sys::{v1, v2, SDK};

use crate::{arr_to_str, iCubeContext, iCubeError, IntoICubeResult};

#[allow(non_camel_case_types)]
pub struct iCubeDevice {
    pub(crate) handle: DeviceHandle,

    pub(crate) callback: Box<OptionalCallbackWrapper>,

    /// This is a marker to ensure that the device is not [`Send`] nor [`Sync`].
    pub(crate) _marker: PhantomData<*const ()>,
}

impl iCubeDevice {
    pub fn device_index(&self) -> i32 {
        self.handle.index()
    }

    /// TODO ???
    ///
    /// This method corresponds to the [`ICubeSDK_Open`] function.
    ///
    /// [`ICubeSDK_Open`]: icube_sdk_sys::ffi::ICubeSDK_Open
    pub fn get_size(&self) -> Result<(i32, i32), iCubeError> {
        let mut width = 0;
        let mut height = 0;

        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.GetSize)(self.handle.index() as _, &mut width, &mut height).v1_result()?,
                SDK::V2(api) => (api.GetSize)(self.handle.index(), &mut width, &mut height).v2_result()?,
            }
        }

        Ok((width, height))
    }

    pub fn set_callback(&self, callback: Box<Callback>) {
        *self.callback.lock().unwrap() = Some(callback);
    }

    pub fn start_video_stream(&self, preview: bool, callback: bool) -> Result<(), iCubeError> {
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.Start)(self.handle.index() as _).v1_result(),
                SDK::V2(api) => (api.Start)(self.handle.index(), std::ptr::null_mut(), preview as _, callback as _).v2_result(),
            }
        }
    }

    pub fn stop_video_stream(&self) -> Result<(), iCubeError> {
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.Stop)(self.handle.index() as _).v1_result(),
                SDK::V2(api) => (api.Stop)(self.handle.index()).v2_result(),
            }
        }
    }

    pub fn is_video_streaming(&self) -> Option<bool> {
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(_) => None,
                SDK::V2(api) => Some(match (api.IsStarted)(self.handle.index()) {
                    v2::ON => true,
                    v2::OFF => false,
                    status => panic!("unexpected return value from ICubeSDK_IsStarted: {}", status),
                }),
            }
        }
    }

    pub fn get_frame_size(&self) -> Result<(i32, i32), iCubeError> {
        let mut width = 0;
        let mut height = 0;
        //ic_try!(ICubeSDK_GetSize(self.handle.index(), &mut width, &mut height))?;
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.GetSize)(self.handle.index() as _, &mut width, &mut height).v1_result()?,
                SDK::V2(api) => (api.GetSize)(self.handle.index(), &mut width, &mut height).v2_result()?,
            }
        }
        Ok((width, height))
    }

    pub fn get_name(&self) -> Result<String, iCubeError> {
        //let mut name = [0i8; ffi::NETCAM_NAME_LENGTH as usize];
        //ic_try!(ICubeSDK_GetName(self.handle.index(), name.as_mut_ptr())).map(|_| {
        //    let name_len = name.iter().position(|&c| c == 0).unwrap_or(name.len());
        //    name[..name_len].iter().map(|&c| c as u8 as char).collect::<String>()
        //})
        let mut name = [0i8; v2::NETCAM_NAME_LENGTH as usize];
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.GetName)(self.handle.index as _, name.as_mut_ptr(), v2::NETCAM_NAME_LENGTH as _).v1_result()?,
                SDK::V2(api) => (api.GetName)(self.handle.index, name.as_mut_ptr()).v2_result()?,
            };
        }
        Ok(arr_to_str(&name))
    }

    pub fn get_broken_frames(&self) -> Result<i32, iCubeError> {
        //ic_try!(ICubeSDK_GetBrokenFrames(self.handle.index(), &mut count))?;
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut count = 0;
                    (api.GetBrokenFrames)(self.handle.index() as _, &mut count).v1_result()?;
                    Ok(count as _)
                },
                SDK::V2(api) => {
                    let mut count = 0;
                    (api.GetBrokenFrames)(self.handle.index(), &mut count).v2_result()?;
                    Ok(count)
                },
            }
        }
    }

    pub fn get_good_frames(&self) -> Result<i32, iCubeError> {
        //ic_try!(ICubeSDK_GetGoodFrames(self.handle.index(), &mut count))?;
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut count = 0;
                    (api.GetGoodFrames)(self.handle.index() as _, &mut count).v1_result()?;
                    Ok(count as _)
                },
                SDK::V2(api) => {
                    let mut count = 0;
                    (api.GetGoodFrames)(self.handle.index(), &mut count).v2_result()?;
                    Ok(count)
                },
            }
        }
    }

    pub fn set_display_mode(
        &self,
        mode: DisplayMode,
        properties: DisplayProperty,
    ) -> Result<(), iCubeError> {
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(_) => Err(iCubeError::Unimplemented),
                SDK::V2(api) => (api.SetDisplayMode)(self.handle.index(), mode as _, properties.into_sdk_struct_v2()).v2_result(),
            }
        }
    }

    pub fn get_version(&self) -> Result<String, iCubeError> {
        //let mut version = [0i8; ffi::NETCAM_VERSION_LENGTH as usize];
        //unsafe {
        //    sdk!(ICubeSDK_GetVersion)(self.handle.index(), version.as_mut_ptr());
        //}
//
        //let version_len = version.iter().position(|&c| c == 0).unwrap_or(version.len());
        //Ok(version[..version_len].iter().map(|&c| c as u8 as char).collect::<String>())
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut version = [0i8; v2::NETCAM_VERSION_LENGTH as usize];
                    (api.GetApiVersion)(version.as_mut_ptr(), v2::NETCAM_VERSION_LENGTH as _);
                    Ok(arr_to_str(&version))
                },
                SDK::V2(api) => {
                    let mut version = [0i8; v2::NETCAM_VERSION_LENGTH as usize];
                    (api.GetVersion)(self.handle.index(), version.as_mut_ptr());
                    Ok(arr_to_str(&version))
                },
            }
        }
    }

    pub fn get_firmware_version(&self) -> Result<String, iCubeError> {
        //let mut version = [0i8; ffi::NETCAM_VERSION_LENGTH as usize];
        //ic_try!(ICubeSDK_GetFWVersion(self.handle.index(), version.as_mut_ptr()))?;
        //let version_len = version.iter().position(|&c| c == 0).unwrap_or(version.len());
        //Ok(version[..version_len].iter().map(|&c| c as u8 as char).collect::<String>())
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut version = [0i8; v2::NETCAM_VERSION_LENGTH as usize];
                    (api.GetFWVersion)(self.handle.index() as _, version.as_mut_ptr(), v2::NETCAM_VERSION_LENGTH as _);
                    Ok(arr_to_str(&version))
                },
                SDK::V2(api) => {
                    let mut version = [0i8; v2::NETCAM_VERSION_LENGTH as usize];
                    (api.GetFWVersion)(self.handle.index(), version.as_mut_ptr());
                    Ok(arr_to_str(&version))
                },
            }
        }
    }

    pub fn get_serial_number(&self) -> Result<String, iCubeError> {
        //let mut serial = [0i8; ffi::NETCAM_SERIAL_LENGTH as usize];
        //ic_try!(ICubeSDK_GetSerialNum(self.handle.index(), serial.as_mut_ptr()))?;
        //let serial_len = serial.iter().position(|&c| c == 0).unwrap_or(serial.len());
        //Ok(serial[..serial_len].iter().map(|&c| c as u8 as char).collect::<String>())
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(_) => Err(iCubeError::Unimplemented),
                SDK::V2(api) => {
                    let mut serial = [0i8; v2::NETCAM_SERIAL_LENGTH as usize];
                    (api.GetSerialNum)(self.handle.index(), serial.as_mut_ptr());
                    Ok(arr_to_str(&serial))
                },
            }
        }
    }

    pub fn get_fpga_version(&self) -> Result<String, iCubeError> {
        //let mut version = [0i8; ffi::NETCAM_VERSION_LENGTH as usize];
        //ic_try!(ICubeSDK_GetFPGAVersion(self.handle.index(), version.as_mut_ptr()))?;
        //let version_len = version.iter().position(|&c| c == 0).unwrap_or(version.len());
        //Ok(version[..version_len].iter().map(|&c| c as u8 as char).collect::<String>())
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(_) => Err(iCubeError::Unimplemented),
                SDK::V2(api) => {
                    let mut version = [0i8; v2::NETCAM_VERSION_LENGTH as usize];
                    (api.GetFGPAVersion)(self.handle.index(), version.as_mut_ptr());
                    Ok(arr_to_str(&version))
                },
            }
        }
    }

    pub fn set_roi_property(&self, roi: RoiProperty) -> Result<(), iCubeError> {
        //ic_try!(ICubeSDK_SetResolution(self.handle.index(), &mut RoiProperty::to_sdk_struct(&roi)))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.SetResolution)(self.handle.index() as _, roi.width, roi.height, roi.x, roi.y).v1_result(), // TODO enable missing!!!
                SDK::V2(api) => (api.SetResolution)(self.handle.index(), &mut roi.to_sdk_struct_v2()).v2_result(),
            }
        }
    }

    pub fn get_resolution(&self) -> Result<RoiProperty, iCubeError> {
        //let mut roi = ffi::ROI_PROPERTY {
        //    bEnabled: 0,
        //    nXPos: 0,
        //    nYPos: 0,
        //    nXRes: 0,
        //    nYRes: 0,
        //};
        //ic_try!(ICubeSDK_GetResolution(self.handle.index(), &mut roi))?;
        //Ok(RoiProperty::from_sdk_struct(&roi))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut x = 0;
                    let mut y = 0;
                    let mut width = 0;
                    let mut height = 0;
                    (api.GetResolution)(self.handle.index() as _, &mut width, &mut height, &mut x, &mut y).v1_result()?;
                    Ok(RoiProperty {
                        enabled: true,
                        x,
                        y,
                        width,
                        height,
                    })
                },
                SDK::V2(api) => {
                    let mut roi = v2::ROI_PROPERTY {
                        bEnabled: 0,
                        nXPos: 0,
                        nYPos: 0,
                        nXRes: 0,
                        nYRes: 0,
                    };
                    (api.GetResolution)(self.handle.index(), &mut roi).v2_result()?;
                    Ok(RoiProperty::from_sdk_struct_v2(&roi))
                },
            }
        }
    }

    pub fn get_resolution_range(&self) -> Result<RoiRange, iCubeError> {
        //let mut roi_range = ffi::ROI_RANGE_PROPERTY {
        //    nXMin: 0,
        //    nXMax: 0,
        //    nYMin: 0,
        //    nYMax: 0,
        //};
        //ic_try!(ICubeSDK_GetResolutionRange(self.handle.index(), &mut roi_range))?;
        //Ok(RoiRange {
        //    width_range: roi_range.nXMin..=roi_range.nXMax,
        //    height_range: roi_range.nYMin..=roi_range.nYMax,
        //})
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut roi_range = v1::ROI_RANGE_PROPERTY {
                        nXMin: 0,
                        nXMax: 0,
                        nYMin: 0,
                        nYMax: 0,
                    };
                    (api.GetResolutionRange)(self.handle.index() as _, &mut roi_range).v1_result()?;
                    Ok(RoiRange {
                        width_range: roi_range.nXMin..=roi_range.nXMax,
                        height_range: roi_range.nYMin..=roi_range.nYMax,
                    })
                },
                SDK::V2(api) => {
                    let mut roi_range = v2::ROI_RANGE_PROPERTY {
                        nXMin: 0,
                        nXMax: 0,
                        nYMin: 0,
                        nYMax: 0,
                    };
                    (api.GetResolutionRange)(self.handle.index(), &mut roi_range).v2_result()?;
                    Ok(RoiRange {
                        width_range: roi_range.nXMin..=roi_range.nXMax,
                        height_range: roi_range.nYMin..=roi_range.nYMax,
                    })
                },
            }
        }
    }

    // TODO incomprehensible!
    //pub fn set_resolution_param(&self, roi: &RoiProperty) -> Result<(), iCubeError> {
    //    ic_try!(ICubeSDK_SetResolutionParam(self.handle.index(), &mut RoiProperty::to_sdk_struct(roi)))
    //}

    pub fn set_mode(&self, resolution_mode: &ResolutionMode) -> Result<(), iCubeError> {
        //ic_try!(ICubeSDK_SetMode(self.handle.index(), resolution_mode.into_sdk_value()))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.SetMode)(self.handle.index() as _, resolution_mode.into_sdk_value_v1()? as _).v1_result(),
                SDK::V2(api) => (api.SetMode)(self.handle.index(), resolution_mode.into_sdk_value_v2()).v2_result(),
            }
        }
    }

    pub fn get_mode(&self) -> Result<ResolutionMode, iCubeError> {
        //let mut mode = 0;
        //ic_try!(ICubeSDK_GetMode(self.handle.index(), &mut mode))?;
        //Ok(ResolutionMode::from_sdk_value(mode))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut mode = 0;
                    (api.GetMode)(self.handle.index() as _, &mut mode).v1_result()?;
                    Ok(ResolutionMode::from_sdk_value_v1(mode as _))
                },
                SDK::V2(api) => {
                    let mut mode = 0;
                    (api.GetMode)(self.handle.index(), &mut mode).v2_result()?;
                    Ok(ResolutionMode::from_sdk_value_v2(mode))
                },
            }
        }
    }

    pub fn get_mode_list(&self) -> Result<Vec<ResolutionMode>, iCubeError> {
        //let mut modes = vec![0; 255];
        //let mut count = 0;
        //ic_try!(ICubeSDK_GetModeList(self.handle.index(), modes.as_mut_ptr(), &mut count))?;
        //modes.truncate(count as usize);
        //Ok(modes.into_iter().map(ResolutionMode::from_sdk_value).collect())
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut modes = vec![0; 255];
                    let mut count = 0;
                    (api.GetModeList)(self.handle.index() as _, modes.as_mut_ptr(), &mut count).v1_result()?;
                    modes.truncate(count as usize);
                    Ok(modes.into_iter().map(|mode| ResolutionMode::from_sdk_value_v1(mode as _)).collect())
                },
                SDK::V2(api) => {
                    let mut modes = vec![0; 255];
                    let mut count = 0;
                    (api.GetModeList)(self.handle.index(), modes.as_mut_ptr(), &mut count).v2_result()?;
                    modes.truncate(count as usize);
                    Ok(modes.into_iter().map(ResolutionMode::from_sdk_value_v2).collect())
                },
            }
        }
    }

    pub fn set_bin_skip(&self, mode: BinOrSkipMode, param: BinSkipParameter) -> Result<(), iCubeError> {
        //ic_try!(ICubeSDK_SetBinSkip(self.handle.index(), param.into_sdk_value(), mode.into_sdk_value()))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.SetBinSkip)(self.handle.index() as _, param.into_sdk_value_v1() as _, mode.into_sdk_value_v1()).v1_result(),
                SDK::V2(api) => (api.SetBinSkip)(self.handle.index(), param.into_sdk_value_v2(), mode.into_sdk_value_v2()).v2_result(),
            }
        }
    }

    pub fn get_bin_skip(&self, mode: BinOrSkipMode) -> Result<BinSkipParameter, iCubeError> {
        //let mut param = 0;
        //ic_try!(ICubeSDK_GetBinSkip(self.handle.index(), &mut param, mode.into_sdk_value()))?;
        //Ok(BinSkipParameter::from_sdk_value(param))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut param = 0;
                    (api.GetBinSkip)(self.handle.index() as _, &mut param, mode.into_sdk_value_v1()).v1_result()?;
                    Ok(BinSkipParameter::from_sdk_value_v1(param as _))
                },
                SDK::V2(api) => {
                    let mut param = 0;
                    (api.GetBinSkip)(self.handle.index(), &mut param, mode.into_sdk_value_v2()).v2_result()?;
                    Ok(BinSkipParameter::from_sdk_value_v2(param))
                },
            }
        }
    }

    pub fn get_bin_skip_list(&self, mode: BinOrSkipMode) -> Result<Vec<BinSkipParameter>, iCubeError> {
        //let mut params = vec![0; 255];
        //let mut count = 0;
        //ic_try!(ICubeSDK_GetBinSkipList(self.handle.index(), mode.into_sdk_value(), params.as_mut_ptr(), &mut count))?;
        //params.truncate(count as usize);
        //Ok(params.into_iter().map(BinSkipParameter::from_sdk_value).collect())
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut params = vec![0; 255];
                    let mut count = 0;
                    (api.GetBinSkipList)(self.handle.index() as _, mode.into_sdk_value_v1() as _, params.as_mut_ptr(), &mut count).v1_result()?;
                    params.truncate(count as usize);
                    Ok(params.into_iter().map(|param| BinSkipParameter::from_sdk_value_v1(param as _)).collect())
                },
                SDK::V2(api) => {
                    let mut params = vec![0; 255];
                    let mut count = 0;
                    (api.GetBinSkipList)(self.handle.index(), mode.into_sdk_value_v2(), params.as_mut_ptr(), &mut count).v2_result()?;
                    params.truncate(count as usize);
                    Ok(params.into_iter().map(BinSkipParameter::from_sdk_value_v2).collect())
                },
            }
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), iCubeError> {
        //let path = std::ffi::CString::new(path).expect("Failed to convert path to CString");
        //ic_try!(ICubeSDK_SaveToFile(self.handle.index(), path.as_ptr()))
        let path = std::ffi::CString::new(path).expect("Failed to convert path to CString");
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.SaveToFile)(self.handle.index() as _, path.as_ptr()).v1_result(),
                SDK::V2(api) => (api.SaveToFile)(self.handle.index(), path.as_ptr()).v2_result(),
            }
        }
    }

    // TODO
    //pub fn save_avi(&self, path: &str) -> Result<(), iCubeError> {
    //    let path = std::ffi::CString::new(path).expect("Failed to convert path to CString");
    //    ic_try!(ICubeSDK_SaveAvi(self.handle.index(), path.as_ptr()))
    //}

    pub fn set_trigger(&self, mode: TriggerMode) -> Result<(), iCubeError> {
        //ic_try!(ICubeSDK_SetTrigger(self.handle.index(), mode.into_sdk_value()))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.SetTrigger)(self.handle.index() as _, mode.into_sdk_value_v1()? as _).v1_result(),
                SDK::V2(api) => (api.SetTrigger)(self.handle.index(), mode.into_sdk_value_v2()).v2_result(),
            }
        }
    }

    pub fn get_trigger(&self) -> Result<TriggerMode, iCubeError> {
        //let mut mode = 0;
        //ic_try!(ICubeSDK_GetTrigger(self.handle.index(), &mut mode))?;
        //Ok(TriggerMode::from_sdk_value(mode))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut mode = 0;
                    (api.GetTrigger)(self.handle.index() as _, &mut mode).v1_result()?;
                    Ok(TriggerMode::from_sdk_value_v1(mode as _))
                },
                SDK::V2(api) => {
                    let mut mode = 0;
                    (api.GetTrigger)(self.handle.index(), &mut mode).v2_result()?;
                    Ok(TriggerMode::from_sdk_value_v2(mode))
                },
            }
        }
    }

    pub fn set_cam_parameter(&self, id: ParameterID, value: u64) -> Result<(), iCubeError> {
        //ic_try!(ICubeSDK_SetCamParameter(self.handle.index(), id as u32 as _, value))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.SetCamParameter)(self.handle.index() as _, id as u32 as _, value).v1_result(),
                SDK::V2(api) => (api.SetCamParameter)(self.handle.index(), id as u32 as _, value).v2_result(),
            }
        }
    }

    pub fn get_cam_parameter(&self, id: ParameterID) -> Result<u64, iCubeError> {
        //let mut value = 0;
        //ic_try!(ICubeSDK_GetCamParameter(self.handle.index(), id as u32 as _, &mut value))?;
        //Ok(value)
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut value = 0;
                    (api.GetCamParameter)(self.handle.index() as _, id as u32 as _, &mut value).v1_result()?;
                    Ok(value as _)
                },
                SDK::V2(api) => {
                    let mut value = 0;
                    (api.GetCamParameter)(self.handle.index(), id as u32 as _, &mut value).v2_result()?;
                    Ok(value)
                },
            }
        }
    }

    pub fn get_cam_parameter_range(&self, id: ParameterID) -> Result<ParameterProperty, iCubeError> {
        //let mut prop = unsafe { std::mem::zeroed() };
        //ic_try!(ICubeSDK_GetCamParameterRange(self.handle.index(), id as u32 as _, &mut prop))?;
        //Ok(ParameterProperty::from_sdk_struct(&prop))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut prop = std::mem::zeroed();
                    (api.GetCamParameterRange)(self.handle.index() as _, id as u32 as _, &mut prop).v1_result()?;
                    Ok(ParameterProperty::from_sdk_struct_v1(&prop))
                },
                SDK::V2(api) => {
                    let mut prop = std::mem::zeroed();
                    (api.GetCamParameterRange)(self.handle.index(), id as u32 as _, &mut prop).v2_result()?;
                    Ok(ParameterProperty::from_sdk_struct_v2(&prop))
                },
            }
        }
    }

    pub fn set_exposure(&self, value: f32) -> Result<(), iCubeError> {
        //ic_try!(ICubeSDK_SetExposure(self.handle.index(), value))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.SetExposure)(self.handle.index() as _, value).v1_result(),
                SDK::V2(api) => (api.SetExposure)(self.handle.index(), value).v2_result(),
            }
        }
    }

    pub fn get_exposure(&self) -> Result<f32, iCubeError> {
        //let mut value = 0.0;
        //ic_try!(ICubeSDK_GetExposure(self.handle.index(), &mut value))?;
        //Ok(value)
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut value = 0.0;
                    (api.GetExposure)(self.handle.index() as _, &mut value).v1_result()?;
                    Ok(value)
                },
                SDK::V2(api) => {
                    let mut value = 0.0;
                    (api.GetExposure)(self.handle.index(), &mut value).v2_result()?;
                    Ok(value)
                },
            }
        }
    }

    pub fn get_exposure_range(&self) -> Result<RangeInclusive<f32>, iCubeError> {
        //let mut prop = unsafe { std::mem::zeroed() };
        //ic_try!(ICubeSDK_GetExposureRange(self.handle.index(), &mut prop))?;
        //Ok(prop.nMin..=prop.nMax)
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut prop = std::mem::zeroed();
                    (api.GetExposureRange)(self.handle.index() as _, &mut prop).v1_result()?;
                    Ok(prop.nMin..=prop.nMax)
                },
                SDK::V2(api) => {
                    let mut prop = std::mem::zeroed();
                    (api.GetExposureRange)(self.handle.index(), &mut prop).v2_result()?;
                    Ok(prop.nMin..=prop.nMax)
                },
            }
        }
    }

    pub fn get_param_auto_supported(&self, id: ParameterID) -> Result<bool, iCubeError> {
        //let mut auto = 0;
        //ic_try!(ICubeSDK_GetParamAuto(self.handle.index(), id as u32 as _, &mut auto))?;
        //Ok(auto != 0)
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => {
                    let mut auto = 0;
                    (api.GetParamAuto)(self.handle.index() as _, id as u32 as _, &mut auto).v1_result()?;
                    Ok(auto != 0)
                },
                SDK::V2(api) => {
                    let mut auto = 0;
                    (api.GetParamAuto)(self.handle.index(), id as u32 as _, &mut auto).v2_result()?;
                    Ok(auto != 0)
                },
            }
        }
    }

    pub fn set_param_auto(&self, id: ParameterID, value: bool) -> Result<(), iCubeError> {
        //ic_try!(ICubeSDK_SetParamAuto(self.handle.index(), id as u32 as _, value as _))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.SetParamAuto)(self.handle.index() as _, id as u32 as _, value as _).v1_result(),
                SDK::V2(api) => (api.SetParamAuto)(self.handle.index(), id as u32 as _, value as _).v2_result(),
            }
        }
    }

    // TODO missing?
    //pub fn set_param_default(&self, id: ParameterID) -> Result<(), iCubeError> {
    //    ic_try!(ICubeSDK_SetParamDef(self.handle.index(), id as u32 as _))
    //}

    pub fn set_param_one_push(&self, id: ParameterID) -> Result<(), iCubeError> {
        //ic_try!(ICubeSDK_SetParamOnePush(self.handle.index(), id as u32 as _))
        unsafe {
            match self.handle.ctx.sdk() {
                SDK::V1(api) => (api.SetParamOnePush)(self.handle.index() as _, id as u32 as _).v1_result(),
                SDK::V2(api) => (api.SetParamOnePush)(self.handle.index(), id as u32 as _).v2_result(),
            }
        }
    }
}

impl Drop for iCubeDevice {
    fn drop(&mut self) {
        if let Err(r) = self.stop_video_stream() {
            log::error!("Failed to stop video stream while dropping iCubeDevice: {:?}", r);
        }
    }
}

pub(crate) struct DeviceHandle {
    pub(crate) ctx: iCubeContext,
    pub(crate) index: i32,
}

impl DeviceHandle {
    pub(crate) fn index(&self) -> i32 {
        self.index
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        let r = unsafe {
            match self.ctx.sdk() {
                SDK::V1(api) => (api.Close)(self.index() as _).v1_result(),
                SDK::V2(api) => (api.Close)(self.index()).v2_result(),
            }
        };
        if let Err(e) = r {
            eprintln!("Failed to close device handle: {:?}", e);
        }
    }
}

impl Deref for DeviceHandle {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.index
    }
}

pub type Callback = dyn Fn(CallbackEventType) + Send + Sync;
pub(crate) type OptionalCallbackWrapper = Mutex<Option<Box<Callback>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub enum CallbackEventType<'a> {
    NEW_FRAME(&'a [u8]),
    DEV_DISCONNECTED,
    DEV_RECONNECTED,
    USB_TRANSFER_FAILED,
    Unknown(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum DisplayMode {
    DISPLAY_NORMAL,
    DISPLAY_FIT_TO_WINDOW,
    DISPLAY_RECT,
}

pub struct DisplayProperty {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

impl DisplayProperty {
    pub fn into_sdk_struct_v2(&self) -> v2::DISP_PROPERTY {
        v2::DISP_PROPERTY {
            top: self.top,
            bottom: self.bottom,
            left: self.left,
            right: self.right,
        }
    }
}

pub struct RoiProperty {
    pub enabled: bool,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl RoiProperty {
    fn to_sdk_struct_v2(&self) -> v2::ROI_PROPERTY {
        v2::ROI_PROPERTY {
            bEnabled: self.enabled as _,
            nXPos: self.x,
            nYPos: self.y,
            nXRes: self.width,
            nYRes: self.height,
        }
    }

    fn from_sdk_struct_v2(sdk_struct: &v2::ROI_PROPERTY) -> Self {
        Self {
            enabled: sdk_struct.bEnabled != 0,
            x: sdk_struct.nXPos,
            y: sdk_struct.nYPos,
            width: sdk_struct.nXRes,
            height: sdk_struct.nYRes,
        }
    }
}

pub struct RoiRange {
    pub width_range: RangeInclusive<i32>,
    pub height_range: RangeInclusive<i32>,
}

impl RoiRange {
    pub fn to_sdk_struct_v1(&self) -> v1::ROI_RANGE_PROPERTY {
        v1::ROI_RANGE_PROPERTY {
            nXMin: *self.width_range.start(),
            nXMax: *self.width_range.end(),
            nYMin: *self.height_range.start(),
            nYMax: *self.height_range.end(),
        }
    }

    pub fn to_sdk_struct_v2(&self) -> v2::ROI_RANGE_PROPERTY {
        v2::ROI_RANGE_PROPERTY {
            nXMin: *self.width_range.start(),
            nXMax: *self.width_range.end(),
            nYMin: *self.height_range.start(),
            nYMax: *self.height_range.end(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum ResolutionMode {
    MODE_320x240,
    MODE_640x480,
    MODE_752x480,
    MODE_800x600,
    MODE_1024x768,
    MODE_1280x1024,
    MODE_1600x1200,
    MODE_2048x1536,
    MODE_2592x1944,
    MODE_3840x2748,
    MODE_1920x1200,
    Unknown(i32),
}

impl ResolutionMode {
    pub fn into_sdk_value_v1(self) -> Result<i32, iCubeError> {
        match self {
            Self::MODE_320x240 => Ok(v1::MOC_MODE_320x240),
            Self::MODE_640x480 => Ok(v1::MOC_MODE_640x480),
            Self::MODE_752x480 => Ok(v1::MOC_MODE_752x480),
            Self::MODE_800x600 => Ok(v1::MOC_MODE_800x600),
            Self::MODE_1024x768 => Ok(v1::MOC_MODE_1024x768),
            Self::MODE_1280x1024 => Ok(v1::MOC_MODE_1280x1024),
            Self::MODE_1600x1200 => Ok(v1::MOC_MODE_1600x1200),
            Self::MODE_2048x1536 => Ok(v1::MOC_MODE_2048x1536),
            Self::MODE_2592x1944 => Ok(v1::MOC_MODE_2592x1944),
            Self::MODE_3840x2748 => Ok(v1::MOC_MODE_3840x2748),
            Self::MODE_1920x1200 => Err(iCubeError::Unimplemented),
            Self::Unknown(v) => Ok(v),
        }
    }

    pub fn into_sdk_value_v2(self) -> i32 {
        match self {
            Self::MODE_320x240 => v2::MODE_320x240 as _,
            Self::MODE_640x480 => v2::MODE_640x480 as _,
            Self::MODE_752x480 => v2::MODE_752x480 as _,
            Self::MODE_800x600 => v2::MODE_800x600 as _,
            Self::MODE_1024x768 => v2::MODE_1024x768 as _,
            Self::MODE_1280x1024 => v2::MODE_1280x1024 as _,
            Self::MODE_1600x1200 => v2::MODE_1600x1200 as _,
            Self::MODE_2048x1536 => v2::MODE_2048x1536 as _,
            Self::MODE_2592x1944 => v2::MODE_2592x1944 as _,
            Self::MODE_3840x2748 => v2::MODE_3840x2748 as _,
            Self::MODE_1920x1200 => v2::MODE_1920x1200 as _,
            Self::Unknown(v) => v,
        }
    }

    pub fn from_sdk_value_v1(value: i32) -> Self {
        match value as _ {
            v1::MOC_MODE_320x240 => Self::MODE_320x240,
            v1::MOC_MODE_640x480 => Self::MODE_640x480,
            v1::MOC_MODE_752x480 => Self::MODE_752x480,
            v1::MOC_MODE_800x600 => Self::MODE_800x600,
            v1::MOC_MODE_1024x768 => Self::MODE_1024x768,
            v1::MOC_MODE_1280x1024 => Self::MODE_1280x1024,
            v1::MOC_MODE_1600x1200 => Self::MODE_1600x1200,
            v1::MOC_MODE_2048x1536 => Self::MODE_2048x1536,
            v1::MOC_MODE_2592x1944 => Self::MODE_2592x1944,
            v1::MOC_MODE_3840x2748 => Self::MODE_3840x2748,
            _ => Self::Unknown(value),
        }
    }

    pub fn from_sdk_value_v2(value: i32) -> Self {
        match value as _ {
            v2::MODE_320x240 => Self::MODE_320x240,
            v2::MODE_640x480 => Self::MODE_640x480,
            v2::MODE_752x480 => Self::MODE_752x480,
            v2::MODE_800x600 => Self::MODE_800x600,
            v2::MODE_1024x768 => Self::MODE_1024x768,
            v2::MODE_1280x1024 => Self::MODE_1280x1024,
            v2::MODE_1600x1200 => Self::MODE_1600x1200,
            v2::MODE_2048x1536 => Self::MODE_2048x1536,
            v2::MODE_2592x1944 => Self::MODE_2592x1944,
            v2::MODE_3840x2748 => Self::MODE_3840x2748,
            v2::MODE_1920x1200 => Self::MODE_1920x1200,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u32)]
pub enum BinSkipParameter {
    BIN_SKIP_OFF,
    BIN_SKIP_2ND_PIXEL,
    BIN_SKIP_4TH_PIXEL,
    Unknown(i32),
}

impl BinSkipParameter {
    pub fn into_sdk_value_v1(self) -> i32 {
        match self {
            Self::BIN_SKIP_OFF => v1::BIN_SKIP_OFF as _,
            Self::BIN_SKIP_2ND_PIXEL => v1::BIN_SKIP_2ND_PIXEL as _,
            Self::BIN_SKIP_4TH_PIXEL => v1::BIN_SKIP_4TH_PIXEL as _,
            Self::Unknown(v) => v,
        }
    }

    pub fn into_sdk_value_v2(self) -> i32 {
        match self {
            Self::BIN_SKIP_OFF => v2::BIN_SKIP_OFF as _,
            Self::BIN_SKIP_2ND_PIXEL => v2::BIN_SKIP_2ND_PIXEL as _,
            Self::BIN_SKIP_4TH_PIXEL => v2::BIN_SKIP_4TH_PIXEL as _,
            Self::Unknown(v) => v,
        }
    }

    pub fn from_sdk_value_v1(value: i32) -> Self {
        match value as _ {
            v1::BIN_SKIP_OFF => Self::BIN_SKIP_OFF,
            v1::BIN_SKIP_2ND_PIXEL => Self::BIN_SKIP_2ND_PIXEL,
            v1::BIN_SKIP_4TH_PIXEL => Self::BIN_SKIP_4TH_PIXEL,
            _ => Self::Unknown(value),
        }
    }

    pub fn from_sdk_value_v2(value: i32) -> Self {
        match value as _ {
            v2::BIN_SKIP_OFF => Self::BIN_SKIP_OFF,
            v2::BIN_SKIP_2ND_PIXEL => Self::BIN_SKIP_2ND_PIXEL,
            v2::BIN_SKIP_4TH_PIXEL => Self::BIN_SKIP_4TH_PIXEL,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub enum BinOrSkipMode {
    Bin,
    Skip,
}

impl BinOrSkipMode {
    pub fn into_sdk_value_v1(self) -> i32 {
        match self {
            Self::Bin => v1::MODE_BIN as _,
            Self::Skip => v1::MODE_SKIP as _,
        }
    }

    pub fn into_sdk_value_v2(self) -> i32 {
        match self {
            Self::Bin => v2::MODE_BIN as _,
            Self::Skip => v2::MODE_SKIP as _,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub enum TriggerMode {
    TRIG_SW_START,
    TRIG_SW_DO,
    TRIG_HW_START,
    TRIG_STOP,
    TRIG_SW_START_2,
    TRIG_HW_START_2,
    Unknown(i32),
}

impl TriggerMode {
    pub fn into_sdk_value_v1(self) -> Result<i32, iCubeError> {
        match self {
            Self::TRIG_SW_START => Ok(v1::TRIG_SW_START),
            Self::TRIG_SW_DO => Ok(v1::TRIG_SW_DO),
            Self::TRIG_HW_START => Ok(v1::TRIG_HW_START),
            Self::TRIG_STOP => Ok(v1::TRIG_STOP),
            Self::TRIG_SW_START_2 => Err(iCubeError::Unimplemented),
            Self::TRIG_HW_START_2 => Err(iCubeError::Unimplemented),
            Self::Unknown(v) => Ok(v),
        }
    }

    pub fn into_sdk_value_v2(self) -> i32 {
        match self {
            Self::TRIG_SW_START => v2::TRIG_SW_START as _,
            Self::TRIG_SW_DO => v2::TRIG_SW_DO as _,
            Self::TRIG_HW_START => v2::TRIG_HW_START as _,
            Self::TRIG_STOP => v2::TRIG_STOP as _,
            Self::TRIG_SW_START_2 => v2::TRIG_SW_START_2 as _,
            Self::TRIG_HW_START_2 => v2::TRIG_HW_START_2 as _,
            Self::Unknown(v) => v,
        }
    }

    pub fn from_sdk_value_v1(value: i32) -> Self {
        match value as _ {
            v1::TRIG_SW_START => Self::TRIG_SW_START,
            v1::TRIG_SW_DO => Self::TRIG_SW_DO,
            v1::TRIG_HW_START => Self::TRIG_HW_START,
            v1::TRIG_STOP => Self::TRIG_STOP,
            _ => Self::Unknown(value),
        }
    }

    pub fn from_sdk_value_v2(value: i32) -> Self {
        match value as _ {
            v2::TRIG_SW_START => Self::TRIG_SW_START,
            v2::TRIG_SW_DO => Self::TRIG_SW_DO,
            v2::TRIG_HW_START => Self::TRIG_HW_START,
            v2::TRIG_STOP => Self::TRIG_STOP,
            v2::TRIG_SW_START_2 => Self::TRIG_SW_START_2,
            v2::TRIG_HW_START_2 => Self::TRIG_HW_START_2,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
pub enum ParameterID {
    BRIGHTNESS,
    CONTRAST,
    GAMMA,
    FLIPPED_V,
    FLIPPED_H,
    WHITE_BALANCE,
    EXPOSURE_TIME,
    EXPOSURE_TARGET,
    RED,
    GREEN,
    BLUE,
    BLACKLEVEL,
    GAIN,
    COLOR,
    PLL,
    STROBE_LENGHT,
    STROBE_DELAY,
    TRIGGER_DELAY,
    SATURATION,
    COLOR_ENHANCE,
    TRIGGER_INVERT,
    RECONNECTIONS,
    MEASURE_FIELD_AE,
    AVI_STATE,
    FOCUS,
    SHUTTER,
    ROI_ID,
    ROI_CYCLE,
    DEFECT_COR,
    BAD_FRAME,
    GOOD_FRAME,
    SW_TRIG_MODE,
    ROI_FPGA_ONE_FRAME,
    CALLBACK_BR_FRAMES,
    FGPA_VBLANKING,
    FGPA_HBLANKING,
    FGPA_CLK_DIVIDER,
    FGPA_ON_BOARD,
    SET_GPIO,
    GET_GPIO,
    SET_GPIO_MODE,
    MASK_ROI_ID,
    RED_OFFSET,
    GREEN_OFFSET,
    BLUE_OFFSET,
    HUE,
    COLOR_CORRECTION,
    GAMMA_ENABLE,
    GAMMA_MODE,
    INVERT_PIXEL,
    TNR,
    BAYER_CONVERSION,
    COLOR_PROCESSING,
    USB_SPEED,
    DEVICE_SPEED,
    DATA_TRANSMISSION,
    SIGNIFICANT_BITS,
    GRAPHIC_MODE,
    EDGE_ENHANCEMENT,
    EDGE_ENHANCEMENT_GAIN,
    SENSOR_STROBE,
    TRIG_TIMEOUT,
    PIPE_TIMEOUT_MODE,
    RESET_CAMERA,
    DISCONNECTIONS,
    XACT_RECOVER_MODE,
    RESET_TO_DEFAULT,
    TRANSFER_BUF_CNT,
    SW_TRIG_WD_MODE,
    AQU_FRAMERATE,
    SENSOR_STROBE_DELAY,
    SENSOR_OVERLAPPED,
    TRIG_DELAY_DIVIDER,
}

impl ParameterID {
    pub fn into_sdk_value_v1(self) -> Result<i32, iCubeError> {
        match self {
            Self::BRIGHTNESS => Ok(v1::REG_BRIGHTNESS),
            Self::CONTRAST => Ok(v1::REG_CONTRAST),
            Self::GAMMA => Ok(v1::REG_GAMMA),
            Self::FLIPPED_V => Ok(v1::REG_FLIPPED_V),
            Self::FLIPPED_H => Ok(v1::REG_FLIPPED_H),
            Self::WHITE_BALANCE => Ok(v1::REG_WHITE_BALANCE),
            Self::EXPOSURE_TIME => Ok(v1::REG_EXPOSURE_TIME),
            Self::EXPOSURE_TARGET => Ok(v1::REG_EXPOSURE_TARGET),
            Self::RED => Ok(v1::REG_RED),
            Self::GREEN => Ok(v1::REG_GREEN),
            Self::BLUE => Ok(v1::REG_BLUE),
            Self::BLACKLEVEL => Ok(v1::REG_BLACKLEVEL),
            Self::GAIN => Ok(v1::REG_GAIN),
            Self::COLOR => Ok(v1::REG_COLOR),
            Self::PLL => Ok(v1::REG_PLL),
            Self::STROBE_LENGHT => Ok(v1::REG_STROBE_LENGHT),
            Self::STROBE_DELAY => Ok(v1::REG_STROBE_DELAY),
            Self::TRIGGER_DELAY => Ok(v1::REG_TRIGGER_DELAY),
            Self::SATURATION => Err(iCubeError::Unimplemented),
            Self::COLOR_ENHANCE =>Err(iCubeError::Unimplemented),
            Self::TRIGGER_INVERT => Ok(v1::REG_TRIGGER_INVERT),
            Self::RECONNECTIONS => Err(iCubeError::Unimplemented),
            Self::MEASURE_FIELD_AE => Ok(v1::REG_MEASURE_FIELD_AE),
            Self::AVI_STATE => Err(iCubeError::Unimplemented),
            Self::FOCUS => Err(iCubeError::Unimplemented),
            Self::SHUTTER => Ok(v1::REG_SHUTTER),
            Self::ROI_ID => Ok(v1::REG_ROI_ID),
            Self::ROI_CYCLE => Ok(v1::REG_ROI_CYCLE),
            Self::DEFECT_COR => Ok(v1::REG_DEFECT_COR),
            Self::BAD_FRAME => Err(iCubeError::Unimplemented),
            Self::GOOD_FRAME => Err(iCubeError::Unimplemented),
            Self::SW_TRIG_MODE => Ok(v1::REG_SW_TRIG_MODE),
            Self::ROI_FPGA_ONE_FRAME => Err(iCubeError::Unimplemented),
            Self::CALLBACK_BR_FRAMES => Ok(v1::REG_CALLBACK_BR_FRAMES),
            Self::FGPA_VBLANKING => Err(iCubeError::Unimplemented),
            Self::FGPA_HBLANKING => Err(iCubeError::Unimplemented),
            Self::FGPA_CLK_DIVIDER => Err(iCubeError::Unimplemented),
            Self::FGPA_ON_BOARD => Err(iCubeError::Unimplemented),
            Self::SET_GPIO => Err(iCubeError::Unimplemented),
            Self::GET_GPIO => Err(iCubeError::Unimplemented),
            Self::SET_GPIO_MODE => Err(iCubeError::Unimplemented),
            Self::MASK_ROI_ID => Err(iCubeError::Unimplemented),
            Self::RED_OFFSET => Err(iCubeError::Unimplemented),
            Self::GREEN_OFFSET => Err(iCubeError::Unimplemented),
            Self::BLUE_OFFSET => Err(iCubeError::Unimplemented),
            Self::HUE => Err(iCubeError::Unimplemented),
            Self::COLOR_CORRECTION => Err(iCubeError::Unimplemented),
            Self::GAMMA_ENABLE => Err(iCubeError::Unimplemented),
            Self::GAMMA_MODE => Err(iCubeError::Unimplemented),
            Self::INVERT_PIXEL => Ok(v1::REG_INVERT_PIXEL),
            Self::TNR => Err(iCubeError::Unimplemented),
            Self::BAYER_CONVERSION => Err(iCubeError::Unimplemented),
            Self::COLOR_PROCESSING => Err(iCubeError::Unimplemented),
            Self::USB_SPEED => Err(iCubeError::Unimplemented),
            Self::DEVICE_SPEED => Err(iCubeError::Unimplemented),
            Self::DATA_TRANSMISSION => Err(iCubeError::Unimplemented),
            Self::SIGNIFICANT_BITS => Err(iCubeError::Unimplemented),
            Self::GRAPHIC_MODE => Err(iCubeError::Unimplemented),
            Self::EDGE_ENHANCEMENT => Err(iCubeError::Unimplemented),
            Self::EDGE_ENHANCEMENT_GAIN => Err(iCubeError::Unimplemented),
            Self::SENSOR_STROBE => Err(iCubeError::Unimplemented),
            Self::TRIG_TIMEOUT => Err(iCubeError::Unimplemented),
            Self::PIPE_TIMEOUT_MODE => Err(iCubeError::Unimplemented),
            Self::RESET_CAMERA => Err(iCubeError::Unimplemented),
            Self::DISCONNECTIONS => Err(iCubeError::Unimplemented),
            Self::XACT_RECOVER_MODE => Err(iCubeError::Unimplemented),
            Self::RESET_TO_DEFAULT => Err(iCubeError::Unimplemented),
            Self::TRANSFER_BUF_CNT => Err(iCubeError::Unimplemented),
            Self::SW_TRIG_WD_MODE => Err(iCubeError::Unimplemented),
            Self::AQU_FRAMERATE => Err(iCubeError::Unimplemented),
            Self::SENSOR_STROBE_DELAY => Err(iCubeError::Unimplemented),
            Self::SENSOR_OVERLAPPED => Err(iCubeError::Unimplemented),
            Self::TRIG_DELAY_DIVIDER => Err(iCubeError::Unimplemented),
        }
    }

    pub fn into_sdk_value_v2(self) -> i32 {
        match self {
            Self::BRIGHTNESS => v2::REG_BRIGHTNESS,
            Self::CONTRAST => v2::REG_CONTRAST,
            Self::GAMMA => v2::REG_GAMMA,
            Self::FLIPPED_V => v2::REG_FLIPPED_V,
            Self::FLIPPED_H => v2::REG_FLIPPED_H,
            Self::WHITE_BALANCE => v2::REG_WHITE_BALANCE,
            Self::EXPOSURE_TIME => v2::REG_EXPOSURE_TIME,
            Self::EXPOSURE_TARGET => v2::REG_EXPOSURE_TARGET,
            Self::RED => v2::REG_RED,
            Self::GREEN => v2::REG_GREEN,
            Self::BLUE => v2::REG_BLUE,
            Self::BLACKLEVEL => v2::REG_BLACKLEVEL,
            Self::GAIN => v2::REG_GAIN,
            Self::COLOR => v2::REG_COLOR,
            Self::PLL => v2::REG_PLL,
            Self::STROBE_LENGHT => v2::REG_STROBE_LENGHT,
            Self::STROBE_DELAY => v2::REG_STROBE_DELAY,
            Self::TRIGGER_DELAY => v2::REG_TRIGGER_DELAY,
            Self::SATURATION => v2::REG_SATURATION,
            Self::COLOR_ENHANCE => v2::REG_COLOR_ENHANCE,
            Self::TRIGGER_INVERT => v2::REG_TRIGGER_INVERT,
            Self::RECONNECTIONS => v2::REG_RECONNECTIONS,
            Self::MEASURE_FIELD_AE => v2::REG_MEASURE_FIELD_AE,
            Self::AVI_STATE => v2::REG_AVI_STATE,
            Self::FOCUS => v2::REG_FOCUS,
            Self::SHUTTER => v2::REG_SHUTTER,
            Self::ROI_ID => v2::REG_ROI_ID,
            Self::ROI_CYCLE => v2::REG_ROI_CYCLE,
            Self::DEFECT_COR => v2::REG_DEFECT_COR,
            Self::BAD_FRAME => v2::REG_BAD_FRAME,
            Self::GOOD_FRAME => v2::REG_GOOD_FRAME,
            Self::SW_TRIG_MODE => v2::REG_SW_TRIG_MODE,
            Self::ROI_FPGA_ONE_FRAME => v2::REG_ROI_FPGA_ONE_FRAME,
            Self::CALLBACK_BR_FRAMES => v2::REG_CALLBACK_BR_FRAMES,
            Self::FGPA_VBLANKING => v2::REG_FGPA_VBLANKING,
            Self::FGPA_HBLANKING => v2::REG_FGPA_HBLANKING,
            Self::FGPA_CLK_DIVIDER => v2::REG_FGPA_CLK_DIVIDER,
            Self::FGPA_ON_BOARD => v2::REG_FGPA_ON_BOARD,
            Self::SET_GPIO => v2::REG_SET_GPIO,
            Self::GET_GPIO => v2::REG_GET_GPIO,
            Self::SET_GPIO_MODE => v2::REG_SET_GPIO_MODE,
            Self::MASK_ROI_ID => v2::REG_MASK_ROI_ID,
            Self::RED_OFFSET => v2::REG_RED_OFFSET,
            Self::GREEN_OFFSET => v2::REG_GREEN_OFFSET,
            Self::BLUE_OFFSET => v2::REG_BLUE_OFFSET,
            Self::HUE => v2::REG_HUE,
            Self::COLOR_CORRECTION => v2::REG_COLOR_CORRECTION,
            Self::GAMMA_ENABLE => v2::REG_GAMMA_ENABLE,
            Self::GAMMA_MODE => v2::REG_GAMMA_MODE,
            Self::INVERT_PIXEL => v2::REG_INVERT_PIXEL,
            Self::TNR => v2::REG_TNR,
            Self::BAYER_CONVERSION => v2::REG_BAYER_CONVERSION,
            Self::COLOR_PROCESSING => v2::REG_COLOR_PROCESSING,
            Self::USB_SPEED => v2::REG_USB_SPEED,
            Self::DEVICE_SPEED => v2::REG_DEVICE_SPEED,
            Self::DATA_TRANSMISSION => v2::REG_DATA_TRANSMISSION,
            Self::SIGNIFICANT_BITS => v2::REG_SIGNIFICANT_BITS,
            Self::GRAPHIC_MODE => v2::REG_GRAPHIC_MODE,
            Self::EDGE_ENHANCEMENT => v2::REG_EDGE_ENHANCEMENT,
            Self::EDGE_ENHANCEMENT_GAIN => v2::REG_EDGE_ENHANCEMENT_GAIN,
            Self::SENSOR_STROBE => v2::REG_SENSOR_STROBE,
            Self::TRIG_TIMEOUT => v2::REG_TRIG_TIMEOUT,
            Self::PIPE_TIMEOUT_MODE => v2::REG_PIPE_TIMEOUT_MODE,
            Self::RESET_CAMERA => v2::REG_RESET_CAMERA,
            Self::DISCONNECTIONS => v2::REG_DISCONNECTIONS,
            Self::XACT_RECOVER_MODE => v2::REG_XACT_RECOVER_MODE,
            Self::RESET_TO_DEFAULT => v2::REG_RESET_TO_DEFAULT,
            Self::TRANSFER_BUF_CNT => v2::REG_TRANSFER_BUF_CNT,
            Self::SW_TRIG_WD_MODE => v2::REG_SW_TRIG_WD_MODE,
            Self::AQU_FRAMERATE => v2::REG_AQU_FRAMERATE,
            Self::SENSOR_STROBE_DELAY => v2::REG_SENSOR_STROBE_DELAY,
            Self::SENSOR_OVERLAPPED => v2::REG_SENSOR_OVERLAPPED,
            Self::TRIG_DELAY_DIVIDER => v2::REG_TRIG_DELAY_DIVIDER,
        }
    }
}

pub struct ParameterProperty {
    pub enabled: bool,
    pub auto: bool,
    pub one_push: bool,
    pub default: u32,
    pub min: u32,
    pub max: u32,
}

impl ParameterProperty {
    fn from_sdk_struct_v1(sdk_struct: &v1::PARAM_PROPERTY) -> Self {
        Self {
            enabled: sdk_struct.bEnabled,
            auto: sdk_struct.bAuto,
            one_push: sdk_struct.bOnePush,
            default: sdk_struct.nDef,
            min: sdk_struct.nMin,
            max: sdk_struct.nMax,
        }
    }

    fn from_sdk_struct_v2(sdk_struct: &v2::PARAM_PROPERTY) -> Self {
        Self {
            enabled: sdk_struct.bEnabled != 0,
            auto: sdk_struct.bAuto != 0,
            one_push: sdk_struct.bOnePush != 0,
            default: sdk_struct.nDef,
            min: sdk_struct.nMin,
            max: sdk_struct.nMax,
        }
    }
}