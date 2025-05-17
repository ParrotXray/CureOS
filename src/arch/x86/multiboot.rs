// Multiboot 魔術數值和校驗和
pub const MULTIBOOT_MAGIC: u32 = 0x1BADB002;

// 校驗和計算函數
#[inline]
pub const fn checksum(flags: u32) -> u32 {
    -(MULTIBOOT_MAGIC as i32 + flags as i32) as u32
}

// 啟動加載器魔術數值（應該在 %eax 中）
pub const MULTIBOOT_BOOTLOADER_MAGIC: u32 = 0x2BADB002;

// Multiboot 模塊對齊
pub const MULTIBOOT_MOD_ALIGN: u32 = 0x00001000;

// Multiboot 資訊結構對齊
pub const MULTIBOOT_INFO_ALIGN: u32 = 0x00000004;

// 在 multiboot header 的 'flags' 成員中設置的標誌

// 在 i386 頁面（4KB）邊界上對齊所有啟動模塊
pub const MULTIBOOT_PAGE_ALIGN: u32 = 0x00000001;

// 必須將內存信息傳遞給操作系統
pub const MULTIBOOT_MEMORY_INFO: u32 = 0x00000002;

// 必須將視頻信息傳遞給操作系統
pub const MULTIBOOT_VIDEO_MODE: u32 = 0x00000004;

// 此標誌表示使用 header 中的地址字段
pub const MULTIBOOT_AOUT_KLUDGE: u32 = 0x00010000;

// 在 multiboot info 結構的 'flags' 成員中設置的標誌

// 是否有基本的低/高內存信息？
pub const MULTIBOOT_INFO_MEMORY: u32 = 0x00000001;
// 是否設置了啟動設備？
pub const MULTIBOOT_INFO_BOOTDEV: u32 = 0x00000002;
// 是否定義了命令行？
pub const MULTIBOOT_INFO_CMDLINE: u32 = 0x00000004;
// 是否有模塊需要處理？
pub const MULTIBOOT_INFO_MODS: u32 = 0x00000008;

// 以下兩個是互斥的

// 是否加載了符號表？
pub const MULTIBOOT_INFO_AOUT_SYMS: u32 = 0x00000010;
// 是否有 ELF 節頭表？
pub const MULTIBOOT_INFO_ELF_SHDR: u32 = 0x00000020;

// 是否有完整的內存映射？
pub const MULTIBOOT_INFO_MEM_MAP: u32 = 0x00000040;

// 是否有驅動器信息？
pub const MULTIBOOT_INFO_DRIVE_INFO: u32 = 0x00000080;

// 是否有配置表？
pub const MULTIBOOT_INFO_CONFIG_TABLE: u32 = 0x00000100;

// 是否有啟動加載器名稱？
pub const MULTIBOOT_INFO_BOOT_LOADER_NAME: u32 = 0x00000200;

// 是否有 APM 表？
pub const MULTIBOOT_INFO_APM_TABLE: u32 = 0x00000400;

// 是否有視頻信息？
pub const MULTIBOOT_INFO_VBE_INFO: u32 = 0x00000800;
pub const MULTIBOOT_INFO_FRAMEBUFFER_INFO: u32 = 0x00001000;

// 檢查特定標誌是否存在
#[inline]
pub const fn present(flags: u32, info: u32) -> bool {
    (flags & info) != 0
}

// 幀緩衝區類型常量
pub const MULTIBOOT_FRAMEBUFFER_TYPE_INDEXED: u8 = 0;
pub const MULTIBOOT_FRAMEBUFFER_TYPE_RGB: u8 = 1;
pub const MULTIBOOT_FRAMEBUFFER_TYPE_EGA_TEXT: u8 = 2;

// 內存映射類型常量
pub const MULTIBOOT_MEMORY_AVAILABLE: u32 = 1;
pub const MULTIBOOT_MEMORY_RESERVED: u32 = 2;
pub const MULTIBOOT_MEMORY_ACPI_RECLAIMABLE: u32 = 3;
pub const MULTIBOOT_MEMORY_NVS: u32 = 4;
pub const MULTIBOOT_MEMORY_BADRAM: u32 = 5;

// Multiboot 結構體定義

// ELF 符號表結構
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MultibootSymbolTable {
    pub tabsize: u32,
    pub strsize: u32,
    pub addr: u32,
    pub reserved: u32,
}

// ELF 節頭表結構
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MultibootElfSectionHeaderTable {
    pub num: u32,
    pub size: u32,
    pub addr: u32,
    pub shndx: u32,
}

// 符號表聯合體
#[repr(C)]
#[derive(Clone, Copy)]
pub union MultibootSymbolTableUnion {
    pub aout_sym: MultibootSymbolTable,
    pub elf_sec: MultibootElfSectionHeaderTable,
}

// 幀緩衝區調色板結構
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MultibootFramebufferPalette {
    pub framebuffer_palette_addr: u32,
    pub framebuffer_palette_num_colors: u16,
}

// 幀緩衝區 RGB 結構
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MultibootFramebufferRgb {
    pub framebuffer_red_field_position: u8,
    pub framebuffer_red_mask_size: u8,
    pub framebuffer_green_field_position: u8,
    pub framebuffer_green_mask_size: u8,
    pub framebuffer_blue_field_position: u8,
    pub framebuffer_blue_mask_size: u8,
}

// 幀緩衝區聯合體
#[repr(C)]
#[derive(Clone, Copy)]
pub union MultibootFramebufferUnion {
    pub palette: MultibootFramebufferPalette,
    pub rgb: MultibootFramebufferRgb,
}

// Multiboot 信息結構
#[repr(C)]
pub struct MultibootInfo {
    // Multiboot 信息版本號
    pub flags: u32,

    // 來自 BIOS 的可用內存
    pub mem_lower: u32,
    pub mem_upper: u32,

    // "根" 分區
    pub boot_device: u32,

    // 內核命令行
    pub cmdline: u32,

    // 引導模塊列表
    pub mods_count: u32,
    pub mods_addr: u32,

    // 符號表聯合體
    pub u: MultibootSymbolTableUnion,

    // 內存映射緩衝區
    pub mmap_length: u32,
    pub mmap_addr: u32,

    // 驅動器信息緩衝區
    pub drives_length: u32,
    pub drives_addr: u32,

    // ROM 配置表
    pub config_table: u32,

    // 引導加載器名稱
    pub boot_loader_name: u32,

    // APM 表
    pub apm_table: u32,

    // 視頻
    pub vbe_control_info: u32,
    pub vbe_mode_info: u32,
    pub vbe_mode: u16,
    pub vbe_interface_seg: u16,
    pub vbe_interface_off: u16,
    pub vbe_interface_len: u16,

    // 幀緩衝區信息
    pub framebuffer_addr: u64,
    pub framebuffer_pitch: u32,
    pub framebuffer_width: u32,
    pub framebuffer_height: u32,
    pub framebuffer_bpp: u8,
    pub framebuffer_type: u8,
    
    pub fb_union: MultibootFramebufferUnion,
}

// 顏色結構
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MultibootColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

// 內存映射條目
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MultibootMemoryMapEntry {
    pub size: u32,
    pub addr_low: u32,
    pub addr_high: u32,
    pub len_low: u32,
    pub len_high: u32,
    pub type_: u32,
}

// 模塊列表
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MultibootModuleList {
    // 使用的內存從 'mod_start' 到 'mod_end-1' 位元組（包含）
    pub mod_start: u32,
    pub mod_end: u32,

    // 模塊命令行
    pub cmdline: u32,

    // 填充到 16 字節（必須為零）
    pub pad: u32,
}

// APM BIOS 信息
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MultibootApmInfo {
    pub version: u16,
    pub cseg: u16,
    pub offset: u32,
    pub cseg_16: u16,
    pub dseg: u16,
    pub flags: u16,
    pub cseg_len: u16,
    pub cseg_16_len: u16,
    pub dseg_len: u16,
}