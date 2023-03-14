#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

use blog_os::println;
use blog_os::task::{executor::Executor, keyboard, Task};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

// the boot info contains info on our memory mapping that only the bootloader knows.
// the kernel can't get on its own so must be passed in by the bootloader!
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::{
        allocator,
        memory::{self, BootInfoFrameAllocator},
    };
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    blog_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // implicitly using the global allocator we set up...
    let heap_value = Box::new(41);
    println!(
        "a number value on the heap: {}, stored at {:p}",
        *heap_value, heap_value
    );

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    core::mem::drop(reference_counted);
    println!(
        "reference count is {} now",
        Rc::strong_count(&cloned_reference)
    );

    // manually traverse paging table
    // let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

    // for (i, entry) in l4_table.iter().enumerate() {
    //     // We could also print empty entries, but wouldn't all fit in the screen.
    //     if !entry.is_unused() {
    //         println!("L4 Entry {}: {:?}", i, entry);

    //         // get physical address form entry and convert it...to get to L3
    //         let phys = entry.frame().unwrap().start_address();
    //         let virt = phys.as_u64() + boot_info.physical_memory_offset;
    //         let ptr = VirtAddr::new(virt).as_mut_ptr();
    //         let l3_table: &PageTable = unsafe { &*ptr };

    //         // print non-empty entries of the level 3 table
    //         for (i, entry) in l3_table.iter().enumerate() {
    //             if !entry.is_unused() {
    //                 println!("  L3 Entry {}: {:?}", i, entry);
    //             }
    //         }
    //     }
    // }

    // let addresses = [
    //     // the identity-mapped vga buffer page ... this maps to same physical address
    //     // because VGA buffer is identity mapped
    //     0xb8000,
    //     // some code page...last 12 bits are same since just offset is added
    //     0x201008,
    //     // some stack page...last 12 bits are same since just offset is added
    //     0x0100_0020_1a10,
    //     // virtual address mapped to physical address 0 ... using OffsetPageTable, huge page works
    //     boot_info.physical_memory_offset,
    // ];
    // for &address in &addresses {
    //     let virt = VirtAddr::new(address);
    //     // let phys = unsafe { translate_addr(virt, phys_mem_offset) };
    //     let phys = mapper.translate_addr(virt);
    //     println!("{:?} -> {:?}", virt, phys);
    // }

    // map an unused page to the VGA buffer...the level 1 table for the page at address 0 already
    // exists so there is no issue with us not creating any new frames
    // let page = Page::containing_address(VirtAddr::new(0));
    // on the other hand, this page would require creating new frames since not on same tables
    // let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);
    // write the string `New!` to the screen through the new mapping
    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // write to offset 400, not start, because top line of VGA buffer is shifted off by the below
    // println. the value itself is just the correct bytes all compact
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    // trigger a page fault
    // unsafe {
    //     *(0xdeadbeef as *mut u64) = 42;
    // };

    // trigger a stack overflow
    // fn stack_overflow() {
    //     stack_overflow();
    // }
    // stack_overflow();

    // attempt to trigger deadlock of WRITER
    // loop {
    //     use blog_os::print;
    //     for _ in 0..10000 {}
    //     print!("-");
    // }

    // attempt to trigger page fault by writing to memory outside kernel
    // let ptr = 0xdeadbeaf as *mut u32;
    // unsafe {
    //     *ptr = 42;
    // }

    // attempt to read to current construction pointer works, but not write
    // let ptr = 0x204137 as *mut u32;
    // // read from a code page
    // unsafe {
    //     let x = *ptr;
    // }
    // println!("read worked");
    // // write to a code page
    // unsafe {
    //     *ptr = 42;
    // }
    // println!("write worked");

    // Determine that physical address of level 4 page table starts at 0x1000
    // use x86_64::registers::control::Cr3;
    // let (level_4_page_table, _) = Cr3::read();
    // println!(
    //     "Level 4 page table at: {:?}",
    //     level_4_page_table.start_address()
    // );

    let mut executor = Executor::new();
    // Wrapping the future in a `Task` type moves it to the heap and pins it!
    executor.spawn(Task::new(example_task()));
    // Never ends, since poll_next method never returns None for scancodes
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    #[cfg(test)]
    test_main();

    // executor.run() diverges...so now no need to have a loop like this
    // println!("It did not crash!");
    // compared to `loop {}` we have much better CPU usage now...put CPU in idle
    // state in between external interrupts like timer
    // blog_os::hlt_loop();
}

// START: Async/await
// first example is one where the task will return Poll::Ready on first poll call...
async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}
// END: Async/await

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
