
use crate::DeviceResult;

pub trait DeviceIO {
    unsafe fn read_register<'a>(
        &'a mut self,
        address: u64,
        buffer: &'a mut [u8],
    ) -> DeviceResult;

    unsafe fn write_register<'a>(
        &'a mut self,
        address: u64,
        buffer: &'a [u8],
    ) -> DeviceResult;
}

// TODO make this interface async by returning DeviceResult<Box<dyn AsyncWrite + 'a>>, polling
// or using some similar mechanism to handle async writes/reads.