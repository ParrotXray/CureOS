// kernel/src/hal/ioapic.rs - 無鎖設計

use x86_64::VirtAddr;
use crate::{log_trace, log_debug, log_info, log_warn, log_error};

/// IO APIC register selector
const IOREGSEL: u32 = 0x00;
const IOWIN: u32 = 0x10;

/// IO APIC register index
#[allow(dead_code)]
mod reg {
    pub const ID: u8 = 0x00;
    pub const VER: u8 = 0x01;
    pub const ARB: u8 = 0x02;
    pub const REDTBL_BASE: u8 = 0x10;
}

/// Redirection Entry flag
#[allow(dead_code)]
mod redir_flags {
    pub const MASKED: u64 = 1 << 16;
    pub const TRIGGER_LEVEL: u64 = 1 << 15;
    pub const TRIGGER_EDGE: u64 = 0;
    pub const POLARITY_LOW: u64 = 1 << 13;
    pub const POLARITY_HIGH: u64 = 0;
    pub const DEST_LOGICAL: u64 = 1 << 11;
    pub const DEST_PHYSICAL: u64 = 0;
    pub const DELIVERY_FIXED: u64 = 0 << 8;
    pub const DELIVERY_LOWEST: u64 = 1 << 8;
}

// Supports up to 8 IO APICs
const MAX_IOAPICS: usize = 8;

//IO APIC information array (read-only after initialization)
static mut IO_APICS: [Option<IoApicInfo>; MAX_IOAPICS] = [None; MAX_IOAPICS];
static mut IO_APIC_COUNT: usize = 0;

#[derive(Debug, Clone, Copy)]
struct IoApicInfo {
    base_vaddr: VirtAddr,
    id: u8,
    gsi_base: u32,
    max_redirection_entries: u8,
}

impl IoApicInfo {
    /// Read IO APIC register
    #[inline]
    unsafe fn read(&self, reg: u8) -> u32 {
        let regsel_addr = self.base_vaddr.as_u64() + IOREGSEL as u64;
        let win_addr = self.base_vaddr.as_u64() + IOWIN as u64;

        core::ptr::write_volatile(regsel_addr as *mut u32, reg as u32);
        core::ptr::read_volatile(win_addr as *const u32)
    }

    /// Write to IO APIC register
    #[inline]
    unsafe fn write(&self, reg: u8, value: u32) {
        let regsel_addr = self.base_vaddr.as_u64() + IOREGSEL as u64;
        let win_addr = self.base_vaddr.as_u64() + IOWIN as u64;

        core::ptr::write_volatile(regsel_addr as *mut u32, reg as u32);
        core::ptr::write_volatile(win_addr as *mut u32, value);
    }

    /// Read the redirection table entry
    #[inline]
    unsafe fn read_redirection_entry(&self, irq: u8) -> u64 {
        if irq >= self.max_redirection_entries {
            log_warn!("IRQ {} out of range for IO APIC {}", irq, self.id);
            return 0;
        }

        let low_reg = reg::REDTBL_BASE + (irq * 2);
        let high_reg = low_reg + 1;

        let low = self.read(low_reg) as u64;
        let high = self.read(high_reg) as u64;

        (high << 32) | low
    }

    /// Write redirection table entry
    #[inline]
    unsafe fn write_redirection_entry(&self, irq: u8, entry: u64) {
        if irq >= self.max_redirection_entries {
            log_warn!("IRQ {} out of range for IO APIC {}", irq, self.id);
            return;
        }

        let low_reg = reg::REDTBL_BASE + (irq * 2);
        let high_reg = low_reg + 1;

        let low = entry as u32;
        let high = (entry >> 32) as u32;

        self.write(high_reg, high);
        self.write(low_reg, low);
    }
}

/// Initialize a single IO APIC (using mapped virtual addresses)
///
/// # Safety
/// Must be called after mapping the IO APIC memory.
pub unsafe fn init_single_ioapic(base_vaddr: VirtAddr, id: u8, gsi_base: u32) {
    if IO_APIC_COUNT >= MAX_IOAPICS {
        log_error!("Too many IO APICs! Maximum {} supported", MAX_IOAPICS);
        return;
    }

    log_debug!("Initializing IO APIC {} at {:#x}", id, base_vaddr.as_u64());

    // Create a temporary structure to read the version information
    let temp_info = IoApicInfo {
        base_vaddr,
        id,
        gsi_base,
        max_redirection_entries: 0,
    };

    // Read version information to get the maximum number of entries
    let version = temp_info.read(reg::VER);
    let max_entries = ((version >> 16) & 0xFF) as u8 + 1;

    // 創建完整的信息結構
    let info = IoApicInfo {
        base_vaddr,
        id,
        gsi_base,
        max_redirection_entries: max_entries,
    };

    // Mask all interrupts
    for irq in 0..max_entries {
        info.write_redirection_entry(irq, redir_flags::MASKED);
    }

    // Store to global array
    IO_APICS[IO_APIC_COUNT] = Some(info);
    IO_APIC_COUNT += 1;

    log_info!("IO APIC {} initialized, {} entries", id, max_entries);
    print_single_ioapic_info(&info);
}

/// Initialize all IO APICs (from physical addresses)
///
/// # Safety
/// The physical addresses must be correctly mapped.
pub unsafe fn init_io_apics(io_apics: &[(x86_64::PhysAddr, u8, u32)]) {
    log_info!("Initializing {} IO APIC(s)...", io_apics.len());

    for (paddr, id, gsi_base) in io_apics {
        let vaddr = crate::mm::vma::phys_to_virt(paddr.as_u64());
        init_single_ioapic(vaddr, *id, *gsi_base);
    }

    log_info!("All IO APICs initialized");
}

/// Configure IRQ redirection
///
/// # Parameters
/// - `irq`: IRQ number (0-23)
/// - `vector`: Interrupt vector number (32-255)
/// - `dest_apic_id`: Destination Local APIC ID
/// - `level_triggered`: true = level triggered, false = edge triggered
/// - `active_low`: true = active low, false = active high
pub fn set_irq_redirect(
    irq: u8,
    vector: u8,
    dest_apic_id: u8,
    level_triggered: bool,
    active_low: bool,
) {
    unsafe {
        // Find the IO APIC responsible for this IRQ
        for i in 0..IO_APIC_COUNT {
            if let Some(ioapic) = &IO_APICS[i] {
                let gsi_start = ioapic.gsi_base as u8;
                let gsi_end = gsi_start + ioapic.max_redirection_entries;

                if irq >= gsi_start && irq < gsi_end {
                    let local_irq = irq - gsi_start;

                    // Build redirection entry
                    let mut entry: u64 = 0;

                    // Set target APIC ID (bits 56-63)
                    entry |= (dest_apic_id as u64) << 56;

                    // Set the trigger mode
                    if level_triggered {
                        entry |= redir_flags::TRIGGER_LEVEL;
                    }

                    // Set polarity
                    if active_low {
                        entry |= redir_flags::POLARITY_LOW;
                    }

                    // Set the transfer mode to Fixed
                    entry |= redir_flags::DELIVERY_FIXED;

                    // Set the target mode to Physical
                    entry |= redir_flags::DEST_PHYSICAL;

                    // Set the vector number
                    entry |= vector as u64;

                    // Write to the redirection table (unmask)
                    ioapic.write_redirection_entry(local_irq, entry);

                    log_info!(
                        "IO APIC {} IRQ {} -> Vector {} (APIC {}, {}, {})",
                        ioapic.id,
                        irq,
                        vector,
                        dest_apic_id,
                        if level_triggered { "level" } else { "edge" },
                        if active_low { "low" } else { "high" }
                    );

                    return;
                }
            }
        }

        log_warn!("No IO APIC found for IRQ {}", irq);
    }
}

/// Mask IRQ
pub fn mask_irq(irq: u8) {
    unsafe {
        for i in 0..IO_APIC_COUNT {
            if let Some(ioapic) = &IO_APICS[i] {
                let gsi_start = ioapic.gsi_base as u8;
                let gsi_end = gsi_start + ioapic.max_redirection_entries;

                if irq >= gsi_start && irq < gsi_end {
                    let local_irq = irq - gsi_start;
                    let mut entry = ioapic.read_redirection_entry(local_irq);
                    entry |= redir_flags::MASKED;
                    ioapic.write_redirection_entry(local_irq, entry);

                    log_debug!("IO APIC {} IRQ {} masked", ioapic.id, irq);
                    return;
                }
            }
        }
    }
}

/// Unmask IRQ
pub fn unmask_irq(irq: u8) {
    unsafe {
        for i in 0..IO_APIC_COUNT {
            if let Some(ioapic) = &IO_APICS[i] {
                let gsi_start = ioapic.gsi_base as u8;
                let gsi_end = gsi_start + ioapic.max_redirection_entries;

                if irq >= gsi_start && irq < gsi_end {
                    let local_irq = irq - gsi_start;
                    let mut entry = ioapic.read_redirection_entry(local_irq);
                    entry &= !redir_flags::MASKED;
                    ioapic.write_redirection_entry(local_irq, entry);

                    log_debug!("IO APIC {} IRQ {} unmasked", ioapic.id, irq);
                    return;
                }
            }
        }
    }
}

/// Print single IO APIC information
fn print_single_ioapic_info(info: &IoApicInfo) {
    unsafe {
        let id = info.read(reg::ID) >> 24;
        let version = info.read(reg::VER);
        let apic_ver = version & 0xFF;

        log_info!("ID: {}", id);
        log_info!("Version: {:#x}", apic_ver);
        log_info!("Max Entries: {}", info.max_redirection_entries);
        log_info!("GSI Base: {}", info.gsi_base);
    }
}

/// Print all IO APIC information
pub fn print_info() {
    unsafe {
        log_info!("=== IO APIC Information ===");
        log_info!("Total IO APICs: {}", IO_APIC_COUNT);

        for i in 0..IO_APIC_COUNT {
            if let Some(info) = &IO_APICS[i] {
                log_info!("IO APIC {}:", i);
                print_single_ioapic_info(info);
            }
        }
    }
}