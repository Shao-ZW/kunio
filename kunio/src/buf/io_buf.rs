pub trait IoBuf {
    fn read_ptr(&self) -> *const u8;
    fn size(&self) -> u32;
}

impl IoBuf for Vec<u8> {
    fn read_ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    fn size(&self) -> u32 {
        self.len() as u32
    }
}

impl IoBuf for Box<[u8]> {
    fn read_ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    fn size(&self) -> u32 {
        self.len() as u32
    }
}

impl IoBuf for &'static [u8] {
    fn read_ptr(&self) -> *const u8 {
        self.as_ptr()
    }

    fn size(&self) -> u32 {
        self.len() as u32
    }
}

pub trait IoBufMut: IoBuf {
    fn write_ptr(&mut self) -> *mut u8;
}

impl IoBufMut for Vec<u8> {
    fn write_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr()
    }
}

impl IoBufMut for Box<[u8]> {
    fn write_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr()
    }
}
