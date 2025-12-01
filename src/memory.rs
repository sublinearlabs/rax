use std::{char::MAX, collections::HashMap};

/// Number of bits to describe entries in a page
const PAGE_SHIFT: u64 = 12;
/// Total number of entries in a page
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
/// Mask to get the last `PAGE_SHIFT` bits of an address
const PAGE_MASK: u64 = (PAGE_SIZE as u64) - 1;
/// Max memory address
const MAX_ADDR: u64 = u64::MAX - 1;

type Page = Box<[u8; PAGE_SIZE]>;

#[derive(Default)]
pub struct Memory {
    pages: HashMap<u64, Page>,
}

impl Memory {
    /// Return the page index given the address
    #[inline]
    fn page_idx(addr: u64) -> u64 {
        // addr = [PAGE_ID][PAGE_SHIFT]
        addr >> PAGE_SHIFT
    }

    /// Return the entry index within a page
    /// given an address
    #[inline]
    fn page_offset(addr: u64) -> usize {
        (addr & PAGE_MASK) as usize
    }

    /// Returns a mutable reference to a page given an address
    /// lazy allocates the page if needed
    #[inline]
    fn ensure_page(&mut self, idx: u64) -> &mut Page {
        self.pages
            .entry(idx)
            .or_insert_with(|| Box::new([0; PAGE_SIZE]))
    }

    /// Read a single byte. Defaults to 0 if page doesn't exist
    fn read_u8(&self, addr: u64) -> u8 {
        if addr > MAX_ADDR {
            panic!("read out of range: 0x{:x}", addr);
        }
        let idx = Self::page_idx(addr);
        let offset = Self::page_offset(addr);
        self.pages.get(&idx).map(|p| p[offset]).unwrap_or(0)
    }

    /// Write a single byte.
    /// Allocates if this is the first time we write to this page.
    fn write_u8(&mut self, addr: u64, data: u8) {
        if addr > MAX_ADDR {
            panic!("write out of range: 0x{:x}", addr);
        }
        let idx = Self::page_idx(addr);
        let offset = Self::page_offset(addr);
        self.ensure_page(idx)[offset] = data;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read_u8() {
        let mut mem = Memory::default();

        // write
        mem.write_u8(0x1000, 0xAB);
        mem.write_u8(0x1001, 0xCD);
        assert_eq!(mem.pages.len(), 1);

        // read
        assert_eq!(mem.read_u8(0x1000), 0xAB);
        assert_eq!(mem.read_u8(0x1001), 0xCD);

        // read unmapped
        assert_eq!(mem.read_u8(0x7F3A_9C02_B47D_E610), 0);
    }

    #[test]
    #[should_panic]
    fn test_read_out_of_range() {
        let mem = Memory::default();
        mem.read_u8(u64::MAX);
    }

    #[test]
    #[should_panic]
    fn test_write_out_of_range() {
        let mut mem = Memory::default();
        mem.write_u8(u64::MAX, 0);
    }

    #[test]
    fn test_cross_page_write() {
        let mut mem = Memory::default();

        // force boundary cross
        // (PAGE_SIZE - 4)..(PAGE_SIZE + 4)
        let start = PAGE_SIZE as u64 - 4;
        for i in 0..8 {
            mem.write_u8(start + i, i as u8);
        }
        assert_eq!(mem.pages.len(), 2);

        // verify
        for i in 0..8 {
            assert_eq!(mem.read_u8(start + i), i as u8);
        }
    }
}
