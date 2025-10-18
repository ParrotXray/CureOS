use x86_64::VirtAddr;
use crate::log_warn;

pub mod ioapic;
pub mod lapic;

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

/// Local APIC register offset
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum ApicRegister {
    Id = 0x20,
    Version = 0x30,
    TaskPriority = 0x80,
    ProcessorPriority = 0xA0,
    Eoi = 0xB0,
    LogicalDestination = 0xD0,
    DestinationFormat = 0xE0,
    SpuriousInterruptVector = 0xF0,
    ErrorStatus = 0x280,
    LvtTimer = 0x320,
    LvtThermalSensor = 0x330,
    LvtPerformanceCounter = 0x340,
    LvtLint0 = 0x350,
    LvtLint1 = 0x360,
    LvtError = 0x370,
    TimerInitialCount = 0x380,
    TimerCurrentCount = 0x390,
    TimerDivideConfig = 0x3E0,
}

/// APIC configuration flags
#[allow(dead_code)]
pub mod flags {
    pub const APIC_ENABLE: u32 = 0x100;
    pub const APIC_SW_ENABLE: u32 = 0x100;
    pub const APIC_SPURIOUS_ALL: u32 = 0xFF;

    pub const LVT_MASKED: u32 = 1 << 16;
    pub const LVT_TIMER_PERIODIC: u32 = 1 << 17;
    pub const LVT_TIMER_ONESHOT: u32 = 0 << 17;
}

// Local APIC base address (read-only after initialization)
static mut LOCAL_APIC_BASE: Option<VirtAddr> = None;

// APIC information (read-only after initialization)
static mut APIC_INFO: ApicInfo = ApicInfo::new();

struct ApicInfo {
    id: u32,
    version: u32,
    max_lvt: u32,
}

impl ApicInfo {
    const fn new() -> Self {
        Self {
            id: 0,
            version: 0,
            max_lvt: 0,
        }
    }
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