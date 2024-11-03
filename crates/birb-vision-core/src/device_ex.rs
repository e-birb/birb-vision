use std::{future::Future, sync::Mutex};

use futures::channel::oneshot;

use crate::{CameraDevice, DeviceResult, StreamEvent, Sample};

impl<T: CameraDevice + ?Sized> CameraDeviceEx for T {}

pub trait CameraDeviceEx: CameraDevice {
    // TODO timeout
    fn get_one_frame<'a>(&'a self) -> impl Future<Output = DeviceResult<Sample<'static>>> + 'a {
        async move {
            let (tx, rx) = oneshot::channel();
            let tx = Mutex::new(Some(tx));

            self.set_stream_callback(Box::new(move |event| {
                match event {
                    StreamEvent::Sample(frame) => {
                        if let Some(tx) = tx.lock().unwrap().take() {
                            if let Err(e) = tx.send(frame.map(|s| s.into_owned())) {
                                log::error!("Error sending frame: {:?}", e);
                            }
                        }
                    },
                    _ => {},
                }
            }))?;

            self.grab()?;

            let frame_result = rx.await.map_err(|e| anyhow::Error::from(e))?;

            Ok(frame_result?)
        }
    }
}