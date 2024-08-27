
use birb_vision_core::backend::{BackendPackage, BackendRegistry};
use birb_vision_mvs::MVContext;

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

    set
}