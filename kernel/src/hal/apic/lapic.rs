// kernel/src/hal/lapic.rs

use super::{flags, ApicInfo, ApicRegister, APIC_INFO, LOCAL_APIC_BASE};
use crate::hal::io;
use crate::mm::vma;
use crate::{log_debug, log_error, log_info, log_trace, log_warn};
use x86_64::VirtAddr;

/// Read APIC registers
///
/// # Safety
/// The caller must ensure the APIC is initialized.
#[inline]
pub unsafe fn read_apic_reg(reg: ApicRegister) -> Option<u32> {
    LOCAL_APIC_BASE.map(|base| {
        let addr = base.as_u64() + reg as u64;
        core::ptr::read_volatile(addr as *const u32)
    })
}

/// Write to APIC registers
///
/// # Safety
/// The caller must ensure the APIC is initialized.
#[inline]
pub unsafe fn write_apic_reg(reg: ApicRegister, value: u32) -> bool {
    if let Some(base) = LOCAL_APIC_BASE {
        let addr = base.as_u64() + reg as u64;
        core::ptr::write_volatile(addr as *mut u32, value);
        true
    } else {
        false
    }
}

/// Directly read the APIC register using an offset (for apic_timer)
///
/// # Safety
/// The caller must ensure the APIC is initialized and the offset is valid.
#[inline]
pub unsafe fn read_apic_reg_raw(offset: u32) -> Option<u32> {
    LOCAL_APIC_BASE.map(|base| {
        let addr = base.as_u64() + offset as u64;
        core::ptr::read_volatile(addr as *const u32)
    })
}

/// Directly read the APIC register using an offset (for apic_timer)
///
/// # Safety
/// The caller must ensure the APIC is initialized and the offset is valid.
#[inline]
pub unsafe fn write_apic_reg_raw(offset: u32, value: u32) -> bool {
    if let Some(base) = LOCAL_APIC_BASE {
        let addr = base.as_u64() + offset as u64;
        core::ptr::write_volatile(addr as *mut u32, value);
        true
    } else {
        false
    }
}

/// Initialize the Local APIC (using mapped virtual addresses)
///
/// # Safety
/// Must be called after paging is initialized and APIC memory is mapped.
pub unsafe fn init_local_apic_with_vaddr(base_vaddr: VirtAddr) {
    log_debug!("Initializing Local APIC at {:#x}", base_vaddr.as_u64());

    // Storage base address
    LOCAL_APIC_BASE = Some(base_vaddr);

    // Read APIC information
    let id_reg = read_apic_reg(ApicRegister::Id).unwrap();
    let version_reg = read_apic_reg(ApicRegister::Version).unwrap();

    APIC_INFO = ApicInfo {
        id: id_reg >> 24,
        version: version_reg & 0xFF,
        max_lvt: (version_reg >> 16) & 0xFF,
    };

    // Enable APIC (via Spurious Interrupt Vector Register)
    let spurious = flags::APIC_SW_ENABLE | 0xFF;
    write_apic_reg(ApicRegister::SpuriousInterruptVector, spurious);

    // Set task priority to 0 (accept all interrupts)
    write_apic_reg(ApicRegister::TaskPriority, 0);

    // Configure LVT entries - default to full mask
    write_apic_reg(ApicRegister::LvtTimer, flags::LVT_MASKED);
    write_apic_reg(ApicRegister::LvtLint0, flags::LVT_MASKED);
    write_apic_reg(ApicRegister::LvtLint1, flags::LVT_MASKED);
    write_apic_reg(ApicRegister::LvtError, flags::LVT_MASKED);
    write_apic_reg(ApicRegister::LvtPerformanceCounter, flags::LVT_MASKED);
    write_apic_reg(ApicRegister::LvtThermalSensor, flags::LVT_MASKED);

    log_info!("Local APIC initialized");
    print_info();
}

/// Initialize Local APIC (from physical address, needs to be mapped first)
///
/// # Safety
/// The physical address must be a valid APIC base address.
pub unsafe fn init_local_apic(base_paddr: x86_64::PhysAddr) {
    let base_vaddr = vma::phys_to_virt(base_paddr.as_u64());
    init_local_apic_with_vaddr(base_vaddr);
}

/// Send EOI (End of Interrupt)
///
/// This is the most frequently called function and must be extremely fast.
#[inline]
pub fn send_eoi() {
    unsafe {
        if let Some(base) = LOCAL_APIC_BASE {
            let addr = base.as_u64() + ApicRegister::Eoi as u64;
            core::ptr::write_volatile(addr as *mut u32, 0);
        }
    }
}

/// Get the Local APIC ID
#[inline]
pub fn get_apic_id() -> Option<u32> {
    unsafe { Some(APIC_INFO.id) }
}

/// Get the Local APIC base virtual address
#[inline]
pub fn get_base_vaddr() -> Option<VirtAddr> {
    unsafe { LOCAL_APIC_BASE }
}

/// Get APIC version
#[inline]
pub fn get_version() -> Option<u32> {
    unsafe {
        if LOCAL_APIC_BASE.is_some() {
            Some(APIC_INFO.version)
        } else {
            None
        }
    }
}

/// Get the maximum number of LVT entries
#[inline]
pub fn get_max_lvt() -> Option<u32> {
    unsafe {
        if LOCAL_APIC_BASE.is_some() {
            Some(APIC_INFO.max_lvt)
        } else {
            None
        }
    }
}

/// Disable legacy 8259 PIC
///
/// Must be called before using the APIC to avoid conflicts
pub fn disable_legacy_pic() {
    unsafe {
        // Master PIC
        io::io_port_wb(0x20, 0x11); // ICW1: initialization
        io::io_port_wb(0x21, 0x20); // ICW2: Interrupt vector offset (32-39)
        io::io_port_wb(0x21, 0x04); // ICW3: Tell the Master PIC Slave to be on IRQ2
        io::io_port_wb(0x21, 0x01); // ICW4: 8086 mode

        // Slave PIC
        io::io_port_wb(0xA0, 0x11); // ICW1: initialization
        io::io_port_wb(0xA1, 0x28); // ICW2: Interrupt vector offset (40-47)
        io::io_port_wb(0xA1, 0x02); // ICW3: Tell the Slave PIC to connect to Master IRQ2
        io::io_port_wb(0xA1, 0x01); // ICW4: 8086 mode

        // Mask all IRQs (disable PIC)
        io::io_port_wb(0x21, 0xFF);
        io::io_port_wb(0xA1, 0xFF);
    }

    log_info!("Legacy 8259 PIC disabled");
}

/// Print APIC information
pub fn print_info() {
    unsafe {
        if LOCAL_APIC_BASE.is_some() {
            log_info!("Local APIC ID: {}", APIC_INFO.id);
            log_info!("Local APIC Version: {:#x}", APIC_INFO.version);
            log_info!("Max LVT Entry: {}", APIC_INFO.max_lvt);
        } else {
            log_warn!("Local APIC not initialized");
        }
    }
}
