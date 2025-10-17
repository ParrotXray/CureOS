// kernel/src/hal/lapic.rs - 添加公開 API

use x86_64::{PhysAddr, VirtAddr};
use spin::Mutex;
use crate::{log_trace, log_debug, log_info, log_warn, log_error};
use crate::mm::vma;

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

pub struct LocalApic {
    base_vaddr: VirtAddr,
}

static LOCAL_APIC: Mutex<Option<LocalApic>> = Mutex::new(None);

impl LocalApic {
    /// Create Local APIC from physical address
    pub unsafe fn new(base_paddr: PhysAddr) -> Self {
        let base_vaddr = vma::phys_to_virt(base_paddr.as_u64());

        log_debug!("Local APIC physical base: {:#x}", base_paddr.as_u64());
        log_debug!("Local APIC virtual base: {:#x}", base_vaddr.as_u64());

        Self { base_vaddr }
    }

    /// Read APIC registers
    pub unsafe fn read(&self, reg: ApicRegister) -> u32 {
        let addr = self.base_vaddr.as_u64() + reg as u64;
        core::ptr::read_volatile(addr as *const u32)
    }

    /// Write to APIC register
    pub unsafe fn write(&mut self, reg: ApicRegister, value: u32) {
        let addr = self.base_vaddr.as_u64() + reg as u64;
        core::ptr::write_volatile(addr as *mut u32, value);
    }

    /// Get base virtual address
    pub fn base_vaddr(&self) -> VirtAddr {
        self.base_vaddr
    }

    /// Initialize Local APIC
    pub unsafe fn init(&mut self) {
        // Enable APIC (via Spurious Interrupt Vector Register)
        let spurious = flags::APIC_SW_ENABLE | 0xFF;
        self.write(ApicRegister::SpuriousInterruptVector, spurious);

        // Set task priority to 0 (accept all interrupts)
        self.write(ApicRegister::TaskPriority, 0);

        // Configure LVT entries - mask all local interrupts by default
        self.write(ApicRegister::LvtTimer, flags::LVT_MASKED);
        self.write(ApicRegister::LvtLint0, flags::LVT_MASKED);
        self.write(ApicRegister::LvtLint1, flags::LVT_MASKED);
        self.write(ApicRegister::LvtError, flags::LVT_MASKED);
        self.write(ApicRegister::LvtPerformanceCounter, flags::LVT_MASKED);
        self.write(ApicRegister::LvtThermalSensor, flags::LVT_MASKED);

        log_info!("Local APIC initialized");
    }

    /// Obtaining the APIC ID
    pub unsafe fn id(&self) -> u32 {
        self.read(ApicRegister::Id) >> 24
    }

    /// Get the APIC version
    pub unsafe fn version(&self) -> u32 {
        self.read(ApicRegister::Version)
    }

    /// Send EOI (End of Interrupt)
    pub unsafe fn send_eoi(&mut self) {
        self.write(ApicRegister::Eoi, 0);
    }

    /// Configuring Timers
    pub unsafe fn setup_timer(&mut self, vector: u8, divide_config: u32, initial_count: u32) {
        // Setting the crossover
        self.write(ApicRegister::TimerDivideConfig, divide_config);

        // Setting the LVT Timer (periodic mode)
        let lvt = flags::LVT_TIMER_PERIODIC | (vector as u32);
        self.write(ApicRegister::LvtTimer, lvt);

        // Set the initial count
        self.write(ApicRegister::TimerInitialCount, initial_count);

        log_info!("Local APIC timer configured: vector={}, count={}", vector, initial_count);
    }

    pub unsafe fn print_info(&self) {
        let id = self.id();
        let version = self.version();
        let max_lvt = (version >> 16) & 0xFF;
        let apic_version = version & 0xFF;

        log_info!("Local APIC ID: {}", id);
        log_info!("Local APIC Version: {:#x}", apic_version);
        log_info!("Max LVT Entry: {}", max_lvt);
    }
}

/// Initialize Local APIC (using mapped virtual address)
pub unsafe fn init_local_apic_with_vaddr(base_vaddr: VirtAddr) {
    let mut apic = LocalApic { base_vaddr };
    apic.init();
    apic.print_info();

    *LOCAL_APIC.lock() = Some(apic);
}

/// Initialize Local APIC (from physical address, needs to be mapped first)
pub unsafe fn init_local_apic(base_paddr: PhysAddr) {
    let base_vaddr = vma::phys_to_virt(base_paddr.as_u64());
    init_local_apic_with_vaddr(base_vaddr);
}

/// Send EOI
pub fn send_eoi() {
    unsafe {
        if let Some(apic) = LOCAL_APIC.lock().as_mut() {
            apic.send_eoi();
        }
    }
}

/// Get the Local APIC ID
pub fn get_apic_id() -> Option<u32> {
    unsafe {
        LOCAL_APIC.lock().as_ref().map(|apic| apic.id())
    }
}

/// Get the Local APIC base virtual address
pub fn get_base_vaddr() -> Option<VirtAddr> {
    LOCAL_APIC.lock().as_ref().map(|apic| apic.base_vaddr())
}

/// 公開的 APIC 寄存器讀取 API
///
/// # Safety
/// 調用者必須確保 APIC 已正確初始化
pub unsafe fn read_apic_reg(reg: ApicRegister) -> Option<u32> {
    LOCAL_APIC.lock().as_ref().map(|apic| apic.read(reg))
}

/// 公開的 APIC 寄存器寫入 API
///
/// # Safety
/// 調用者必須確保 APIC 已正確初始化
pub unsafe fn write_apic_reg(reg: ApicRegister, value: u32) -> bool {
    if let Some(apic) = LOCAL_APIC.lock().as_mut() {
        apic.write(reg, value);
        true
    } else {
        false
    }
}

/// 直接通過偏移量讀取 APIC 寄存器（用於 apic_timer）
///
/// # Safety
/// 調用者必須確保 APIC 已正確初始化且偏移量有效
pub unsafe fn read_apic_reg_raw(offset: u32) -> Option<u32> {
    LOCAL_APIC.lock().as_ref().map(|apic| {
        let addr = apic.base_vaddr.as_u64() + offset as u64;
        core::ptr::read_volatile(addr as *const u32)
    })
}

/// 直接通過偏移量寫入 APIC 寄存器（用於 apic_timer）
///
/// # Safety
/// 調用者必須確保 APIC 已正確初始化且偏移量有效
pub unsafe fn write_apic_reg_raw(offset: u32, value: u32) -> bool {
    if let Some(apic) = LOCAL_APIC.lock().as_ref() {
        let addr = apic.base_vaddr.as_u64() + offset as u64;
        core::ptr::write_volatile(addr as *mut u32, value);
        true
    } else {
        false
    }
}

/// Disable legacy 8259 PIC
/// This function should be called before using the APIC to avoid conflicts.
pub fn disable_legacy_pic() {
    use crate::hal::io::io_port_wb;

    unsafe {
        // Remap PIC to unused interrupt vector
        // Master PIC
        io_port_wb(0x20, 0x11); // ICW1: initialization
        io_port_wb(0x21, 0x20); // ICW2: Interrupt vector offset (32-39)
        io_port_wb(0x21, 0x04); // ICW3: Tell the Master PIC Slave to be on IRQ2
        io_port_wb(0x21, 0x01); // ICW4: 8086 mode

        // Slave PIC
        io_port_wb(0xA0, 0x11); // ICW1: initialization
        io_port_wb(0xA1, 0x28); // ICW2: Interrupt vector offset (40-47)
        io_port_wb(0xA1, 0x02); // ICW3: Tell the Slave PIC to connect to Master IRQ2
        io_port_wb(0xA1, 0x01); // ICW4: 8086 mode

        // Block all IRQs (disable PIC)
        io_port_wb(0x21, 0xFF);
        io_port_wb(0xA1, 0xFF);
    }

    log_info!("Legacy 8259 PIC disabled");
}