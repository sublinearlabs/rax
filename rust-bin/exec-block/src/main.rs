#![no_std]
#![no_main]

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::panic::PanicInfo;
use core::ptr;
use core::slice;
use core::sync::atomic::{AtomicUsize, Ordering};

mod exec;

extern crate alloc;

/// Heap size: 64 MB (adjust as needed for your workload)
const HEAP_SIZE: usize = 64 * 1024 * 1024;

/// Simple bump allocator for bare-metal environments
struct BumpAllocator {
    heap: UnsafeCell<[u8; HEAP_SIZE]>,
    next: AtomicUsize,
}

unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    const fn new() -> Self {
        Self {
            heap: UnsafeCell::new([0u8; HEAP_SIZE]),
            next: AtomicUsize::new(0),
        }
    }
}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();

        loop {
            let current = self.next.load(Ordering::Relaxed);
            let heap_start = self.heap.get() as usize;
            let alloc_start = (heap_start + current + align - 1) & !(align - 1);
            let alloc_end = alloc_start + size;

            if alloc_end - heap_start > HEAP_SIZE {
                return ptr::null_mut();
            }

            let new_next = alloc_end - heap_start;
            if self
                .next
                .compare_exchange(current, new_next, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                return alloc_start as *mut u8;
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't support deallocation
        // Memory is only freed when the program exits
    }
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator::new();

// Critical section implementation for bare-metal RISC-V
// These are required by the `critical-section` crate used by dependencies

mod critical_section_impl {
    use core::sync::atomic::{AtomicBool, Ordering};

    static LOCKED: AtomicBool = AtomicBool::new(false);

    #[no_mangle]
    unsafe extern "C" fn _critical_section_1_0_acquire() -> u8 {
        // For single-threaded bare-metal, we just need to track nesting
        // In a real system with interrupts, you'd disable interrupts here
        let was_locked = LOCKED.swap(true, Ordering::SeqCst);
        was_locked as u8
    }

    #[no_mangle]
    unsafe extern "C" fn _critical_section_1_0_release(token: u8) {
        // Restore previous state
        if token == 0 {
            LOCKED.store(false, Ordering::SeqCst);
        }
    }
}

/// Base address where input data is stored
/// The first 8 bytes contain the length (as u64), followed by the actual data
const INPUT_BASE_ADDR: usize = 0x80000000;

/// Entry point for the RISC-V program
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Read input length from memory (first 8 bytes at INPUT_BASE_ADDR)
    let input_len = unsafe { core::ptr::read_volatile(INPUT_BASE_ADDR as *const u64) as usize };

    // Read input data from memory (starts after the length field)
    let input_data_addr = INPUT_BASE_ADDR + 8;
    let input: &[u8] = unsafe { slice::from_raw_parts(input_data_addr as *const u8, input_len) };

    // Execute block by calling runner()
    let result: [u8; 32] = exec::runner(input);

    // Convert the 32-byte result into 4 u64 values for registers a0, a1, a2, a3
    // Each register holds 8 bytes (64 bits) on RV64
    let a0 = u64::from_le_bytes(result[0..8].try_into().unwrap());
    let a1 = u64::from_le_bytes(result[8..16].try_into().unwrap());
    let a2 = u64::from_le_bytes(result[16..24].try_into().unwrap());
    let a3 = u64::from_le_bytes(result[24..32].try_into().unwrap());

    // Store result in a0, a1, a2, a3 registers
    unsafe {
        core::arch::asm!(
            "mv a0, {0}",
            "mv a1, {1}",
            "mv a2, {2}",
            "mv a3, {3}",
            in(reg) a0,
            in(reg) a1,
            in(reg) a2,
            in(reg) a3,
        );
    }

    // Exit with ecall (for emulators that support it)
    exit(0);
}

/// Exit the program using RISC-V ecall
fn exit(code: i32) -> ! {
    unsafe {
        core::arch::asm!(
            "li a7, 93",    // syscall number for exit
            "mv a0, {0}",   // exit code
            "ecall",
            in(reg) code,
            options(noreturn)
        );
    }
}

/// Panic handler required for no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
