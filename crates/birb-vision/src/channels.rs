use std::{any::Any, pin::Pin, sync::{Arc, Mutex, Weak}};

use futures::{stream::BoxStream, Stream, StreamExt};


#[derive(Clone)]
pub struct CallbackTx<F: 'static + ?Sized> {
    callback: Weak<Box<F>>, // Note it is inside a mutex so FnMut is acceptable
}

impl<F: ?Sized> CallbackTx<F> {
    pub fn new(f: Box<F>) -> (Self, CallbackHandle) {
        let f = f.into();
        let cb = Arc::new(f);
        let weak = Arc::downgrade(&cb);
        (Self { callback: weak }, CallbackHandle { cb })
    }

    pub fn connected(&self) -> bool {
        self.callback.strong_count() > 0
    }

    pub fn try_call<R>(&self, f: impl FnOnce(&F) -> R) -> Option<R> {
        self.callback.upgrade().map(|cb| f(&cb))
    }
}

#[derive(Clone)]
pub struct CallbackHandle {
    cb: Arc<dyn Any>,
}

/*impl<T: 'static> CallbackHandle<T> {
    pub fn set_callback(&mut self, f: impl FnMut(T) + Send + Sync + 'static) {
        let mut cb = self.cb.lock().unwrap();
        *cb = Some(Box::new(f));
    }

    pub fn into_buffered_stream<R: Send + Sync + 'static>(mut self, buffer: usize, mut f: impl FnMut(T) -> R + Send + Sync + 'static) -> StreamChannel<R> {
        let (mut tx, rx) = futures::channel::mpsc::channel(buffer);
        self.set_callback(move |frame| {
            tx.try_send(f(frame)).unwrap(); // TODO handle error
        });

        StreamChannel {
            _callback: Box::new(self),
            rx: Box::pin(rx),
        }
    }

    // TODO map, filter, etc
}*/

pub struct StreamChannel<T> {
    /// Keep the callback alive
    _callback: Box<dyn Any + Send + Sync>,
    rx: BoxStream<'static, T>,
}

impl<T> Stream for StreamChannel<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}

impl<T> StreamChannel<T> {
}