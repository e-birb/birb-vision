use birb_vision_daheng::Ctx;

fn main() {
    let ctx = Ctx::new().unwrap();
    let n = ctx.get_all_device_base_info().unwrap().len();
    println!("Found {n} devices");
}