// kernel/src/kernel/asm/amd64/gdt.rs
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::instructions::segmentation::{Segment, CS, DS, ES, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::{VirtAddr, PrivilegeLevel};
use lazy_static::lazy_static;
use crate::kprintln;
use crate::{log_trace, log_debug, log_info, log_warn, log_error, log_fatal};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        unsafe { tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;  // 20KB
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::new(&raw const STACK as *const _ as u64);
            let stack_end = stack_start + STACK_SIZE as u64;

            stack_end
        }; }

        unsafe { tss.privilege_stack_table[0] = {
            const STACK_SIZE: usize = 4096 * 5;  // 20KB
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::new(&raw const STACK as *const _ as u64);
            let stack_end = stack_start + STACK_SIZE as u64;

            stack_end
        }; }

        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        // 0x00: Null

        // 0x08 ring 0
        let kernel_code_selector = gdt.append(Descriptor::kernel_code_segment());

        // 0x10 ring 0
        let kernel_data_selector = gdt.append(Descriptor::kernel_data_segment());

        // 0x18 ring 3
        let user_code_selector_index = gdt.append(Descriptor::user_code_segment());
        // User code selector needs RPL=3, so we create a new selector with the correct privilege level
        let user_code_selector = SegmentSelector::new(user_code_selector_index.index(), PrivilegeLevel::Ring3);

        // 0x20 ring 3
        let user_data_selector_index = gdt.append(Descriptor::user_data_segment());
        // User data selector needs RPL=3, so we create a new selector with the correct privilege level
        let user_data_selector = SegmentSelector::new(user_data_selector_index.index(), PrivilegeLevel::Ring3);

        // 0x28 Task seg
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));

        (
            gdt,
            Selectors {
                kernel_code_selector,
                kernel_data_selector,
                user_code_selector,
                user_data_selector,
                tss_selector,
            },
        )
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Selectors {
    pub kernel_code_selector: SegmentSelector,
    pub kernel_data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

pub fn get_selectors() -> Selectors {
    GDT.1
}

pub fn user_code_selector() -> SegmentSelector {
    GDT.1.user_code_selector
}

pub fn user_data_selector() -> SegmentSelector {
    GDT.1.user_data_selector
}

pub fn kernel_code_selector() -> SegmentSelector {
    GDT.1.kernel_code_selector
}

pub fn kernel_data_selector() -> SegmentSelector {
    GDT.1.kernel_data_selector
}

// 初始化 GDT
pub fn init() {
    GDT.0.load();
    let gdt_addr = &GDT.0 as *const _ as u64;
    log_debug!("GDT address: {:#018x}", gdt_addr);

    unsafe {
        CS::set_reg(GDT.1.kernel_code_selector);

        DS::set_reg(GDT.1.kernel_data_selector);
        ES::set_reg(GDT.1.kernel_data_selector);
        SS::set_reg(GDT.1.kernel_data_selector);

        // 加載 TSS
        load_tss(GDT.1.tss_selector);
    }
}

pub fn print_info() {
    let selectors = get_selectors();

    log_info!("Kernel Code: {:#x} (ring 0)", selectors.kernel_code_selector.0);
    log_info!("Kernel Data: {:#x} (ring 0)", selectors.kernel_data_selector.0);
    log_info!("User Code:   {:#x} (ring 3)", selectors.user_code_selector.0);
    log_info!("User Data:   {:#x} (ring 3)", selectors.user_data_selector.0);
    log_info!("TSS:         {:#x}", selectors.tss_selector.0);
    
    // Verify that user segments have correct RPL bits
    log_debug!("User Code RPL: {:?}", selectors.user_code_selector.rpl());
    log_debug!("User Data RPL: {:?}", selectors.user_data_selector.rpl());
}