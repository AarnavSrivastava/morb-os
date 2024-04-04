#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(morb_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use morb_os::{println, vga_buffer::OS_LIVE};
use bootloader::{BootInfo, entry_point};
// use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    // use morb_os::{memory::{self, BootInfoFrameAllocator}, allocator};
    use x86_64::VirtAddr;

    println!("Booting system up...");
    morb_os::init();

    // let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // let mut frame_allocator = unsafe {
    //     BootInfoFrameAllocator::init(&boot_info.memory_map)
    // };

    // allocator::init_heap(&mut mapper, &mut frame_allocator)
    //     .expect("heap initialization failed");

    // // allocate a number on the heap
    // let heap_value = Box::new(41);
    // println!("heap_value at {:p}", heap_value);

    // // create a dynamically sized vector
    // let mut vec = Vec::new();
    // for i in 0..500 {
    //     vec.push(i);
    // }
    // println!("vec at {:p}", vec.as_slice());

    // // create a reference counted vector -> will be freed when count reaches 0
    // let reference_counted = Rc::new(vec![1, 2, 3]);
    // let cloned_reference = reference_counted.clone();
    // println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    // core::mem::drop(reference_counted);
    // println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    println!("MorbOS is live!");

    *OS_LIVE.lock() = true;
    morb_os::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    morb_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    morb_os::test_panic_handler(info)
}