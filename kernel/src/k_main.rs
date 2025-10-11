use crate::hal::cpu;
use crate::kprintln;

pub fn kernel_main() -> ! {
    kprintln!("Welcome to CureOS!");

    let mut brand_buf = [0u8; 64];
    let mut model_buf = [0u8; 16];
    kprintln!("CPU: {} ({})",
        cpu::cpu_get_brand(&mut brand_buf),
        cpu::cpu_get_model(&mut model_buf)
    );

    kprintln!();
    kprintln!("Control Registers:");
    kprintln!("  CR0: 0x{:016x}", cpu::cpu_r_cr0().bits());
    kprintln!("  CR2: 0x{:016x}", cpu::cpu_r_cr2());
    kprintln!("  CR3: 0x{:016x}", cpu::cpu_r_cr3());
    kprintln!("  CR4: 0x{:016x}", cpu::cpu_r_cr4().bits());

    kprintln!();
    kprintln!("Kernel initialized successfully!");

    loop {
        cpu::cpu_halt();
    }
}