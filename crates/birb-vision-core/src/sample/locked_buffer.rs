
pub trait LockedBuffer: Send + Sync {
    fn data(&self) -> &[u8];
}