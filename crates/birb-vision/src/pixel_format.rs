
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PixelFormat {
    Mono(u8),
    MonoPacked(u8),
    Bayer(BayerPixelFormat),
    BayerPacked(BayerPixelFormat),
    Packed(PackedPixelFormat),
    YUV(u8),

    // TODO ... see C:\Program Files (x86)\MVS\Development\Includes\PixelType.h
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BayerPixelFormat {
    RG(u8),
    RB(u8),
    GR(u8),
    GB(u8),
    BR(u8),
    BG(u8),
    // TODO ...
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PackedPixelFormat {
    RGB(u8),
    BGR(u8),
    RGBA(u8),
    BGRA(u8),
    RGB565,
    BGR565,
    // TODO ...
    // TODO V1 formats???
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum YUVPixelFormat {
    // TODO ...
}