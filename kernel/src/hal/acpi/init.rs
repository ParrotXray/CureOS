use super::power::{extract_power_info, store_power_info};
use super::{AcpiInfo, CureAcpiHandler};
use crate::hal::{cpu, io};
use crate::kprintln;
use crate::{log_debug, log_error, log_fatal, log_info, log_trace, log_warn};
use acpi::platform::{interrupt::InterruptModel, AcpiPlatform, PciConfigRegions};
use acpi::rsdp::Rsdp;
use acpi::sdt::hpet::HpetInfo;
use acpi::{aml, sdt, AcpiTables, Handle, Handler, PciAddress, PhysicalMapping};
use core::{mem, ptr::NonNull};

pub fn init(rsdp_addr: u64, physical_memory_offset: u64) -> Option<AcpiInfo> {
    let handler = CureAcpiHandler::new(physical_memory_offset);

    let rsdp_mapping =
        unsafe { handler.map_physical_region::<Rsdp>(rsdp_addr as usize, size_of::<Rsdp>()) };
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

    // Check interrupt mode
    let (has_apic, local_apic_addr, io_apics_info) = match &platform.interrupt_model {
        InterruptModel::Apic(apic) => {
            log_info!("Local APIC Address: {:#x}", apic.local_apic_address);
            log_info!("IO APICs: {} controller(s)", apic.io_apics.len());

            let mut io_apics = alloc::vec::Vec::new();

            for (i, io_apic) in apic.io_apics.iter().enumerate() {
                log_info!(
                    "IO APIC {}: ID={}, Address={:#x}, GSI Base={}",
                    i,
                    io_apic.id,
                    io_apic.address,
                    io_apic.global_system_interrupt_base
                );

                io_apics.push((
                    io_apic.address as u64,
                    io_apic.id,
                    io_apic.global_system_interrupt_base,
                ));
            }

            (true, Some(apic.local_apic_address as u64), io_apics)
        }
        InterruptModel::Unknown => {
            log_warn!("Interrupt Model: Unknown (not APIC)");
            (false, None, alloc::vec::Vec::new())
        }
        _ => {
            log_warn!("Interrupt Model: Other");
            (false, None, alloc::vec::Vec::new())
        }
    };

    let has_hpet = match HpetInfo::new(&platform.tables) {
        Ok(hpet) => {
            log_info!("Base Address: {:#x}", hpet.base_address);
            log_info!("Hardware Rev: {}", hpet.hardware_rev);
            log_info!("Comparator Count: {}", hpet.num_comparators);
            log_info!(
                "Counter Size: {} bit",
                if hpet.main_counter_is_64bits { 64 } else { 32 }
            );
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

    kprintln!();
    log_info!("Extracting power management information...");
    if let Some(power_info) = extract_power_info(&platform.tables, &handler) {
        store_power_info(power_info);
    } else {
        log_warn!("Could not extract ACPI power info");
    }

    log_info!("ACPI initialized successfully!");

    Some(AcpiInfo {
        revision,
        boot_processor,
        cpu_count,
        has_apic,
        has_hpet,
        local_apic_address: local_apic_addr,
        io_apics: io_apics_info,
    })
}

pub fn print_info(info: &AcpiInfo) {
    kprintln!();
    log_info!("Revision: ACPI {}.0", info.revision);
    log_info!("CPUs: {} processor(s)", info.cpu_count);
    if let Some(boot_proc) = info.boot_processor {
        log_info!("Boot Processor: UID {}", boot_proc);
    }
    log_info!(
        "APIC: {}",
        if info.has_apic {
            "Available "
        } else {
            "Not available"
        }
    );
    log_info!(
        "HPET: {}",
        if info.has_hpet {
            "Available "
        } else {
            "Not available"
        }
    );
}
