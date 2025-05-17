use crate::kernel::common;

// 常量定義
#[allow(dead_code)]
pub const PG_MAX_ENTRIES: u32 = 1024;
#[allow(dead_code)]
pub const PG_LAST_TABLE: u32 = PG_MAX_ENTRIES - 1;
#[allow(dead_code)]
pub const PG_FIRST_TABLE: u32 = 0;

// 内存地址转换宏
#[allow(dead_code)]
#[inline]
pub fn p2v(paddr: usize) -> usize {
    paddr + common::HIGHER_HLF_BASE as usize
}

#[allow(dead_code)]
#[inline]
pub fn v2p(vaddr: usize) -> usize {
    vaddr - common::HIGHER_HLF_BASE as usize
}

// 页面对齐函数
#[allow(dead_code)]
#[inline]
pub fn pg_align(addr: usize) -> usize {
    addr & 0xFFFFF000
}

// 页表索引函数
#[allow(dead_code)]
#[inline]
pub fn pd_index(vaddr: usize) -> usize {
    (vaddr & 0xFFC00000) >> 22
}

#[allow(dead_code)]
#[inline]
pub fn pt_index(vaddr: usize) -> usize {
    (vaddr & 0x003FF000) >> 12
}

#[allow(dead_code)]
#[inline]
pub fn pg_offset(vaddr: usize) -> usize {
    vaddr & 0x00000FFF
}

// 获取页表和页面地址
#[allow(dead_code)]
#[inline]
pub fn get_pt_addr(pde: u32) -> u32 {
    pg_align(pde as usize) as u32
}

#[allow(dead_code)]
#[inline]
pub fn get_pg_addr(pte: u32) -> u32 {
    pg_align(pte as usize) as u32
}

// 页面状态查询函数
#[allow(dead_code)]
#[inline]
pub fn pg_dirty(pte: u32) -> u32 {
    (pte & (1 << 6)) >> 6
}

#[allow(dead_code)]
#[inline]
pub fn pg_accessed(pte: u32) -> u32 {
    (pte & (1 << 5)) >> 5
}

#[allow(dead_code)]
#[inline]
pub fn is_cached(entry: u32) -> bool {
    (entry & 0x1) != 0
}

// 页表标志位常量
#[allow(dead_code)]
pub const PG_PRESENT: u32 = 0x1;
#[allow(dead_code)]
pub const PG_WRITE: u32 = 0x1 << 1;
#[allow(dead_code)]
pub const PG_ALLOW_USER: u32 = 0x1 << 2;
#[allow(dead_code)]
pub const PG_WRITE_THROUGHT: u32 = 1 << 3;
#[allow(dead_code)]
pub const PG_DISABLE_CACHE: u32 = 1 << 4;
#[allow(dead_code)]
pub const PG_PDE_4MB: u32 = 1 << 7;

// 页目录和页表项创建函数
#[allow(dead_code)]
#[inline]
pub fn pde(flags: u32, pt_addr: u32) -> u32 {
    pg_align(pt_addr as usize) as u32 | (flags & 0xfff)
}

#[allow(dead_code)]
#[inline]
pub fn pte(flags: u32, pg_addr: u32) -> u32 {
    pg_align(pg_addr as usize) as u32 | (flags & 0xfff)
}

// 虚拟地址和物理地址构建函数
#[allow(dead_code)]
#[inline]
pub fn v_addr(pd: u32, pt: u32, offset: u32) -> u32 {
    (pd << 22) | (pt << 12) | offset
}

#[allow(dead_code)]
#[inline]
pub fn p_addr(ppn: u32, offset: u32) -> u32 {
    (ppn << 12) | offset
}

// 页面权限常量
#[allow(dead_code)]
pub const PG_PREM_R: u32 = PG_PRESENT;
#[allow(dead_code)]
pub const PG_PREM_RW: u32 = PG_PRESENT | PG_WRITE;
#[allow(dead_code)]
pub const PG_PREM_UR: u32 = PG_PRESENT | PG_ALLOW_USER;
#[allow(dead_code)]
pub const PG_PREM_URW: u32 = PG_PRESENT | PG_WRITE | PG_ALLOW_USER;
#[allow(dead_code)]
pub const T_SELF_REF_PERM: u32 = PG_PREM_RW | PG_DISABLE_CACHE;

// 页目录和页表的虚拟基地址常量
#[allow(dead_code)]
pub const PTD_BASE_VADDR: u32 = 0xFFFFF000;
#[allow(dead_code)]
pub const PT_BASE_VADDR: u32 = 0xFFC00000;

// 获取特定页表的虚拟地址函数
#[allow(dead_code)]
#[inline]
pub fn pt_vaddr(pd_offset: u32) -> u32 {
    PT_BASE_VADDR | (pd_offset << 12)
}

// 类型定义
#[allow(dead_code)]
pub type PtdT = u32;
#[allow(dead_code)]
pub type PtT = u32;
#[allow(dead_code)]
pub type PtAttr = u32;