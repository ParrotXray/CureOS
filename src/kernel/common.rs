#[allow(dead_code)]
pub const K_STACK_SIZE: u32 = 0x100000;
#[allow(dead_code)]
pub const K_STACK_START: u32 = (0xFFBFFFFF - K_STACK_SIZE) + 1;
#[allow(dead_code)]
pub const HIGHER_HLF_BASE: u32 = 0xC0000000;
#[allow(dead_code)]
pub const MEM_1MB: u32 = 0x100000;

#[allow(dead_code)]
pub const VGA_BUFFER_VADDR: u32 = 0xB0000000;
#[allow(dead_code)]
pub const VGA_BUFFER_PADDR: u32 = 0xB8000;
#[allow(dead_code)]
pub const VGA_BUFFER_SIZE: u32 = 4096;