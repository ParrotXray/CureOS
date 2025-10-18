// kernel/src/hal/power.rs

use crate::hal::{io, cpu, rtc};
use crate::{log_info, log_debug, log_warn, log_error};
use super::acpi;

#[derive(Debug, Clone, Copy)]
pub enum PowerState {
    S0,  // Working
    S1,  // Sleep
    S3,  // Suspend to RAM
    S4,  // Suspend to Disk
    S5,  // Soft Off
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShutdownMethod {
    Acpi,
    QemuExit,
    BochsExit,
    VirtualBox,
    Apm,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RebootMethod {
    BootACPI,
    BootEFI,
    BootKBD,
    BootCF9,
    Boot92h,
}

pub fn shutdown() -> ! {
    log_info!("Initiating system shutdown...");

    log_info!("Disabling CPU interrupts");
    cpu::cpu_disable_interrupts();

    log_info!("Shutting down the RTC Timer");
    rtc::disable_timer();

    let methods = [
        ShutdownMethod::Acpi,
        ShutdownMethod::QemuExit,
        ShutdownMethod::BochsExit,
        ShutdownMethod::VirtualBox,
        ShutdownMethod::Apm,
    ];

    for method in methods.iter() {
        log_debug!("Trying shutdown method: {:?}", method);
        try_shutdown(*method);

        log_warn!("{:?} shutdown failed", method);
        cpu::cpu_pause(1000);
    }

    log_error!("All shutdown methods failed!");
    log_error!("System halted. Please power off manually.");

    loop {
        cpu::cpu_halt();
    }
}

fn try_shutdown(method: ShutdownMethod) {
    unsafe {
        match method {
            ShutdownMethod::Acpi => {
                acpi::power::acpi_shutdown();
            }

            ShutdownMethod::QemuExit => {
                // QEMU isa-debug-exit device
                io::io_port_ww(0x604, 0x2000);
                io::io_port_rl(0x501);
            }

            ShutdownMethod::BochsExit => {
                // Bochs dedicated shutdown port
                io::io_port_ww(0xB004, 0x2000);
            }

            ShutdownMethod::VirtualBox => {
                // VirtualBox shutdown port
                io::io_port_ww(0x4004, 0x3400);
            }

            ShutdownMethod::Apm => {
                // APM (Advanced Power Management) BIOS
                // APM version 1.0
                io::io_port_wb(0x8900, 0x53);
                io::io_port_wb(0x8900, 0x00);
                io::io_port_wb(0x8900, 0x01);
                io::io_port_wb(0x8900, 0x53);
            }
        }

        cpu::cpu_pause(10000);
    }
}

/// Restart the system
pub fn reboot() -> ! {
    log_info!("Rebooting system...");

    log_info!("Disabling CPU interrupts");
    cpu::cpu_disable_interrupts();

    log_info!("Shutting down the RTC Timer");
    rtc::disable_timer();

    let methods = [
        RebootMethod::BootACPI,
        RebootMethod::BootEFI,
        RebootMethod::BootKBD,
        RebootMethod::BootCF9,
        RebootMethod::Boot92h,
    ];

    for method in methods.iter() {
        log_debug!("Trying reboot method: {:?}", method);
        try_reboot(*method);

        log_warn!("{:?} reboot failed", method);
        cpu::cpu_pause(1000);
    }

    log_error!("All reboot methods failed!");
    log_error!("System halted. Please power off manually.");

    loop {
        cpu::cpu_halt();
    }
}

fn try_reboot(method: RebootMethod) {
    match method {
        RebootMethod::BootACPI => {
            acpi::power::acpi_reset_reg_reboot();
        }

        RebootMethod::BootKBD => {
            keyboard_controller_reboot();
        }

        RebootMethod::BootCF9 => {
            pci_reboot();
        }
        RebootMethod::BootEFI => {
            efi_reboot();
        }

        RebootMethod::Boot92h => {
            cpu_reset();
        }
    }

    cpu::cpu_pause(10000);
}

fn keyboard_controller_reboot() -> bool {
    log_debug!("keyboard controller reboot...");

    unsafe {
        for _ in 0..1000 {
            if (io::io_port_rb(0x64) & 0x02) == 0 {
                break;
            }
            cpu::cpu_pause(10);
        }

        io::io_port_wb(0x64, 0xFE);

        cpu::cpu_pause(100000);
    }

    false
}

fn pci_reboot() -> bool {
    log_debug!("PCI reboot...");

    unsafe {
        let mut val = io::io_port_rb(0xCF9) & !0x06;
        io::io_port_wb(0xCF9, val | 0x02);
        cpu::cpu_pause(1000);
        io::io_port_wb(0xCF9, val | 0x06);

        cpu::cpu_pause(100000);
    }

    false
}


fn efi_reboot() -> bool {
    log_debug!("Trying EFI runtime services reboot...");

    // TODO: 實現 EFI ResetSystem 調用
    false
}

fn cpu_reset() -> bool {
    log_debug!("CPU reset via port 92h...");

    unsafe {
        let mut val = io::io_port_rb(0x92);
        val &= !0x01; // Clear Fast A20 Bit
        val |= 0x01;  // Set the reset bit
        io::io_port_wb(0x92, val);

        cpu::cpu_pause(100000);
    }

    false
}