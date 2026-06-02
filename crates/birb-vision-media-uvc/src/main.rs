

fn main() {
    #[cfg(target_os = "linux")]
    {
        use uvc::Context;
        let ctx = Context::new().expect("Could not create context");

        let _devices = ctx.devices();
    }
}