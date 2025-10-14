// kernel/src/hal/ioapic.rs
use x86_64::{PhysAddr, VirtAddr};
use spin::Mutex;
use alloc::vec::Vec;
use crate::{log_trace, log_debug, log_info, log_warn, log_error};
use crate::mm::vma;

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

pub struct IoApic {
    base_vaddr: VirtAddr,
    id: u8,
    gsi_base: u32,
    max_redirection_entries: u8,
}

static IO_APICS: Mutex<Vec<IoApic>> = Mutex::new(Vec::new());

impl IoApic {
    /// Create IO APIC from physical address
    pub unsafe fn new(base_paddr: PhysAddr, id: u8, gsi_base: u32) -> Self {
        let base_vaddr = vma::phys_to_virt(base_paddr.as_u64());

        log_debug!("IO APIC {} physical base: {:#x}", id, base_paddr.as_u64());
        log_debug!("IO APIC {} virtual base: {:#x}", id, base_vaddr.as_u64());
        log_debug!("IO APIC {} GSI base: {}", id, gsi_base);

        let mut ioapic = Self {
            base_vaddr,
            id,
            gsi_base,
            max_redirection_entries: 0,
        };

        // Read version information to get the maximum number of redirect entries
        let version = ioapic.read(reg::VER);
        ioapic.max_redirection_entries = ((version >> 16) & 0xFF) as u8 + 1;

        ioapic
    }

    /// Read IO APIC registers
    unsafe fn read(&mut self, reg: u8) -> u32 {
        let regsel_addr = self.base_vaddr.as_u64() + IOREGSEL as u64;
        let win_addr = self.base_vaddr.as_u64() + IOWIN as u64;

        core::ptr::write_volatile(regsel_addr as *mut u32, reg as u32);
        core::ptr::read_volatile(win_addr as *const u32)
    }

    /// Writing to IO APIC registers
    unsafe fn write(&mut self, reg: u8, value: u32) {
        let regsel_addr = self.base_vaddr.as_u64() + IOREGSEL as u64;
        let win_addr = self.base_vaddr.as_u64() + IOWIN as u64;

        core::ptr::write_volatile(regsel_addr as *mut u32, reg as u32);
        core::ptr::write_volatile(win_addr as *mut u32, value);
    }

    /// Read redirection table entries
    unsafe fn read_redirection_entry(&mut self, irq: u8) -> u64 {
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
    unsafe fn write_redirection_entry(&mut self, irq: u8, entry: u64) {
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

    /// Initialize the IO APIC
    pub unsafe fn init(&mut self) {
        // Mask all interrupts
        for irq in 0..self.max_redirection_entries {
            let entry = redir_flags::MASKED;
            self.write_redirection_entry(irq, entry);
        }

        log_info!("IO APIC {} initialized, {} entries", self.id, self.max_redirection_entries);
    }

    /// Configure IRQ redirection
    ///
    /// # Parameters
    /// - `irq`: IRQ number (0-23)
    /// - `vector`: Interrupt vector number (32-255)
    /// - `dest_apic_id`: Destination Local APIC ID
    /// - `level_triggered`: true = level triggered, false = edge triggered
    /// - `active_low`: true = active low, false = active high
    pub unsafe fn set_irq_redirect(
        &mut self,
        irq: u8,
        vector: u8,
        dest_apic_id: u8,
        level_triggered: bool,
        active_low: bool,
    ) {
        let mut entry: u64 = 0;

        // Set target APIC ID (bits 56-63)
        entry |= (dest_apic_id as u64) << 56;

        // Set the trigger mode
        if level_triggered {
            entry |= redir_flags::TRIGGER_LEVEL;
        }

        // Setting Polarity
        if active_low {
            entry |= redir_flags::POLARITY_LOW;
        }

        // Set the delivery mode to Fixed
        entry |= redir_flags::DELIVERY_FIXED;

        // Set the target mode to Physical
        entry |= redir_flags::DEST_PHYSICAL;

        // Set vector number
        entry |= vector as u64;

        // Write redirection table (unmask)
        self.write_redirection_entry(irq, entry);

        log_info!(
            "IO APIC {} IRQ {} -> Vector {} (APIC {}, {}, {})",
            self.id,
            irq,
            vector,
            dest_apic_id,
            if level_triggered { "level" } else { "edge" },
            if active_low { "low" } else { "high" }
        );
    }

    /// masked IRQ
    pub unsafe fn mask_irq(&mut self, irq: u8) {
        let mut entry = self.read_redirection_entry(irq);
        entry |= redir_flags::MASKED;
        self.write_redirection_entry(irq, entry);

        log_debug!("IO APIC {} IRQ {} masked", self.id, irq);
    }

    /// unmasked IRQ
    pub unsafe fn unmask_irq(&mut self, irq: u8) {
        let mut entry = self.read_redirection_entry(irq);
        entry &= !redir_flags::MASKED;
        self.write_redirection_entry(irq, entry);

        log_debug!("IO APIC {} IRQ {} unmasked", self.id, irq);
    }

    /// Print IO APIC information
    pub unsafe fn print_info(&mut self) {
        let id = self.read(reg::ID) >> 24;
        let version = self.read(reg::VER);
        let apic_ver = version & 0xFF;

        log_info!("IO APIC {} ID: {}", self.id, id);
        log_info!("IO APIC {} Version: {:#x}", self.id, apic_ver);
        log_info!("IO APIC {} Max Redirection Entries: {}", self.id, self.max_redirection_entries);
        log_info!("IO APIC {} GSI Base: {}", self.id, self.gsi_base);
    }
}

/// Initialize a single IO APIC (using mapped virtual address)
pub unsafe fn init_single_ioapic(base_vaddr: VirtAddr, id: u8, gsi_base: u32) {
    let mut ioapic = IoApic {
        base_vaddr,
        id,
        gsi_base,
        max_redirection_entries: 0,
    };

    // Read version information to get the maximum number of redirect entries
    let version = ioapic.read(reg::VER);
    ioapic.max_redirection_entries = ((version >> 16) & 0xFF) as u8 + 1;

    ioapic.init();
    ioapic.print_info();

    IO_APICS.lock().push(ioapic);
}

/// Initialize all IO APICs (from physical address, need to be mapped first)
pub unsafe fn init_io_apics(io_apics: &[(PhysAddr, u8, u32)]) {
    let mut apics = IO_APICS.lock();

    for (paddr, id, gsi_base) in io_apics {
        let mut ioapic = IoApic::new(*paddr, *id, *gsi_base);
        ioapic.init();
        ioapic.print_info();
        apics.push(ioapic);
    }

    log_info!("All IO APICs initialized");
}

/// Configuring IRQ Redirection
pub fn set_irq_redirect(
    irq: u8,
    vector: u8,
    dest_apic_id: u8,
    level_triggered: bool,
    active_low: bool,
) {
    unsafe {
        let mut apics = IO_APICS.lock();

        // Find the IO APIC responsible for this IRQ
        for ioapic in apics.iter_mut() {
            let gsi_start = ioapic.gsi_base as u8;
            let gsi_end = gsi_start + ioapic.max_redirection_entries;

            if irq >= gsi_start && irq < gsi_end {
                let local_irq = irq - gsi_start;
                ioapic.set_irq_redirect(local_irq, vector, dest_apic_id, level_triggered, active_low);
                return;
            }
        }

        log_warn!("No IO APIC found for IRQ {}", irq);
    }
}

/// Block IRQ
pub fn mask_irq(irq: u8) {
    unsafe {
        let mut apics = IO_APICS.lock();
        for ioapic in apics.iter_mut() {
            let gsi_start = ioapic.gsi_base as u8;
            let gsi_end = gsi_start + ioapic.max_redirection_entries;

            if irq >= gsi_start && irq < gsi_end {
                let local_irq = irq - gsi_start;
                ioapic.mask_irq(local_irq);
                return;
            }
        }
    }
}

/// Unmask IRQ
pub fn unmask_irq(irq: u8) {
    unsafe {
        let mut apics = IO_APICS.lock();
        for ioapic in apics.iter_mut() {
            let gsi_start = ioapic.gsi_base as u8;
            let gsi_end = gsi_start + ioapic.max_redirection_entries;

            if irq >= gsi_start && irq < gsi_end {
                let local_irq = irq - gsi_start;
                ioapic.unmask_irq(local_irq);
                return;
            }
        }
    }
}