pub mod init;
pub mod power;

use crate::hal::{cpu, io};
use acpi::{aml, Handle, Handler, PciAddress, PhysicalMapping};
use core::ptr::NonNull;

#[derive(Clone, Copy)]
pub struct CureAcpiHandler {
    physical_memory_offset: u64,
}

pub struct AcpiInfo {
    pub revision: u8,
    pub boot_processor: Option<u32>,
    pub cpu_count: usize,
    pub has_apic: bool,
    pub has_hpet: bool,
    pub local_apic_address: Option<u64>,
    pub io_apics: alloc::vec::Vec<(u64, u8, u32)>, // (address, id, gsi_base)
}

/// Information required for ACPI shutdown
pub struct AcpiPowerInfo {
    pub pm1a_control_block: u32,
    pub pm1b_control_block: u32,
    pub slp_typa: u16,
    pub slp_typb: u16,
    pub slp_en: u16,
}

/// ACPI 重置寄存器信息
#[derive(Debug, Clone, Copy)]
pub struct ResetRegister {
    pub address_space: u8,  // 0=SystemMemory, 1=SystemIO, 2=PciConfig
    pub address: u64,
    pub value: u8,
}

static mut ACPI_RESET_REG: Option<ResetRegister> = None;

static mut ACPI_POWER_INFO: Option<AcpiPowerInfo> = None;

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
        // Bootloader has mapped all physical memory
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
        unsafe { io::io_port_rb(port) }
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        unsafe { io::io_port_rw(port) }
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        unsafe { io::io_port_rl(port) }
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        unsafe { io::io_port_wb(port, value) };
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        unsafe { io::io_port_ww(port, value) };
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        unsafe { io::io_port_wl(port, value) };
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
        cpu::cpu_pause(_microseconds * 1000);
    }

    fn sleep(&self, _milliseconds: u64) {
        // TODO: 實作毫秒級睡眠
        // 簡單的忙等待實作
        self.stall(_milliseconds * 1000);
    }

    fn create_mutex(&self) -> Handle {
        // TODO: 實作 Mutex
        // 目前返回一個假的 handle
        Handle(0)
    }

    fn acquire(&self, mutex: Handle, timeout: u16) -> Result<(), aml::AmlError> {
        // TODO: 實作 Mutex 獲取
        // 暫時直接返回成功
        Ok(())
    }

    fn release(&self, _handle: Handle) {
        // TODO: 實作 Mutex 釋放
    }
}
