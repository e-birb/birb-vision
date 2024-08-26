use std::{io, time::Duration};

use v4l::{buffer::Type, control::Description, io::traits::CaptureStream, prelude::*, Control};

fn main() -> io::Result<()> {
    let path = "/dev/video0";
    println!("Using device: {}\n", path);

    let mut dev = Device::with_path(path)?;
    let controls: Vec<Description> = dev.query_controls()?;
    println!("{} controls:", controls.len());

    for control in controls {
        let value = dev.control(control.id);
        println!("{}: {:?}\n", control, value);
    }

    let mut s = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)?;
    let j = std::thread::spawn(move || {
        while let Ok(frame) = s.next() {
            println!("frame: {:?}", frame.0.len());
            s.set_timeout(Duration::from_millis(1));
            std::thread::sleep(Duration::from_millis(10));
            println!("ERR: {}", s.next().err().unwrap());
        }
        println!("stream ended");
    });
    std::thread::sleep(std::time::Duration::from_secs(2));
    let s = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)?;
    drop(s);
    j.join().unwrap();

    Ok(())
}