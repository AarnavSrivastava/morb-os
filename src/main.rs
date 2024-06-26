#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(morb_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use morb_os::allocator::HEAP_SIZE;
use morb_os::println;
use bootloader::{BootInfo, entry_point};
use morb_os::task::{Task, simple_executor::SimpleExecutor};
use morb_os::task::keyboard;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use morb_os::{memory::{self, BootInfoFrameAllocator}, allocator};
    use x86_64::VirtAddr;

    println!("Booting system up...");
    morb_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    println!("Memory Available: {:?} KBs", HEAP_SIZE / 1024);

    #[cfg(test)]
    test_main();

    println!("MorbOS is live!");

    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    morb_os::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
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