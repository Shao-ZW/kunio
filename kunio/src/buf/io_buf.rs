pub trait IoBuf {
    fn read_ptr(&self) -> *const u8;
    fn valid_len(&self) -> u32;
}

impl IoBuf for Vec<u8> {
    fn read_ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    fn valid_len(&self) -> u32 {
        self.len() as u32
    }
}

impl IoBuf for Box<[u8]> {
    fn read_ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    fn valid_len(&self) -> u32 {
        self.len() as u32
    }
}

impl IoBuf for &'static [u8] {
    fn read_ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    fn valid_len(&self) -> u32 {
        self.len() as u32
    }
}

pub trait IoBufMut {
    fn write_ptr(&mut self) -> *mut u8;
    fn available_len(&self) -> u32;
    unsafe fn set_valid_len(&mut self, size: u32);
}

impl IoBufMut for Vec<u8> {
    fn write_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr()
    }

    fn available_len(&self) -> u32 {
        self.capacity() as u32
    }

    unsafe fn set_valid_len(&mut self, size: u32) {
        unsafe {
            self.set_len(size as usize);
        }
    }
}
