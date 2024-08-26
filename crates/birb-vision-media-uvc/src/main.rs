use uvc::Context;


fn main() {
    let ctx = Context::new().expect("Could not create context");

    let devices = ctx.devices();
}