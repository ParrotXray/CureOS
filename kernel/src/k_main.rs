// kernel/src/k_main.rs (Updated with Logger Demo)
use crate::hal::cpu;
use crate::kprintln;
use crate::{log_trace, log_debug, log_info, log_warn, log_error, log_fatal};

pub fn _kernel_main() -> ! {
    kprintln!();
    kprintln!("=== Welcome to CureOS! ===");
    kprintln!();

    let mut brand_buf = [0u8; 64];
    let mut model_buf = [0u8; 16];
    log_info!("CPU: {} ({})",
        cpu::cpu_get_brand(&mut brand_buf),
        cpu::cpu_get_model(&mut model_buf)
    );

    kprintln!();
    log_debug!("CR0: 0x{:016x}", cpu::cpu_r_cr0().bits());
    log_debug!("CR2: 0x{:016x}", cpu::cpu_r_cr2());
    log_debug!("CR3: 0x{:016x}", cpu::cpu_r_cr3());
    log_debug!("CR4: 0x{:016x}", cpu::cpu_r_cr4().bits());

    // kprintln!();
    // kprintln!("=== Logger Level Demonstration ===");
    // kprintln!();
    //
    // log_trace!("TRACE: This is a trace message (lowest priority)");
    // log_debug!("DEBUG: Detailed debugging information");
    // log_info!("INFO: General information about system operation");
    // log_warn!("WARN: Warning message - something might be wrong");
    // log_error!("ERROR: Error occurred but system can continue");
    // log_fatal!("FATAL: Critical error (highest priority)");
    //
    // kprintln!();
    //
    // log_info!("Starting system services...");
    // log_debug!("Loading drivers...");
    // log_trace!(" Scanning PCI bus");
    // log_trace!("Initializing USB controller");
    // log_debug!("Drivers loaded successfully");
    //
    // log_info!("System is ready!");

    kprintln!();
    log_warn!("Entering idle loop");

    loop {
        cpu::cpu_halt();
    }
}