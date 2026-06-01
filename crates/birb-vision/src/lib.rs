pub use birb_vision_core as core;

mod backend;

pub use backend::*;

pub fn all_backends() -> BackendRegistry {
    let mut set = BackendRegistry::new();

    #[cfg(feature = "mvs")]
    set.register(
        BackendPackage::from_builder_fn(|| {
            let ctx = birb_vision_mvs::MVContext::new(None)?;
            Ok(ctx)
        })
        .with_display_name("Hikrobot (MVS SDK)")
    ).unwrap();

    #[cfg(feature = "v4l")]
    set.register(
        BackendPackage::from_builder_fn(|| {
            let ctx = birb_vision_v4l::V4lContext::new();
            Ok(ctx)
        })
        .with_display_name("Video4Linux (v4l)")
    ).unwrap();

    #[cfg(feature = "icube")]
    set.register(
        BackendPackage::from_builder_fn(|| {
            let ctx = birb_vision_icube::iCubeContext::new()?;
            Ok(ctx)
        })
        .with_display_name("iCube (NET CAMERA)")
    ).unwrap();

    #[cfg(feature = "media-foundation")]
    set.register(
        BackendPackage::from_builder_fn(|| {
            let ctx = birb_vision_media_foundation::MediaFoundationContext::new()
                .map_err(|e| anyhow::anyhow!("Failed to create MediaFoundationContext: {e}"))?;
            Ok(ctx)
        })
        .with_display_name("Media Foundation")
    ).unwrap();

    #[cfg(feature = "daheng")]
    set.register(
        BackendPackage::from_builder_fn(|| {
            let ctx = birb_vision_daheng::Ctx::new()
                .map_err(|e| anyhow::anyhow!("Failed to create DahengContext: {e}"))?;
            Ok(ctx)
        })
        .with_display_name("Daheng")
    ).unwrap();

    #[cfg(feature = "directshow")]
    set.register(
        BackendPackage::from_builder_fn(|| {
            let ctx = birb_vision_directshow::DirectShowContext::new()
                .map_err(|e| anyhow::anyhow!("Failed to create DirectShowContext: {e}"))?;
            Ok(ctx)
        })
        .with_display_name("DirectShow")
    ).unwrap();

    set
}