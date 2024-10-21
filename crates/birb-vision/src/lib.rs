use birb_vision_core::backend::BackendPackage;
pub use birb_vision_core as core;

use core::backend::BackendRegistry;

pub fn all_backends() -> BackendRegistry {
    let set = BackendRegistry::new();

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
        .with_display_name("Hikrobot (MVS SDK)")
    ).unwrap();

    set
}