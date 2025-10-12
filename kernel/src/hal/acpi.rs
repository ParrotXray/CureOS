use acpi::{aml, AcpiTables, Handle, Handler, PciAddress, PhysicalMapping};
use acpi::platform::{AcpiPlatform, interrupt::InterruptModel, PciConfigRegions};
use acpi::sdt::hpet::HpetInfo;
use acpi::rsdp::Rsdp;
use core::ptr::NonNull;
use core::mem;
use crate::kprintln;
use crate::{log_trace, log_debug, log_info, log_warn, log_error, log_fatal};

#[derive(Clone, Copy)]
pub struct CureAcpiHandler {
    physical_memory_offset: u64,
}

impl CureAcpiHandler {
    pub const fn new(physical_memory_offset: u64) -> Self {
        Self {
            physical_memory_offset,
        }
    }
}

impl Handler for CureAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        // Bootloader 已經映射了所有物理記憶體
        let virtual_address = physical_address as u64 + self.physical_memory_offset;
        let virtual_start = NonNull::new((virtual_address) as *mut T).unwrap();

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start,
            region_length: size,
            mapped_length: size,
            handler: *self,
        }
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {
        //
    }

    fn read_u8(&self, address: usize) -> u8 {
        unsafe {
            let ptr = self.map_physical_region::<u8>(address, 1);
            core::ptr::read_volatile(ptr.virtual_start.as_ptr())
        }
    }

    fn read_u16(&self, address: usize) -> u16 {
        unsafe {
            let ptr = self.map_physical_region::<u16>(address, 2);
            core::ptr::read_volatile(ptr.virtual_start.as_ptr())
        }
    }

    fn read_u32(&self, address: usize) -> u32 {
        unsafe {
            let ptr = self.map_physical_region::<u32>(address, 4);
            core::ptr::read_volatile(ptr.virtual_start.as_ptr())
        }
    }

    fn read_u64(&self, address: usize) -> u64 {
        unsafe {
            let ptr = self.map_physical_region::<u64>(address, 8);
            core::ptr::read_volatile(ptr.virtual_start.as_ptr())
        }
    }

    fn write_u8(&self, address: usize, value: u8) {
        unsafe {
            let ptr = self.map_physical_region::<u8>(address, 1);
            core::ptr::write_volatile(ptr.virtual_start.as_ptr(), value);
        }
    }

    fn write_u16(&self, address: usize, value: u16) {
        unsafe {
            let ptr = self.map_physical_region::<u16>(address, 2);
            core::ptr::write_volatile(ptr.virtual_start.as_ptr(), value);
        }
    }

    fn write_u32(&self, address: usize, value: u32) {
        unsafe {
            let ptr = self.map_physical_region::<u32>(address, 4);
            core::ptr::write_volatile(ptr.virtual_start.as_ptr(), value);
        }
    }

    fn write_u64(&self, address: usize, value: u64) {
        unsafe {
            let ptr = self.map_physical_region::<u64>(address, 8);
            core::ptr::write_volatile(ptr.virtual_start.as_ptr(), value);
        }
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        unsafe { crate::hal::io::io_port_rb(port) }
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        unsafe { crate::hal::io::io_port_rw(port) }
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        unsafe { crate::hal::io::io_port_rl(port) }
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        unsafe { crate::hal::io::io_port_wb(port, value) };
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        unsafe { crate::hal::io::io_port_ww(port, value) };
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        unsafe { crate::hal::io::io_port_wl(port, value) };
    }

    fn read_pci_u8(&self, address: PciAddress, offset: u16) -> u8 {
        // TODO: 實作 PCI 配置空間讀取
        0xFF
    }

    fn read_pci_u16(&self, address: PciAddress, offset: u16) -> u16 {
        // TODO: 實作 PCI 配置空間讀取
        0xFFFF
    }

    fn read_pci_u32(&self, address: PciAddress, offset: u16) -> u32 {
        // TODO: 實作 PCI 配置空間讀取
        0xFFFFFFFF
    }

    fn write_pci_u8(&self, address: PciAddress, offset: u16, value: u8) {
        // TODO: 實作 PCI 配置空間寫入
    }

    fn write_pci_u16(&self, address: PciAddress, offset: u16, value: u16) {
        // TODO: 實作 PCI 配置空間寫入
    }

    fn write_pci_u32(&self, address: PciAddress, offset: u16, value: u32) {
        // TODO: 實作 PCI 配置空間寫入
    }

    fn nanos_since_boot(&self) -> u64 {
        // TODO: 實作高精度計時器 (需要 HPET 或 TSC)
        // 目前返回 0
        0
    }

    fn stall(&self, _microseconds: u64) {
        // TODO: 實作微秒級延遲
        // 簡單的忙等待實作
        for _ in 0..(_microseconds * 1000) {
            crate::hal::cpu::cpu_pause();
        }
    }

    fn sleep(&self, _milliseconds: u64) {
        // TODO: 實作毫秒級睡眠
        // 簡單的忙等待實作
        self.stall(_milliseconds * 1000);
    }

    fn create_mutex(&self) -> acpi::Handle {
        // TODO: 實作 Mutex
        // 目前返回一個假的 handle
        acpi::Handle(0)
    }

    fn acquire(&self, mutex: Handle, timeout: u16) -> Result<(), aml::AmlError> {
        // TODO: 實作 Mutex 獲取
        // 暫時直接返回成功
        Ok(())
    }

    fn release(&self, _handle: acpi::Handle) {
        // TODO: 實作 Mutex 釋放
    }
}

pub struct AcpiInfo {
    pub revision: u8,
    pub boot_processor: Option<u32>,
    pub cpu_count: usize,
    pub has_apic: bool,
    pub has_hpet: bool,
}

pub fn init(rsdp_addr: u64, physical_memory_offset: u64) -> Option<AcpiInfo> {

    let handler = CureAcpiHandler::new(physical_memory_offset);

    let rsdp_mapping = unsafe {
        handler.map_physical_region::<Rsdp>(rsdp_addr as usize, mem::size_of::<Rsdp>())
    };
    let revision = rsdp_mapping.revision();
    log_info!("ACPI Revision: {}", revision);

    let tables = unsafe {
        match AcpiTables::from_rsdp(handler, rsdp_addr as usize) {
            Ok(tables) => tables,
            Err(e) => {
                log_error!("Failed to parse ACPI tables: {:?}", e);
                return None;
            }
        }
    };
    // kprintln!("  ACPI Revision: {}", tables.rsdp_revision);

    let platform = match AcpiPlatform::new(tables, handler) {
        Ok(platform) => platform,
        Err(e) => {
            log_error!("Failed to create ACPI platform: {:?}", e);
            return None;
        }
    };

    log_info!("Power Profile: {:?}", platform.power_profile);

    let (boot_processor, cpu_count) = if let Some(proc_info) = &platform.processor_info {
        let boot_proc = Some(proc_info.boot_processor.processor_uid);
        let cpu_cnt = proc_info.application_processors.len() + 1;

        log_info!("Boot Processor UID: {:?}", boot_proc);
        log_info!("Total CPU Count: {}", cpu_cnt);

        (boot_proc, cpu_cnt)
    } else {
        log_warn!("No processor info found");
        (None, 0)
    };

    // 檢查中斷模型
    let has_apic = match &platform.interrupt_model {
        InterruptModel::Apic(apic) => {
            log_info!("Local APIC Address: {:#x}", apic.local_apic_address);
            log_info!("IO APICs: {} controller(s)", apic.io_apics.len());

            for (i, io_apic) in apic.io_apics.iter().enumerate() {
                log_info!("IO APIC {}: ID={}, Address={:#x}, GSI Base={}",
                    i, io_apic.id, io_apic.address, io_apic.global_system_interrupt_base);
            }

            true
        }
        InterruptModel::Unknown => {
            log_warn!("Interrupt Model: Unknown (not APIC)");
            false
        }
        _ => {
            log_warn!("Interrupt Model: Other");
            false
        }
    };

    let has_hpet = match HpetInfo::new(&platform.tables) {
        Ok(hpet) => {
            log_info!("Base Address: {:#x}", hpet.base_address);
            log_info!("Hardware Rev: {}", hpet.hardware_rev);
            log_info!("Comparator Count: {}", hpet.num_comparators);
            log_info!("Counter Size: {} bit", if hpet.main_counter_is_64bits { 64 } else { 32 });
            log_info!("Legacy IRQ Capable: {}", hpet.legacy_irq_capable);
            log_info!("PCI Vendor ID: {:#x}", hpet.pci_vendor_id);
            true
        }
        Err(_) => {
            log_warn!("HPET: Not available");
            false
        }
    };

    if let Ok(mcfg) = PciConfigRegions::new(&platform.tables) {
        for (i, entry) in mcfg.regions.iter().enumerate() {
            let segment_group = entry.pci_segment_group;
            let base_addr = entry.base_address;
            let bus_start = entry.bus_number_start;
            let bus_end = entry.bus_number_end;

            log_info!("Entry {}: Segment Group {}", i, segment_group);
            log_info!("Base Address: {:#x}", base_addr);
            log_info!("Bus Range: {}-{}", bus_start, bus_end);
        }
    }

    log_info!("ACPI initialized successfully!");

    Some(AcpiInfo {
        revision,
        boot_processor,
        cpu_count,
        has_apic,
        has_hpet,
    })
}

pub fn print_info(info: &AcpiInfo) {
    kprintln!();
    log_info!("Revision: ACPI {}.0", info.revision);
    log_info!("CPUs: {} processor(s)", info.cpu_count);
    if let Some(boot_proc) = info.boot_processor {
        log_info!("Boot Processor: UID {}", boot_proc);
    }
    log_info!("APIC: {}", if info.has_apic { "Available " } else { "Not available" });
    log_info!("HPET: {}", if info.has_hpet { "Available " } else { "Not available" });
}