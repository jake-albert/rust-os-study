#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use blog_os::{exit_qemu, serial_print, serial_println, QemuExitCode};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);

    loop {}
}

fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

// Override with `test_runner` to make sure panics...not using Testable trait
// Drawback...since we exit after one panic, currently usable for just one test
// but now we don't need anymore because we removed harness, so run from _start
// pub fn test_runner(tests: &[&dyn Fn()]) {
//     serial_print!("Running {} tests", tests.len());
//     for test in tests {
//         test();
//         serial_print!("[test did not panic");
//         exit_qemu(QemuExitCode::Failed);
//     }
//     exit_qemu(QemuExitCode::Success);
// }
