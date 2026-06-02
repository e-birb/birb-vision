use std::time::Duration;

use birb_vision::all_backends;
use birb_vision_core::CameraDeviceEx;

fn main() -> anyhow::Result<()> { pollster::block_on(async {

    for (_, pkg) in all_backends().all_packages() {
        let ctx = pkg.build_backend()?;
        for info in ctx.enumerate(&ctx.default_transport_layers())? {
            let device = ctx.create(&info)?.unwrap();
            device.start_grabbing()?;
            let frame = device.get_one_frame(Duration::from_secs(2)).await?;
            frame.try_decode().unwrap()?.save("frame.png")?;
        }
    }

    Ok(())
}) }