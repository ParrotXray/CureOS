// src/kernel/asm/x86/gdt.rs
use core::mem;
use core::ptr;
use x86::segmentation::{self, Descriptor, DataSegmentType, CodeSegmentType, SegmentSelector};
use x86::segmentation::{SegmentDescriptorBuilder, BuildDescriptor};
use x86::dtables::{DescriptorTablePointer, lgdt};
use x86::Ring;

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

pub const GDT_ENTRY_COUNT: usize = 5;

#[no_mangle]
pub static mut _GDT: [Descriptor; GDT_ENTRY_COUNT] = [Descriptor::NULL; GDT_ENTRY_COUNT];

#[no_mangle]
pub static mut _GDT_LIMIT: u16 = (mem::size_of::<[Descriptor; GDT_ENTRY_COUNT]>() - 1) as u16;

#[no_mangle]
pub extern "C" fn _init_gdt() {
    unsafe {
        _GDT[0] = Descriptor::NULL;
        
        _GDT[1] = segmentation::DescriptorBuilder::code_descriptor(
                0,
                0xFFFFF,
                CodeSegmentType::ExecuteRead
            )
            .present()
            .dpl(Ring::Ring0)
            .db()
            .limit_granularity_4kb()
            .finish();

        _GDT[2] = segmentation::DescriptorBuilder::data_descriptor(
                0,
                0xFFFFF,
                DataSegmentType::ReadWrite
            )
            .present()
            .dpl(Ring::Ring0) 
            .db() 
            .limit_granularity_4kb() 
            .finish();
        
        _GDT[3] = segmentation::DescriptorBuilder::code_descriptor(
                0,
                0xFFFFF,
                CodeSegmentType::ExecuteRead
            )
            .present() 
            .dpl(Ring::Ring3)
            .db()
            .limit_granularity_4kb()
            .finish();
        
        _GDT[4] = segmentation::DescriptorBuilder::data_descriptor(
                0,
                0xFFFFF,
                DataSegmentType::ReadWrite
            )
            .present()
            .dpl(Ring::Ring3)
            .db()
            .limit_granularity_4kb()
            .finish();
    }
}

#[no_mangle]
pub extern "C" fn _load_gdt() {
    unsafe {
        _init_gdt();

        let gdtr = DescriptorTablePointer {
            limit: _GDT_LIMIT,
            base: ptr::addr_of!(_GDT),
        };
        lgdt(&gdtr);
    }
}