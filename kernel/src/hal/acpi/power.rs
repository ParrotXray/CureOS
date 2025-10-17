use acpi::{aml, sdt, AcpiTables, Handle, Handler, PciAddress, PhysicalMapping};
use acpi::platform::{AcpiPlatform, interrupt::InterruptModel, PciConfigRegions};
use core::mem;
use crate::kprintln;
use crate::{log_trace, log_debug, log_info, log_warn, log_error, log_fatal};
use crate::hal::{cpu, io};
use crate::mm::vma;
use super::*;


/// Extract shutdown information from ACPI table
pub fn extract_power_info(tables: &AcpiTables<CureAcpiHandler>) -> Option<AcpiPowerInfo> {
    log_info!("Extracting ACPI power management info...");

    let fadt = match tables.find_table::<sdt::fadt::Fadt>() {
        Some(fadt) => fadt,
        None => {
            log_error!("Failed to find FADT");
            return None;
        }
    };

    log_debug!("FADT found");

    // Get the control block address and DSDT address from FADT
    unsafe {
        // Get the raw pointer of FADT to read the fields manually
        let fadt_ptr = (&*fadt as *const sdt::fadt::Fadt) as *const u8;

        // FADT structure offset
        let pm1a_control_block = core::ptr::read_unaligned(fadt_ptr.add(64) as *const u32);
        let pm1b_control_block = core::ptr::read_unaligned(fadt_ptr.add(68) as *const u32);

        // Read DSDT address
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

        // Parse the _S5 object
        let (slp_typa, slp_typb) = match parse_s5_object(dsdt_address) {
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
fn parse_s5_object(dsdt_phys_addr: u64) -> Option<(u16, u16)> {
    let dsdt_virt_addr = vma::phys_to_virt(dsdt_phys_addr);

    unsafe {
        let dsdt_ptr = dsdt_virt_addr.as_ptr::<u8>();

        // DSDT header
        let signature = core::slice::from_raw_parts(dsdt_ptr, 4);
        if signature != b"DSDT" {
            log_error!("Invalid DSDT signature");
            return None;
        }

        // Get DSDT length
        let length = core::ptr::read_unaligned(dsdt_ptr.add(4) as *const u32);
        log_debug!("DSDT length: {} bytes", length);

        // Search for the "_S5_" string in DSDT
        let dsdt_data = core::slice::from_raw_parts(dsdt_ptr, length as usize);

        // Byte representation of "_S5_" in AML
        let s5_name = b"_S5_";

        for i in 0..(dsdt_data.len() - 4) {
            if &dsdt_data[i..i+4] == s5_name {
                log_debug!("Found _S5 at offset {:#x}", i);

                // Parse the _S5 package
                // Typical AML bytecode:
                // Name(_S5, Package() {...})
                // Or: 08 5F 53 35 5F 12 [pkg_length] [num_elements] ...

                let mut offset = i + 4;

                // Skip possible NameOp (0x08)
                if offset < dsdt_data.len() && dsdt_data[offset] == 0x08 {
                    offset += 1;
                }

                // Find PackageOp (0x12)
                while offset < dsdt_data.len() && dsdt_data[offset] != 0x12 {
                    offset += 1;
                    if offset - i > 16 {
                        break;
                    }
                }

                if offset >= dsdt_data.len() {
                    log_warn!("PackageOp not found after _S5");
                    continue;
                }

                offset += 1; // 跳過 PackageOp

                // Parse PkgLength
                let pkg_length = parse_pkg_length(&dsdt_data[offset..]);
                offset += get_pkg_length_size(&dsdt_data[offset..]);

                // NumElements
                let num_elements = dsdt_data[offset];
                offset += 1;

                log_debug!("Package length: {}, elements: {}", pkg_length, num_elements);

                if num_elements < 2 {
                    log_warn!("_S5 package has less than 2 elements");
                    continue;
                }

                // Extract SLP_TYPa
                let slp_typa = parse_aml_integer(&dsdt_data[offset..]).unwrap_or(0);
                offset += get_aml_integer_size(&dsdt_data[offset..]);

                // Extract SLP_TYPb
                let slp_typb = parse_aml_integer(&dsdt_data[offset..]).unwrap_or(0);

                log_info!("Parsed _S5: SLP_TYPa={:#x}, SLP_TYPb={:#x}", slp_typa, slp_typb);

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
            if data.len() < 2 { return 0; }
            ((lead_byte & 0x0F) as usize) | ((data[1] as usize) << 4)
        }
        2 => {
            if data.len() < 3 { return 0; }
            ((lead_byte & 0x0F) as usize)
                | ((data[1] as usize) << 4)
                | ((data[2] as usize) << 12)
        }
        3 => {
            if data.len() < 4 { return 0; }
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

/// 解析 AML 整數
fn parse_aml_integer(data: &[u8]) -> Option<u64> {
    if data.is_empty() {
        return None;
    }

    match data[0] {
        0x00 => Some(0), // ZeroOp
        0x01 => Some(1), // OneOp
        0x0A => {        // BytePrefix
            if data.len() < 2 { return None; }
            Some(data[1] as u64)
        }
        0x0B => {        // WordPrefix
            if data.len() < 3 { return None; }
            Some(u16::from_le_bytes([data[1], data[2]]) as u64)
        }
        0x0C => {        // DWordPrefix
            if data.len() < 5 { return None; }
            Some(u32::from_le_bytes([data[1], data[2], data[3], data[4]]) as u64)
        }
        0x0E => {        // QWordPrefix
            if data.len() < 9 { return None; }
            Some(u64::from_le_bytes([
                data[1], data[2], data[3], data[4],
                data[5], data[6], data[7], data[8],
            ]))
        }
        _ => None,
    }
}

/// Get the byte size of the AML integer
fn get_aml_integer_size(data: &[u8]) -> usize {
    if data.is_empty() {
        return 0;
    }

    match data[0] {
        0x00 | 0x01 => 1,
        0x0A => 2,
        0x0B => 3,
        0x0C => 5,
        0x0E => 9,
        _ => 1,
    }
}

/// Perform ACPI shutdown
pub fn acpi_shutdown() -> ! {
    log_info!("Attempting ACPI shutdown...");

    let power_info = unsafe {
        match &ACPI_POWER_INFO {
            Some(info) => info,
            None => {
                log_error!("ACPI power info not initialized!");
                return fallback_shutdown();
            }
        }
    };

    unsafe {
        let slp_cmd_a = (power_info.slp_typa << 10) | power_info.slp_en;

        log_info!("Writing {:#x} to PM1a_CNT ({:#x})",
            slp_cmd_a, power_info.pm1a_control_block);

        // Write to PM1a control register
        io::io_port_ww(power_info.pm1a_control_block as u16, slp_cmd_a);

        // If PM1b exists, also write
        if power_info.pm1b_control_block != 0 {
            let slp_cmd_b = (power_info.slp_typb << 10) | power_info.slp_en;
            log_info!("Writing {:#x} to PM1b_CNT ({:#x})",
                slp_cmd_b, power_info.pm1b_control_block);
            io::io_port_ww(power_info.pm1b_control_block as u16, slp_cmd_b);
        }

        // 等待關機
        for _ in 0..1000000 {
            cpu::cpu_pause(100);
        }
    }

    log_error!("ACPI shutdown failed!");
    fallback_shutdown()
}

/// Backup shutdown method
fn fallback_shutdown() -> ! {
    log_warn!("Using fallback shutdown methods...");

    unsafe {
        // QEMU
        io::io_port_ww(0x604, 0x2000);
        cpu::cpu_pause(10000);

        // Bochs
        for &c in b"Shutdown" {
            io::io_port_wb(0x8900, c);
        }
        cpu::cpu_pause(10000);

        // VirtualBox
        io::io_port_ww(0x4004, 0x3400);
    }

    log_error!("All shutdown methods failed!");

    loop {
        cpu::cpu_halt();
    }
}

/// Store ACPI shutdown information
pub fn store_power_info(info: AcpiPowerInfo) {
    unsafe {
        ACPI_POWER_INFO = Some(info);
    }
    log_info!("ACPI power info stored successfully");
}