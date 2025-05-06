// src/kernel/asm/x86/gdt.rs

use core::mem;

#[allow(dead_code)]
pub const fn sd_type(x: u64) -> u64             { x << 8 }
#[allow(dead_code)]
pub const fn sd_code_data(x: u64) -> u64        { x << 12 }
#[allow(dead_code)]
pub const fn sd_dpl(x: u64) -> u64              { x << 13 }
#[allow(dead_code)]
pub const fn sd_present(x: u64) -> u64          { x << 15 }
#[allow(dead_code)]
pub const fn sd_avl(x: u64) -> u64              { x << 20 }
#[allow(dead_code)]
pub const fn sd_64bits(x: u64) -> u64           { x << 21 }
#[allow(dead_code)]
pub const fn sd_32bits(x: u64) -> u64           { x << 22 }
#[allow(dead_code)]
pub const fn sd_4k_gran(x: u64) -> u64          { x << 23 }

#[allow(dead_code)]
pub const fn seg_lim_l(x: u64) -> u64           { x & 0x0ffff }
#[allow(dead_code)]
pub const fn seg_lim_h(x: u64) -> u64           { x & 0xf0000 }
#[allow(dead_code)]
pub const fn seg_base_l(x: u64) -> u64          { (x & 0x0000ffff) << 16 }
#[allow(dead_code)]
pub const fn seg_base_m(x: u64) -> u64          { (x & 0x00ff0000) >> 16 }
#[allow(dead_code)]
pub const fn seg_base_h(x: u64) -> u64          { x & 0xff000000 }

#[allow(dead_code)]
pub const SEG_DATA_RD: u64 = 0x00; // Read-Only
#[allow(dead_code)]
pub const SEG_DATA_RDA: u64 = 0x01; // Read-Only, accessed
#[allow(dead_code)]
pub const SEG_DATA_RDWR: u64 = 0x02; // Read/Write
#[allow(dead_code)]
pub const SEG_DATA_RDWRA: u64 = 0x03; // Read/Write, accessed
#[allow(dead_code)]
pub const SEG_DATA_RDEXPD: u64 = 0x04; // Read-Only, expand-down
#[allow(dead_code)]
pub const SEG_DATA_RDEXPDA: u64 = 0x05; // Read-Only, expand-down, accessed
#[allow(dead_code)]
pub const SEG_DATA_RDWREXPD: u64 = 0x06; // Read/Write, expand-down
#[allow(dead_code)]
pub const SEG_DATA_RDWREXPDA: u64 = 0x07; // Read/Write, expand-down, accessed
#[allow(dead_code)]
pub const SEG_CODE_EX: u64 = 0x08; // Execute-Only
#[allow(dead_code)]
pub const SEG_CODE_EXA: u64 = 0x09; // Execute-Only, accessed
#[allow(dead_code)]
pub const SEG_CODE_EXRD: u64 = 0x0A; // Execute/Read
#[allow(dead_code)]
pub const SEG_CODE_EXRDA: u64 = 0x0B; // Execute/Read, accessed
#[allow(dead_code)]
pub const SEG_CODE_EXC: u64 = 0x0C; // Execute-Only, conforming
#[allow(dead_code)]
pub const SEG_CODE_EXCA: u64 = 0x0D; // Execute-Only, conforming, accessed
#[allow(dead_code)]
pub const SEG_CODE_EXRDC: u64 = 0x0E; // Execute/Read, conforming
#[allow(dead_code)]
pub const SEG_CODE_EXRDCA: u64 = 0x0F; // Execute/Read, conforming, accessed

#[allow(dead_code)]
pub const SEG_R0_CODE: u64 = 
    sd_type(SEG_CODE_EXRD) | sd_code_data(1) | sd_dpl(0) |
    sd_present(1) | sd_avl(0) | sd_64bits(0) | sd_32bits(1) |
    sd_4k_gran(1);

#[allow(dead_code)]
pub const SEG_R0_DATA: u64 = 
    sd_type(SEG_DATA_RDWR) | sd_code_data(1) | sd_dpl(0) |
    sd_present(1) | sd_avl(0) | sd_64bits(0) | sd_32bits(1) |
    sd_4k_gran(1);

#[allow(dead_code)]
pub const SEG_R3_CODE: u64 = 
    sd_type(SEG_CODE_EXRD) | sd_code_data(1) | sd_dpl(3) |
    sd_present(1) | sd_avl(0) | sd_64bits(0) | sd_32bits(1) |
    sd_4k_gran(1);

#[allow(dead_code)]
pub const SEG_R3_DATA: u64 = 
    sd_type(SEG_DATA_RDWR) | sd_code_data(1) | sd_dpl(3) |
    sd_present(1) | sd_avl(0) | sd_64bits(0) | sd_32bits(1) |
    sd_4k_gran(1);

pub const GDT_ENTRY: usize = 5;

#[no_mangle]
pub static mut _GDT: [u64; GDT_ENTRY] = [0; GDT_ENTRY];

#[no_mangle]
pub static mut _GDT_LIMIT: u16 = (mem::size_of::<[u64; GDT_ENTRY]>() - 1) as u16;

#[no_mangle]
pub fn _set_gdt_entry(index: usize, base: u64, limit: u64, flags: u64) {
    unsafe {
        _GDT[index] = seg_base_h(base) | flags | seg_lim_h(limit) | seg_base_m(base);
        _GDT[index] <<= 32;
        _GDT[index] |= seg_base_l(base) | seg_lim_l(limit);
    }
    
}

#[no_mangle]
pub fn _init_gdt() {
    _set_gdt_entry(0, 0, 0, 0);
    _set_gdt_entry(1, 0, 0xfffff, SEG_R0_CODE);
    _set_gdt_entry(2, 0, 0xfffff, SEG_R0_DATA);
    _set_gdt_entry(3, 0, 0xfffff, SEG_R3_CODE);
    _set_gdt_entry(4, 0, 0xfffff, SEG_R3_DATA);
}