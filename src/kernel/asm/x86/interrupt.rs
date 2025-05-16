use core::mem;
use core::ptr;
use x86::segmentation::{SegmentSelector, Descriptor};
use x86::Ring;
use x86::irq::{self, PageFaultError, EXCEPTIONS};
use crate::{print, println};
use crate::hal::cpu;

#[repr(C, packed)]
pub struct IsrParam {
    vector: u32,
    err_code: u32,
    eip: u32,
    cs: u32,
    eflags: u32,
    esp: u32,
    ss: u32,
}

impl IsrParam {
    pub fn vector(&self) -> u32 {
        return self.vector;
    }

    pub fn err_code(&self) -> u32 {
        // unsafe { 
        //     let ptr = ptr::addr_of!(self.err_code);
        //     ptr::read_unaligned(ptr)
        // }
        return self.err_code;
    }
    
    pub fn eip(&self) -> u32 {
        return self.eip;
    }
    
    pub fn cs(&self) -> u32 {
        return self.cs;
    }
    
    pub fn eflags(&self) -> u32 {
        return self.eflags;
    }
}

#[no_mangle]
pub extern "C" fn interrupt_handler(param: *const IsrParam) {
    // param.as_ref().expect("中斷參數為 null")
    let param_ref = unsafe { param.as_ref().unwrap() };
    
    match param_ref.vector {
        0 => divide_error_handler(param_ref),
        1 => debug_error_handler(param_ref),
        2 => nonmaskable_interrupt_handler(param_ref),
        3 => breakpoint_handler(param_ref),
        4 => overflow_handler(param_ref),

        5 => bound_range_exceeded_handler(param_ref),
        6 => invalid_opcode_handler(param_ref),
        7 => device_not_available_handler(param_ref),
        8 => double_fault_handler(param_ref),
        9 => coprocessor_segment_overrun_handler(param_ref),
    
        10 => invalid_tss_handler(param_ref),
        11 => segment_not_present_handler(param_ref),
        12 => stack_segment_fault_handler(param_ref),
        13 => general_protection_handler(param_ref),
        14 => page_fault_handler(param_ref),
        // 15 => reserved_handler(param_ref),
        16 => x87_fpu_floating_point_handler(param_ref),
        17 => alignment_check_handler(param_ref),
        18 => machine_check_handler(param_ref),
        19 => simd_floating_point_handler(param_ref),
        20 => virtualization_handler(param_ref),
        // 21 => reserved_handler(param_ref),
        // 22 => reserved_handler(param_ref),
        // 23 => reserved_handler(param_ref),
        // 24 => reserved_handler(param_ref),
        // 25 => reserved_handler(param_ref),
        // 26 => reserved_handler(param_ref),
        // 27 => reserved_handler(param_ref),
        // 28 => reserved_handler(param_ref),
        // 29 => reserved_handler(param_ref),
        // 30 => reserved_handler(param_ref),
        // 31 => reserved_handler(param_ref),

        _ => reserved_handler(param_ref),
    }
}

fn print_exception(param: &IsrParam) {
    let vector = param.vector();

    if vector < 32 {
        let ex = &EXCEPTIONS[vector as usize];
        println!("CPU Exception: {}", ex);
        println!("EIP: 0x{:x}, CS: 0x{:x}, EFLAGS: 0x{:x}", param.eip(), param.cs(), param.eflags());

        let error_code = param.err_code();
        
        if error_code != 0 {
            println!("Error code: 0x{:x}", error_code);
            
            if vector == irq::PAGE_FAULT_VECTOR.into() {
                let cr2 = cpu::cpu_r_cr2() ;
                let pf_error = PageFaultError::from_bits_truncate(error_code);

                println!("Fault address: 0x{:x}", cr2);
                println!("Fault details:\n{}", pf_error);
            }
        }
        
        // if let Some(addr) = fault_addr {
        //     println!("Fault address: 0x{:x}", addr);
        // }
    } else {
        println!("Unhandled interrupt: Vector {}", vector);
    }
    
    loop {}
}

/// isr0
#[no_mangle]
pub fn divide_error_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr1
#[no_mangle]
pub fn debug_error_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr2
#[no_mangle]
pub fn nonmaskable_interrupt_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr3
#[no_mangle]
pub fn breakpoint_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr4
#[no_mangle]
pub fn overflow_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr5
#[no_mangle]
pub fn bound_range_exceeded_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr6
#[no_mangle]
pub fn invalid_opcode_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr7
#[no_mangle]
pub fn device_not_available_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr8
#[no_mangle]
pub fn double_fault_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr9
#[no_mangle]
pub fn coprocessor_segment_overrun_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr10
#[no_mangle]
pub fn invalid_tss_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr11
#[no_mangle]
pub fn segment_not_present_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr12
#[no_mangle]
pub fn stack_segment_fault_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr13
#[no_mangle]
pub fn general_protection_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr14
#[no_mangle]
pub fn page_fault_handler(param: &IsrParam) {    
    print_exception(param)
}

/// isr16
#[no_mangle]
pub fn x87_fpu_floating_point_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr17
#[no_mangle]
pub fn alignment_check_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr18
#[no_mangle]
pub fn machine_check_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr19
#[no_mangle]
pub fn simd_floating_point_handler(param: &IsrParam) {
    print_exception(param)
}

/// isr20
#[no_mangle]
pub fn virtualization_handler(param: &IsrParam) {
    print_exception(param)
}


/// isr15, 21-31
#[no_mangle]
pub fn reserved_handler(param: &IsrParam) {
    print_exception(param)
}