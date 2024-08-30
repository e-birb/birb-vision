
macro_rules! def_pixel_format {
    (
        $(#[$enum_attr:meta])*
        $enum_name:ident {$(
        $(#[$attr:meta])*
        $name:ident : $bpp:literal,
    )*}) => {
        $(#[$enum_attr])*
        pub enum $enum_name { $(
                $(#[$attr])*
                $name,
        )*}

        impl $enum_name {
            /// Bits per pixel
            pub fn bpp(&self) -> u8 {
                match self {
                    $(
                        $enum_name::$name => $bpp,
                    )*
                }
            }
        }
    };
}

def_pixel_format! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[allow(non_camel_case_types)]
    PixelFormat {
        Mono1Packed:1,
        Mono2Packed:2,
        Mono4Packed:4,
        Mono8Packed:8,
        Mono8Signed:8,
        Mono10:16,
        Mono10Packed:16,
        Mono12:16,
        Mono12Packed:16,
        Mono14:16,
        Mono16:16,

        // Bayer formats

        BayerGR8:8,
        BayerRG8:8,
        BayerGB8:8,
        BayerBG8:8,
        BayerRBGG8:8,
        BayerGR10:16,
        BayerRG10:16,
        BayerGB10:16,
        BayerBG10:16,
        BayerGR12:16,
        BayerRG12:16,
        BayerGB12:16,
        BayerBG12:16,
        BayerGR10Packed:12,
        BayerRG10Packed:12,
        BayerGB10Packed:12,
        BayerBG10Packed:12,
        BayerGR12Packed:12,
        BayerRG12Packed:12,
        BayerGB12Packed:12,
        BayerBG12Packed:12,
        BayerGR16:16,
        BayerRG16:16,
        BayerGB16:16,
        BayerBG16:16,

        // RGB formats

        RGB8Packed:24,
        BGR8Packed:24,
        RGBA8Packed:32,
        BGRA8Packed:32,
        RGB10Packed:48,
        BGR10Packed:48,
        RGB12Packed:48,
        BGR12Packed:48,
        RGB16Packed:48,
        BGR16Packed:48,
        RGBA16Packed:64,
        BGRA16Packed:64,
        RGB10V1Packed:32,
        RGB10V2Packed:32,
        RGB12V1Packed:36,
        RGB565Packed:16,
        BGR565Packed:16,

        YUV411Packed:12,
        YUV422Packed:16,
        YUV422YUYVPacked:16,
        YUV444Packed:24,
        YCBCR8_CBYCR:24, // TODO fix from now on
        YCBCR422_8:16,
        YCBCR422_8_CBYCRY:16,
        YCBCR411_8_CBYYCRYY:12,
        YCBCR601_8_CBYCR:24,
        YCBCR601_422_8:16,
        YCBCR601_422_8_CBYCRY:16,
        YCBCR601_411_8_CBYYCRYY:12,
        YCBCR709_8_CBYCR:24,
        YCBCR709_422_8:16,
        YCBCR709_422_8_CBYCRY:16,
        YCBCR709_411_8_CBYYCRYY:12,

        // TODO ... see "/media/luca/Acer/Program Files (x86)/MVS/Development/Includes/PixelType.h"
    }
}

