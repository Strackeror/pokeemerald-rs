## Panic handler and allocator
### panic.rs
```rs
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        mgba_print(1, c"PANIC".to_bytes());
        let mut text: ArrayVec<u8, 256> = Default::default();
        _ = write!(text, "{info:?}\0");
        mgba_print(0, &text);
    }
    loop {}
}
```

### alloc.rs
```rs
struct PokeAllocator;
unsafe impl GlobalAlloc for PokeAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let ptr = unsafe { Alloc_(layout.size() as u32, c"RUST".as_ptr()) } as *mut u8;
        if ptr.is_null() {
            panic!("heap overflow")
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        unsafe { Free(ptr as *mut _) }
    }
}

#[global_allocator]
static GLOBAL: PokeAllocator = PokeAllocator;
