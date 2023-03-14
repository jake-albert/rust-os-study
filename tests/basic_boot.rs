#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::println;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

// don't need this anymore because we use test_runner from blog_os...lib.rs
// fn test_runner(tests: &[&dyn Fn()]) {
//     unimplemented!();
// }

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

// Ensures that `println` works right after booting...without any initialization
// from the `_start` function in `main.rs`, since the `_start` that IS called
// in this file for this environment does no initialization and goes straight to
// running tests with `test_main`
#[test_case]
fn test_println() {
    println!("test_println output");
}
