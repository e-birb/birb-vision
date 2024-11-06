use std::{future::Future, sync::Mutex, time::Duration};

use anyhow::anyhow;
use futures::{channel::oneshot, select};

use crate::{CameraDevice, DeviceError, DeviceResult, Sample, StreamEvent};

impl<T: CameraDevice + ?Sized> CameraDeviceEx for T {}

pub trait CameraDeviceEx: CameraDevice {
    // TODO timeout
    fn get_one_frame<'a>(&'a self, timeout: Duration) -> impl Future<Output = DeviceResult<Sample<'static>>> + 'a {
        async move {
            let (tx, rx) = oneshot::channel();
            let tx = Mutex::new(Some(tx));

            self.set_stream_callback(Box::new(move |event| {
                //log::warn!("Event: {:?}", event);
                match event {
                    StreamEvent::Sample(frame) => {
                        if let Some(tx) = tx.lock().unwrap().take() {
                            if let Err(e) = tx.send(frame.map(|s| s.into_owned())) {
                                log::error!("Error sending frame: {:?}", e);
                            } else {
                                log::warn!("Frame sent");
                            }
                        }
                    },
                    _ => {},
                }
            }))?;

            if let Err(e) = self.grab() {
                log::error!("Error sending grab: {e}");
            }

            //let frame_result = rx.await.map_err(|e| anyhow::Error::from(e))?;
            let frame_result = async_std::future::timeout(timeout, async {
                rx.await.map_err(|e| anyhow!("Failed to receive frame: {e}"))
            }).await.map_err(|_| DeviceError::Timeout)??;

            Ok(frame_result?)
        }
    }
}