// kernel/src/hal/ioapic.rs

use super::{redir_flags, reg, IoApicInfo, IO_APICS, IO_APIC_COUNT, MAX_IOAPICS};
use crate::{log_debug, log_error, log_info, log_trace, log_warn};
use x86_64::VirtAddr;

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
