
use birb_vision_core::backend::{BackendPackage, BackendRegistry};
use birb_vision_mvs::MVContext;
use birb_vision_v4l::V4lContext;

pub fn all_backends() -> BackendRegistry {
    let set = BackendRegistry::new();

    #[cfg(feature = "mvs")]
    set.register(
        BackendPackage::from_builder_fn(|| {
            let ctx = MVContext::new(None)?;
            Ok(ctx)
        })
        .with_display_name("Hikrobot (MVS SDK)")
    ).unwrap();

    #[cfg(feature = "v4l")]
    set.register(
        BackendPackage::from_builder_fn(|| {
            let ctx = V4lContext::new();
            Ok(ctx)
        })
        .with_display_name("Video4Linux (v4l)")
    ).unwrap();

    set
}