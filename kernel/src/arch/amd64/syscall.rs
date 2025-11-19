// kernel/src/arch/amd64/syscall.rs

use crate::{kprint, log_debug, log_info};
use core::arch::global_asm;
use x86_64::registers::model_specific::*;
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

global_asm!(
    ".global syscall_entry",
    ".type syscall_entry, @function",
    "syscall_entry:",
    "   swapgs",
    "   movq %rsp, %gs:0", // Storage User Stack
    "   movq %gs:8, %rsp", // Switching core stacks
    "   pushq %rcx",       // Store back address
    "   pushq %r11",       // Save RFLAGS
    "   pushq %rdi",
    "   pushq %rsi",
    "   pushq %rdx",
    "   pushq %r10",
    "   pushq %r8",
    "   pushq %r9",
    "   movq %r10, %rcx", // The fourth parameter was moved from r10 to rcx.
    "   call syscall_dispatch",
    "   popq %r9", // Restore register
    "   popq %r8",
    "   popq %r10",
    "   popq %rdx",
    "   popq %rsi",
    "   popq %rdi",
    "   popq %r11",
    "   popq %rcx",
    "   movq %gs:0, %rsp", // Restore user stack
    "   swapgs",
    "   sysretq",
    options(att_syntax)
);

extern "C" {
    fn syscall_entry();

}

/// Initialize system call mechanism
pub fn init() {
    unsafe {
        // Configure segment selector
        Star::write(
            crate::arch::amd64::gdt::user_code_selector(),
            crate::arch::amd64::gdt::user_data_selector(),
            crate::arch::amd64::gdt::kernel_code_selector(),
            crate::arch::amd64::gdt::kernel_data_selector(),
        )
        .unwrap();

        // Set the SYSCALL entry address
        LStar::write(VirtAddr::new(syscall_entry as u64));

        // Configure the RFLAGS mask (clear the interrupt flag during SYSCALL).
        SFMask::write(RFlags::INTERRUPT_FLAG);

        // Enable SYSCALL/SYSRET
        Efer::update(|flags| {
            *flags |= EferFlags::SYSTEM_CALL_EXTENSIONS;
        });
    }

    log_info!("SYSCALL/SYSRET enabled");
}

/// System call dispatcher
#[no_mangle]
extern "C" fn syscall_dispatch(number: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    log_debug!("syscall({}, {:#x}, {:#x}, {:#x})", number, arg1, arg2, arg3);

    match number {
        0 => sys_read(arg1, arg2, arg3),

        1 => sys_write(arg1, arg2, arg3),

        60 => sys_exit(arg1),

        _ => {
            log_debug!("Unknown syscall: {}", number);
            -1 // Error
        }
    }
}

/// sys_write
/// fd=1 (stdout), fd=2 (stderr)
fn sys_write(fd: u64, buf: u64, count: u64) -> i64 {
    if fd != 1 && fd != 2 {
        return -1;
    }

    if buf == 0 || count == 0 || count > 4096 {
        return -1;
    }

    // Read data from user space
    let slice = unsafe { core::slice::from_raw_parts(buf as *const u8, count as usize) };

    if let Ok(s) = core::str::from_utf8(slice) {
        kprint!("{}", s);
    } else {
        for &byte in slice {
            kprint!("{}", byte as char);
        }
    }

    count as i64
}

/// sys_read
/// fd=0 (stdin)
fn sys_read(fd: u64, buf: u64, count: u64) -> i64 {
    // 只支持 stdin

    if fd != 0 {
        return -1;
    }

    if buf == 0 || count == 0 || count > 4096 {
        return -1;
    }
    // TODO: Implement the actual read

    // Currently, returning 0 indicates EOF
    0
}

/// sys_exit
fn sys_exit(status: u64) -> ! {
    log_info!("Process exiting with status: {}", status);

    // TODO: Clean up process resources

    loop {
        crate::hal::cpu::cpu_halt();
    }
}
