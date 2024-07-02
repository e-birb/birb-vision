use std::fmt::Debug;

use windows::{core::GUID, Win32::Media::MediaFoundation::*};

use crate::MFResult;

#[derive(Debug, Clone)]
pub struct VideoFormatQuery {
    pub must_be_supported: bool,
    pub video_subtype: Option<Vec<VideoSubtype>>,
    pub resolutions: Option<Vec<(u32, u32)>>,
    pub frame_rates: Option<Vec<Framerate>>,
}

impl From<()> for VideoFormatQuery {
    fn from(_: ()) -> Self {
        Self::any_supported_format()
    }
}

impl From<&VideoFormat> for VideoFormatQuery {
    fn from(format: &VideoFormat) -> Self {
        if let Some(subtype) = format.recognize_supported_media_subtype() {
            Self {
                must_be_supported: true,
                video_subtype: Some(vec![subtype]),
                resolutions: Some(vec![(format.width(), format.height())]),
                frame_rates: Some(vec![format.frame_rate()]),
            }
        } else {
            Self {
                must_be_supported: false,
                video_subtype: None,
                resolutions: Some(vec![(format.width(), format.height())]),
                frame_rates: Some(vec![format.frame_rate()]),
            }
        }
    }
}

impl From<VideoFormat> for VideoFormatQuery {
    fn from(format: VideoFormat) -> Self {
        Self::from(&format)
    }
}

impl From<&VideoFormatQuery> for VideoFormatQuery {
    fn from(query: &VideoFormatQuery) -> Self {
        query.clone()
    }
}

impl VideoFormatQuery {
    pub fn any_supported_format() -> Self {
        Self {
            must_be_supported: true,
            video_subtype: None,
            resolutions: None,
            frame_rates: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VideoFormat {
    //subtype: GUID,
    //video_format: GUID,
    /// [`MF_MT_SUBTYPE`]
    media_subtype: GUID,
    width: u32,
    height: u32,
    stride: Option<i32>,
    frame_rate: Framerate,
}

impl Debug for VideoFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoFormat")
            .field("media_subtype", &video_format_name(&self.media_subtype))
            .field("width", &self.width)
            .field("height", &self.height)
            .field("stride", &self.stride)
            .field("frame_rate", &self.frame_rate)
            .finish()
    }
}

impl VideoFormat {
    pub fn from_media_type(media_type: &IMFMediaType) -> MFResult<Self> {
        let media_subtype = unsafe { media_type.GetGUID(&MF_MT_SUBTYPE)? };

        let (width, height) = unsafe { media_type.GetUINT64(&MF_MT_FRAME_SIZE) }
            .map(|s| {
                let width = (s >> 32) as u32;
                let height = s as u32; // the cast will truncate the upper bits
                (width, height)
            })?;

        let stride = unsafe { media_type.GetUINT32(&MF_MT_DEFAULT_STRIDE) }
            .ok()
            .map(|stride| unsafe { std::mem::transmute(stride) });

        let fraction_u64 = unsafe { media_type.GetUINT64(&MF_MT_FRAME_RATE)? };
        let frame_rate = Framerate {
            numerator: (fraction_u64 >> 32) as u32,
            denominator: fraction_u64 as u32,
        };

        Ok(Self {
            media_subtype,
            width,
            height,
            stride,
            frame_rate,
        })
    }

    /// Lists compatible video formats for the given media type.
    ///
    /// Note that the returned formats only differ in the frame rate.
    pub fn list(media_type: &IMFMediaType) -> MFResult<Vec<Self>> {
        let media_subtype = unsafe { media_type.GetGUID(&MF_MT_SUBTYPE)? };

        let (width, height) = unsafe { media_type.GetUINT64(&MF_MT_FRAME_SIZE) }
            .map(|s| {
                let width = (s >> 32) as u32;
                let height = s as u32; // the cast will truncate the upper bits
                (width, height)
            })?;

        let stride = unsafe { media_type.GetUINT32(&MF_MT_DEFAULT_STRIDE) }
            .ok()
            .map(|stride| unsafe { std::mem::transmute(stride) });

        let mut list = vec![];

        const FRAMERATE_QUERIES: &[GUID] = &[
            MF_MT_FRAME_RATE_RANGE_MAX,
            MF_MT_FRAME_RATE,
            MF_MT_FRAME_RATE_RANGE_MIN,
        ];

        for framerate_query in FRAMERATE_QUERIES {
            if let Ok(fraction_u64) = unsafe { media_type.GetUINT64(framerate_query) } {
                let frame_rate = Framerate {
                    numerator: (fraction_u64 >> 32) as u32,
                    denominator: fraction_u64 as u32,
                };

                let format = Self {
                    media_subtype,
                    width,
                    height,
                    stride,
                    frame_rate,
                };

                let new = list.iter().all(|f| f != &format);

                if new {
                    list.push(format);
                } else {
                    if !new {
                        log::trace!(
                            "Ignoring duplicate frame rate for media type {:?}",
                            video_format_name(&media_subtype)
                        );
                    }
                }
            } else {
                log::warn!(
                    "Failed to get frame rate for media type {:?}",
                    video_format_name(&media_subtype)
                );
            }
        }

        Ok(list)
    }

    pub fn media_subtype(&self) -> GUID {
        self.media_subtype
    }

    /// Try to recognize the media subtype.
    ///
    /// This will return `None` if the subtype is not **supported** by this crate.
    pub fn recognize_supported_media_subtype(&self) -> Option<VideoSubtype> {
        VideoSubtype::try_recognize(&self.media_subtype)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn stride(&self) -> Option<i32> {
        self.stride
    }

    pub fn frame_rate(&self) -> Framerate {
        self.frame_rate
    }

    pub fn satisfies(&self, query: &VideoFormatQuery) -> bool {
        if let Some(subtype) = self.recognize_supported_media_subtype() {
            if let Some(video_subtype) = &query.video_subtype {
                if !video_subtype.contains(&subtype) {
                    return false;
                }
            }
        } else {
            if query.must_be_supported {
                return false;
            }
        }

        if let Some(resolutions) = &query.resolutions {
            if !resolutions.contains(&(self.width, self.height)) {
                return false;
            }
        }

        if let Some(frame_rates) = &query.frame_rates {
            if !frame_rates.contains(&self.frame_rate) {
                return false;
            }
        }

        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Framerate {
    pub numerator: u32,
    pub denominator: u32,
}

impl Framerate {
    pub fn as_f32(&self) -> f32 {
        self.numerator as f32 / self.denominator as f32
    }
}

impl std::fmt::Display for Framerate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

impl Debug for Framerate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Framerate")
            .field("numerator", &self.numerator)
            .field("denominator", &self.denominator)
            .field("as_f32", &self.as_f32())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VideoSubtype {
    Uncompressed(PixelFormat),
    CompressedFrame(CompressedFrame),
}

impl VideoSubtype {
    #[allow(non_upper_case_globals)]
    pub fn try_recognize(subtype: &GUID) -> Option<Self> {
        match subtype {
            &MFVideoFormat_RGB24 | &MEDIASUBTYPE_RGB24 => Some(Self::Uncompressed(PixelFormat::RGB24)),
            &MFVideoFormat_RGB32 | &MEDIASUBTYPE_RGB32 => Some(Self::Uncompressed(PixelFormat::RGB32)),
            &MFVideoFormat_YUY2 /*| &MEDIASUBTYPE_YUY2*/ => Some(Self::Uncompressed(PixelFormat::YUY2)),
            &MFVideoFormat_NV12 /*| &MEDIASUBTYPE_NV12*/ => Some(Self::Uncompressed(PixelFormat::NV12)),
            &MFVideoFormat_MJPG /*| &MEDIASUBTYPE_MJPG*/ => Some(Self::CompressedFrame(CompressedFrame::MJpeg)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormat {
    /// RGB 24
    ///
    /// FourCC:
    /// - 0x00000014 ([`D3DFMT_R8G8B8`])
    ///
    /// [`GUID`]:
    /// - `00000014-0000-0010-8000-00AA00389B71` [`MFVideoFormat_RGB24`]
    /// - `E436EB7D-524F-11CE-9F53-0020AF0BA770` [`MEDIASUBTYPE_RGB24`]
    RGB24,

    /// RGB 32
    ///
    /// FourCC:
    /// - 0x00000014 ([`D3DFMT_X8R8G8B8`])
    ///
    /// [`GUID`]:
    /// - `00000016-0000-0010-8000-00AA00389B71` [`MFVideoFormat_RGB32`]
    /// - `E436EB7E-524F-11CE-9F53-0020AF0BA770` [`MEDIASUBTYPE_RGB32`]
    RGB32,

    /// YUY
    ///
    /// FourCC:
    /// - 0x32595559 [`YUY2`]
    ///
    /// [`GUID`]:
    /// - `32595559-0000-0010-8000-00AA00389B71` [`MFVideoFormat_YUY2`]
    /// - `32595559-0000-0010-8000-00AA00389B71` [`MEDIASUBTYPE_YUY2`]
    YUY2,

    /// NV12
    ///
    /// FourCC:
    /// - 0x3231564E [`NV12`]
    ///
    /// [`GUID`]:
    /// - `3231564E-0000-0010-8000-00AA00389B71` [`MFVideoFormat_NV12`]
    /// - `3231564E-0000-0010-8000-00AA00389B71` [`MEDIASUBTYPE_NV12`]
    NV12,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompressedFrame {
    /// Motion JPEG
    ///
    /// FourCC:
    /// - 0x47504A4D [`MJPG`]
    ///
    /// [`GUID`]:
    /// - `47504A4D-0000-0010-8000-00AA00389B71` [`MFVideoFormat_MJPG`]
    /// - `47504A4D-0000-0010-8000-00AA00389B71` [`MEDIASUBTYPE_MJPG`]
    MJpeg,
}

macro_rules! video_formats {
    (
        $($name:ident,)*
    ) => {
        fn video_format_name(guid: &GUID) -> String {
            #[allow(non_upper_case_globals)]
            #[allow(unreachable_patterns)]
            #[allow(unused_variables)]
            #[allow(non_snake_case)]
            match guid {
                $(
                    &$name => format!("{} ({:#034X})", stringify!($name), guid.to_u128()),
                )*
                _ => format!("{:#034X}", guid.to_u128()),
            }
        }
    };
}

video_formats! {
    MFVideoFormat_420O,
    MFVideoFormat_A16B16G16R16F,
    MFVideoFormat_A2R10G10B10,
    MFVideoFormat_AI44,
    MFVideoFormat_ARGB32,
    MFVideoFormat_AV1,
    MFVideoFormat_AYUV,
    MFVideoFormat_Base,
    MFVideoFormat_Base_HDCP,
    MFVideoFormat_D16,
    MFVideoFormat_DV25,
    MFVideoFormat_DV50,
    MFVideoFormat_DVH1,
    MFVideoFormat_DVHD,
    MFVideoFormat_DVSD,
    MFVideoFormat_DVSL,
    MFVideoFormat_H263,
    MFVideoFormat_H264,
    MFVideoFormat_H264_ES,
    MFVideoFormat_H264_HDCP,
    MFVideoFormat_H265,
    MFVideoFormat_HEVC,
    MFVideoFormat_HEVC_ES,
    MFVideoFormat_HEVC_HDCP,
    MFVideoFormat_I420,
    MFVideoFormat_IYUV,
    MFVideoFormat_L16,
    MFVideoFormat_L8,
    MFVideoFormat_M4S2,
    MFVideoFormat_MJPG,
    MFVideoFormat_MP43,
    MFVideoFormat_MP4S,
    MFVideoFormat_MP4V,
    MFVideoFormat_MPEG2,
    MFVideoFormat_MPG1,
    MFVideoFormat_MSS1,
    MFVideoFormat_MSS2,
    MFVideoFormat_NV11,
    MFVideoFormat_NV12,
    MFVideoFormat_NV21,
    MFVideoFormat_ORAW,
    MFVideoFormat_P010,
    MFVideoFormat_P016,
    MFVideoFormat_P210,
    MFVideoFormat_P216,
    MFVideoFormat_RGB24,
    MFVideoFormat_RGB32,
    MFVideoFormat_RGB555,
    MFVideoFormat_RGB565,
    MFVideoFormat_RGB8,
    MFVideoFormat_Theora,
    MFVideoFormat_UYVY,
    MFVideoFormat_VP10,
    MFVideoFormat_VP80,
    MFVideoFormat_VP90,
    MFVideoFormat_WMV1,
    MFVideoFormat_WMV2,
    MFVideoFormat_WMV3,
    MFVideoFormat_WVC1,
    MFVideoFormat_Y210,
    MFVideoFormat_Y216,
    MFVideoFormat_Y410,
    MFVideoFormat_Y416,
    MFVideoFormat_Y41P,
    MFVideoFormat_Y41T,
    MFVideoFormat_Y42T,
    MFVideoFormat_YUY2,
    MFVideoFormat_YV12,
    MFVideoFormat_YVU9,
    MFVideoFormat_YVYU,
    MFVideoFormat_v210,
    MFVideoFormat_v216,
    MFVideoFormat_v410,

    MEDIASUBTYPE_RGB1,
    MEDIASUBTYPE_RGB4,
    MEDIASUBTYPE_RGB8,
    MEDIASUBTYPE_ARGB4444,
    MEDIASUBTYPE_RGB565,
    MEDIASUBTYPE_RGB555,
    MEDIASUBTYPE_ARGB1555,
    MEDIASUBTYPE_RGB24,
    MEDIASUBTYPE_RGB32,
    MEDIASUBTYPE_ARGB32,
    MEDIASUBTYPE_A2R10G10B10,
    MEDIASUBTYPE_A2B10G10R10,
    MEDIASUBTYPE_AYUV,
    MEDIASUBTYPE_UYVY,
    MEDIASUBTYPE_YUY2,
    MEDIASUBTYPE_YUYV,
    MEDIASUBTYPE_YVYU,
    MEDIASUBTYPE_Y411,
    MEDIASUBTYPE_Y41P,
    MEDIASUBTYPE_I420,
    MEDIASUBTYPE_IYUV,
    MEDIASUBTYPE_NV11,
    MEDIASUBTYPE_NV12,
    MEDIASUBTYPE_NV21,
    MEDIASUBTYPE_IMC1,
    MEDIASUBTYPE_IMC2,
    MEDIASUBTYPE_IMC3,
    MEDIASUBTYPE_IMC4,
    MEDIASUBTYPE_YV12,
    MEDIASUBTYPE_YVU9,
    MEDIASUBTYPE_Y211,
    MEDIASUBTYPE_AI44,
    MEDIASUBTYPE_IA44,
    MEDIASUBTYPE_IF09,
    MEDIASUBTYPE_Y41T,
    MEDIASUBTYPE_Y42T,
    MEDIASUBTYPE_P208,
    MEDIASUBTYPE_P408,
    MEDIASUBTYPE_NV24,
    MEDIASUBTYPE_P010,
    MEDIASUBTYPE_P016,
    MEDIASUBTYPE_P210,
    MEDIASUBTYPE_P216,
    MEDIASUBTYPE_v210,
    MEDIASUBTYPE_V216,
    MFVideoFormat_v216,
    MEDIASUBTYPE_V410,
    MEDIASUBTYPE_v410,
    MEDIASUBTYPE_Y210,
    MEDIASUBTYPE_Y216,
    MEDIASUBTYPE_RGB16_D3D_DX7_RT,
    MEDIASUBTYPE_RGB32_D3D_DX7_RT,
    MEDIASUBTYPE_ARGB32_D3D_DX7_RT,
    MEDIASUBTYPE_ARGB1555_D3D_DX7_RT,
    MEDIASUBTYPE_ARGB4444_D3D_DX7_RT,
    MEDIASUBTYPE_RGB16_D3D_DX9_RT,
    MEDIASUBTYPE_RGB32_D3D_DX9_RT,
    MEDIASUBTYPE_ARGB32_D3D_DX9_RT,
    MEDIASUBTYPE_ARGB1555_D3D_DX9_RT,
    MEDIASUBTYPE_ARGB4444_D3D_DX9_RT,
    MEDIASUBTYPE_V422,
    MEDIASUBTYPE_Y41B,
    MEDIASUBTYPE_PAL1,
    MEDIASUBTYPE_PAL4,
    MEDIASUBTYPE_PAL8,
    MEDIASUBTYPE_RGB2,
    MEDIASUBTYPE_RGB3,
    MEDIASUBTYPE_RGB5,
    MEDIASUBTYPE_RGB6,
    MEDIASUBTYPE_CLJR,
    MEDIASUBTYPE_CLPL,
    MEDIASUBTYPE_CPLA,
    MEDIASUBTYPE_422P,
    MEDIASUBTYPE_444P,
    MEDIASUBTYPE_411P,
    MEDIASUBTYPE_410P,
    MEDIASUBTYPE_VYUY,
    MEDIASUBTYPE_Y800,
    MEDIASUBTYPE_YV16,
    MEDIASUBTYPE_YV24,
}