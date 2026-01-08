use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

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