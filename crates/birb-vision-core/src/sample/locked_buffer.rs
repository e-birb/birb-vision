use std::borrow::Cow;

use super::FlatSample;


pub trait LockedBuffer: Send + Sync {
    fn sample(&self) -> FlatSample<Cow<[u8]>>;
}