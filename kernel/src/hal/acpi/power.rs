use super::{AcpiPowerInfo, CureAcpiHandler, ResetRegister, ACPI_POWER_INFO, ACPI_RESET_REG};
use crate::hal::{cpu, io};
use crate::kprintln;
use crate::mm::vma;
use crate::{log_debug, log_error, log_fatal, log_info, log_trace, log_warn};
use acpi::platform::{interrupt::InterruptModel, AcpiPlatform, PciConfigRegions};
use acpi::sdt::{SdtHeader, Signature};
use acpi::{aml, sdt, AcpiTables, Handle, Handler, PciAddress, PhysicalMapping};
use core::mem;

/// Extract shutdown information from ACPI table
pub fn extract_power_info(
    tables: &AcpiTables<CureAcpiHandler>,
    handler: &CureAcpiHandler,
) -> Option<AcpiPowerInfo> {
    log_info!("Extracting ACPI power management info...");

    let fadt = match tables.find_table::<sdt::fadt::Fadt>() {
        Some(fadt) => fadt,
        None => {
            log_error!("Failed to find FADT");
            return None;
        }
    };

    unsafe {
        let fadt_ptr = (&*fadt as *const sdt::fadt::Fadt) as *const u8;

        let pm1a_control_block = core::ptr::read_unaligned(fadt_ptr.add(64) as *const u32);
        let pm1b_control_block = core::ptr::read_unaligned(fadt_ptr.add(68) as *const u32);

        let fadt_revision = core::ptr::read_unaligned(fadt_ptr.add(8) as *const u8);
        let dsdt_address = if fadt_revision >= 2 {
            let x_dsdt = core::ptr::read_unaligned(fadt_ptr.add(140) as *const u64);
            if x_dsdt != 0 {
                x_dsdt
            } else {
                core::ptr::read_unaligned(fadt_ptr.add(40) as *const u32) as u64
            }
        } else {
            core::ptr::read_unaligned(fadt_ptr.add(40) as *const u32) as u64
        };

        log_info!("PM1a Control Block: {:#x}", pm1a_control_block);
        if pm1b_control_block != 0 {
            log_info!("PM1b Control Block: {:#x}", pm1b_control_block);
        }
        log_debug!("DSDT address: {:#x}", dsdt_address);

        let (slp_typa, slp_typb) = match parse_s5_object(dsdt_address, handler) {
            Some(values) => values,
            None => {
                log_warn!("Could not parse _S5 object, using default values");
                (5, 5)
            }
        };

        log_info!("SLP_TYPa: {:#x}", slp_typa);
        log_info!("SLP_TYPb: {:#x}", slp_typb);

        Some(AcpiPowerInfo {
            pm1a_control_block,
            pm1b_control_block,
            slp_typa,
            slp_typb,
            slp_en: 1 << 13,
        })
    }
}

/// Parse the _S5 object in the DSDT
///
/// AML bytecode format for the _S5 object:
/// ```
/// Name(_S5, Package() {
/// SLP_TYPa, // Type A for entering the S5 state
/// SLP_TYPb, // Type B for entering the S5 state
/// ...
/// })
/// ```
fn parse_s5_object(dsdt_phys_addr: u64, handler: &CureAcpiHandler) -> Option<(u16, u16)> {
    log_debug!("Parsing DSDT at physical address {:#x}", dsdt_phys_addr);

    // Map the DSDT header first to get the length
    let dsdt_header_mapping = unsafe {
        handler.map_physical_region::<SdtHeader>(dsdt_phys_addr as usize, size_of::<SdtHeader>())
    };

    if dsdt_header_mapping.signature != Signature::DSDT {
        log_error!("Invalid DSDT signature");
        return None;
    }

    let dsdt_length = dsdt_header_mapping.length as usize;
    log_debug!("DSDT length: {} bytes", dsdt_length);

    // Release the header mapping
    drop(dsdt_header_mapping);

    // 映射完整的 DSDT
    let dsdt_mapping =
        unsafe { handler.map_physical_region::<u8>(dsdt_phys_addr as usize, dsdt_length) };

    unsafe {
        let dsdt_ptr = dsdt_mapping.virtual_start.as_ptr();
        let dsdt_data = core::slice::from_raw_parts(dsdt_ptr, dsdt_length);

        // Search DSDT for "_S5_"
        let s5_name = b"_S5_";

        for i in 0..(dsdt_data.len() - 4) {
            if &dsdt_data[i..i + 4] == s5_name {
                log_debug!("Found _S5 at offset {:#x}", i);

                let debug_range = i..core::cmp::min(i + 32, dsdt_data.len());
                log_debug!("_S5 region bytes: {:02x?}", &dsdt_data[debug_range]);

                let mut offset = i + 4; // Skip "_S5_"

                // Skip any intermediate bytes and go straight to PackageOp
                let search_limit = offset + 16;
                while offset < search_limit && offset < dsdt_data.len() {
                    if dsdt_data[offset] == 0x12 {
                        log_debug!("Found PackageOp at offset {:#x}", offset);
                        break;
                    }
                    offset += 1;
                }

                if offset >= dsdt_data.len() || dsdt_data[offset] != 0x12 {
                    log_warn!("PackageOp not found after _S5");
                    continue;
                }

                offset += 1; // Skip PackageOp (0x12)

                log_debug!("Found _S5 at offset {:#x}", i);
                log_debug!("Found _S5 at offset {:#x}", i);

                // Parse PkgLength
                let pkg_length = parse_pkg_length(&dsdt_data[offset..]);
                let pkg_length_size = get_pkg_length_size(&dsdt_data[offset..]);
                log_debug!("PkgLength: {}, size: {}", pkg_length, pkg_length_size);
                offset += pkg_length_size;

                // NumElements
                if offset >= dsdt_data.len() {
                    log_warn!("Unexpected end of data");
                    continue;
                }
                let num_elements = dsdt_data[offset];
                offset += 1;

                log_debug!("Package elements: {}", num_elements);

                if num_elements < 2 {
                    log_warn!("_S5 package has less than 2 elements");
                    continue;
                }

                log_debug!(
                    "Current offset: {:#x}, next bytes: {:02x?}",
                    offset,
                    &dsdt_data[offset..core::cmp::min(offset + 8, dsdt_data.len())]
                );

                // Extract SLP_TYPa
                log_debug!(
                    "Reading SLP_TYPa at offset {:#x}, byte: {:#x}",
                    offset,
                    dsdt_data[offset]
                );
                let slp_typa = parse_aml_integer(&dsdt_data[offset..]).unwrap_or(0);
                log_debug!("SLP_TYPa parsed: {:#x}", slp_typa);
                let typa_size = get_aml_integer_size(&dsdt_data[offset..]);
                offset += typa_size;

                // Extract SLP_TYPb
                log_debug!(
                    "Reading SLP_TYPb at offset {:#x}, byte: {:#x}",
                    offset,
                    dsdt_data[offset]
                );
                let slp_typb = parse_aml_integer(&dsdt_data[offset..]).unwrap_or(0);
                log_debug!("SLP_TYPb parsed: {:#x}", slp_typb);

                log_info!(
                    "Parsed _S5: SLP_TYPa={:#x}, SLP_TYPb={:#x}",
                    slp_typa,
                    slp_typb
                );

                return Some((slp_typa as u16, slp_typb as u16));
            }
        }
        log_error!("_S5 object not found in DSDT");
        None
    }
}

/// Parse AML packet length
fn parse_pkg_length(data: &[u8]) -> usize {
    if data.is_empty() {
        return 0;
    }

    let lead_byte = data[0];
    let byte_count = (lead_byte >> 6) as usize;

    match byte_count {
        0 => (lead_byte & 0x3F) as usize,
        1 => {
            if data.len() < 2 {
                return 0;
            }
            ((lead_byte & 0x0F) as usize) | ((data[1] as usize) << 4)
        }
        2 => {
            if data.len() < 3 {
                return 0;
            }
            ((lead_byte & 0x0F) as usize) | ((data[1] as usize) << 4) | ((data[2] as usize) << 12)
        }
        3 => {
            if data.len() < 4 {
                return 0;
            }
            ((lead_byte & 0x0F) as usize)
                | ((data[1] as usize) << 4)
                | ((data[2] as usize) << 12)
                | ((data[3] as usize) << 20)
        }
        _ => 0,
    }
}

/// Get the number of bytes encoded by the packet length
fn get_pkg_length_size(data: &[u8]) -> usize {
    if data.is_empty() {
        return 0;
    }

    let lead_byte = data[0];
    let byte_count = (lead_byte >> 6) as usize;
    1 + byte_count
}

/// Parse AML integer
fn parse_aml_integer(data: &[u8]) -> Option<u64> {
    if data.is_empty() {
        return None;
    }

    match data[0] {
        0x00 => Some(0),          // ZeroOp
        0x01 => Some(1),          // OneOp
        0xFF => Some(0xFFFFFFFF), // OnesOp
        0x0A => {
            // BytePrefix
            if data.len() < 2 {
                return None;
            }
            Some(data[1] as u64)
        }
        0x0B => {
            // WordPrefix
            if data.len() < 3 {
                return None;
            }
            Some(u16::from_le_bytes([data[1], data[2]]) as u64)
        }
        0x0C => {
            // DWordPrefix
            if data.len() < 5 {
                return None;
            }
            Some(u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as u64)
        }
        0x0E => {
            // QWordPrefix
            if data.len() < 9 {
                return None;
            }
            Some(u64::from_le_bytes([
                data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            ]))
        }
        _ => {
            log_warn!("Unknown AML integer prefix: {:#02x}", data[0]);
            None
        }
    }
}

/// Get the byte size of the AML integer
fn get_aml_integer_size(data: &[u8]) -> usize {
    if data.is_empty() {
        return 0;
    }

    match data[0] {
        0x00 | 0x01 | 0xFF => 1, // Zero, One, Ones
        0x0A => 2,               // Byte
        0x0B => 3,               // Word
        0x0C => 5,               // DWord
        0x0E => 9,               // QWord
        _ => 1,
    }
}

/// Perform ACPI shutdown
pub fn acpi_shutdown() -> bool {
    log_info!("Attempting ACPI shutdown...");

    let power_info = unsafe {
        match &ACPI_POWER_INFO {
            Some(info) => info,
            None => {
                log_error!("ACPI power info not initialized!");
                return false;
            }
        }
    };

    unsafe {
        let slp_cmd_a = (power_info.slp_typa << 10) | power_info.slp_en;

        log_info!(
            "Writing {:#x} to PM1a_CNT ({:#x})",
            slp_cmd_a,
            power_info.pm1a_control_block
        );

        // Write to PM1a control register
        io::io_port_ww(power_info.pm1a_control_block as u16, slp_cmd_a);

        // If PM1b exists, also write
        if power_info.pm1b_control_block != 0 {
            let slp_cmd_b = (power_info.slp_typb << 10) | power_info.slp_en;
            log_info!(
                "Writing {:#x} to PM1b_CNT ({:#x})",
                slp_cmd_b,
                power_info.pm1b_control_block
            );
            io::io_port_ww(power_info.pm1b_control_block as u16, slp_cmd_b);
        }

        cpu::cpu_pause(10000);
    }

    log_error!("ACPI shutdown failed!");
    false
}

/// Store ACPI shutdown information
pub fn store_power_info(info: AcpiPowerInfo) {
    unsafe {
        ACPI_POWER_INFO = Some(info);
    }
    log_info!("ACPI power info stored successfully");
}

/// Extract reset register information from FADT
pub fn extract_reset_reg(tables: &AcpiTables<CureAcpiHandler>) -> Option<ResetRegister> {
    log_info!("Extracting ACPI reset register info...");

    let fadt = match tables.find_table::<sdt::fadt::Fadt>() {
        Some(fadt) => fadt,
        None => {
            log_error!("Failed to find FADT");
            return None;
        }
    };

    unsafe {
        let fadt_ptr = (&*fadt as *const sdt::fadt::Fadt) as *const u8;

        let fadt_revision = core::ptr::read_unaligned(fadt_ptr.add(8) as *const u8);

        if fadt_revision < 2 {
            log_warn!("FADT revision {} does not support RESET_REG", fadt_revision);
            return None;
        }

        // 讀取 Flags (offset 112 in FADT)
        let flags = core::ptr::read_unaligned(fadt_ptr.add(112) as *const u32);
        let reset_reg_supported = (flags & (1 << 10)) != 0;

        if !reset_reg_supported {
            log_warn!("RESET_REG not supported (FADT flags bit 10 not set)");
            return None;
        }

        // RESET_REG is at offset 116 in the FADT
        // Generic Address Structure format:
        // +0: Address Space ID (1 byte)
        // +1: Register Bit Width (1 byte)
        // +2: Register Bit Offset (1 byte)
        // +3: Access Size (1 byte)
        // +4: Address (8 bytes)

        let reset_reg_offset = 116;

        let address_space = core::ptr::read_unaligned(fadt_ptr.add(reset_reg_offset) as *const u8);
        let bit_width = core::ptr::read_unaligned(fadt_ptr.add(reset_reg_offset + 1) as *const u8);
        let bit_offset = core::ptr::read_unaligned(fadt_ptr.add(reset_reg_offset + 2) as *const u8);
        let address = core::ptr::read_unaligned(fadt_ptr.add(reset_reg_offset + 4) as *const u64);

        // RESET_VALUE is after RESET_REG (offset 128)
        let reset_value = core::ptr::read_unaligned(fadt_ptr.add(128) as *const u8);

        if bit_width != 8 || bit_offset != 0 {
            log_warn!(
                "Invalid RESET_REG configuration: width={}, offset={}",
                bit_width,
                bit_offset
            );
            return None;
        }

        if address_space > 2 {
            log_warn!("Invalid address space ID: {}", address_space);
            return None;
        }

        log_info!("RESET_REG found:");
        log_info!(
            "  Address Space: {} ({})",
            address_space,
            match address_space {
                0 => "System Memory",
                1 => "System I/O",
                2 => "PCI Config",
                _ => "Unknown",
            }
        );
        log_info!("Address: {:#x}", address);
        log_info!("Reset Value: {:#x}", reset_value);

        Some(ResetRegister {
            address_space,
            address,
            value: reset_value,
        })
    }
}

pub fn store_reset_reg(reset_reg: ResetRegister) {
    unsafe {
        ACPI_RESET_REG = Some(reset_reg);
    }
    log_info!("ACPI reset register info stored");
}

/// Restart using ACPI RESET_REG
pub fn acpi_reset_reg_reboot() -> bool {
    unsafe {
        let reset_reg = match &ACPI_RESET_REG {
            Some(reg) => reg,
            None => {
                log_debug!("ACPI RESET_REG not available");
                return false;
            }
        };

        log_info!("Using ACPI RESET_REG for reboot");
        log_info!(
            "  Space: {}, Address: {:#x}, Value: {:#x}",
            reset_reg.address_space,
            reset_reg.address,
            reset_reg.value
        );

        match reset_reg.address_space {
            // System I/O
            1 => {
                log_debug!(
                    "Writing {:#x} to I/O port {:#x}",
                    reset_reg.value,
                    reset_reg.address
                );
                io::io_port_wb(reset_reg.address as u16, reset_reg.value);
                true
            }

            // System Memory
            0 => {
                log_debug!(
                    "Writing {:#x} to memory address {:#x}",
                    reset_reg.value,
                    reset_reg.address
                );

                let virt_addr = vma::phys_to_virt(reset_reg.address);
                let ptr = virt_addr.as_mut_ptr::<u8>();
                core::ptr::write_volatile(ptr, reset_reg.value);
                true
            }

            // PCI Config Space
            2 => {
                log_debug!(
                    "Writing {:#x} to PCI config space {:#x}",
                    reset_reg.value,
                    reset_reg.address
                );

                // PCI address encoding (ACPI format):
                // Bits 63-32: Reserved
                // Bits 31-16: Bus Number
                // Bits 15-11: Device Number
                // Bits 10-8: Function Number
                // Bits 7-0: Register Offset

                let bus = ((reset_reg.address >> 16) & 0xFFFF) as u8;
                let device = ((reset_reg.address >> 11) & 0x1F) as u8;
                let function = ((reset_reg.address >> 8) & 0x7) as u8;
                let offset = (reset_reg.address & 0xFF) as u8;

                log_debug!(
                    "PCI Bus={}, Dev={}, Func={}, Offset={:#x}",
                    bus,
                    device,
                    function,
                    offset
                );

                // TODO: 調用你的 PCI 配置空間寫入函數
                // pci_config_write_byte(bus, device, function, offset, reset_reg.value);
                false
            }

            _ => {
                log_error!("Unknown address space: {}", reset_reg.address_space);
                false
            }
        }
    }
}
