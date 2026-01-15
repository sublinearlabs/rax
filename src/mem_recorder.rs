use std::{
    fs::File,
    io::{BufWriter, Write},
};

#[repr(u8)]
enum OpKind {
    Store = 1,
    Load = 2,
}

pub(crate) struct MemRecorder {
    out: BufWriter<File>,
}

impl MemRecorder {
    pub(crate) fn new(path: String) -> Self {
        let file = File::create(path).unwrap();
        let out = BufWriter::new(file);
        Self { out }
    }

    #[inline]
    fn write_store(&mut self, addr: u64, width: u8, value_le: &[u8]) {
        debug_assert!(value_le.len() == width as usize);

        // HEADER: [op, width] + addr
        let mut header = [0_u8; 10];
        header[0] = OpKind::Store as u8;
        header[1] = width;
        header[2..10].copy_from_slice(&addr.to_le_bytes());

        // Write the header + value bytes
        self.out.write_all(&header).unwrap();
        self.out.write_all(value_le).unwrap();
    }

    #[inline]
    fn write_load(&mut self, addr: u64, width: u8) {
        let mut header = [0_u8; 10];
        header[0] = OpKind::Store as u8;
        header[1] = width;
        header[2..10].copy_from_slice(&addr.to_le_bytes());

        self.out.write_all(&header).unwrap();
    }

    pub(crate) fn store_u8(&mut self, addr: u64, value: u8) {
        self.write_store(addr, 1, &[value]);
    }

    pub(crate) fn store_u16(&mut self, addr: u64, value: u16) {
        let val_bytes = value.to_le_bytes();
        self.write_store(addr, val_bytes.len() as u8, &val_bytes);
    }

    pub(crate) fn store_u32(&mut self, addr: u64, value: u32) {
        let val_bytes = value.to_le_bytes();
        self.write_store(addr, val_bytes.len() as u8, &val_bytes);
    }

    pub(crate) fn store_u64(&mut self, addr: u64, value: u64) {
        let val_bytes = value.to_le_bytes();
        self.write_store(addr, val_bytes.len() as u8, &val_bytes);
    }

    pub(crate) fn load_u8(&mut self, addr: u64) {
        self.write_load(addr, 1);
    }

    pub(crate) fn load_u16(&mut self, addr: u64) {
        self.write_load(addr, 2);
    }
    pub(crate) fn load_u32(&mut self, addr: u64) {
        self.write_load(addr, 4);
    }

    pub(crate) fn load_u64(&mut self, addr: u64) {
        self.write_load(addr, 8);
    }
}
