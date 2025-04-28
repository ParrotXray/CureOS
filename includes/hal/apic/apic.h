#ifndef _LUNAIX_APIC_H
#define _LUNAIX_APIC_H

#include <stdint.h>

// Local APIC寄存器偏移量 (基於APIC基地址)
#define APIC_ID                  0x20  // Local APIC ID寄存器
#define APIC_VERSION             0x30  // Local APIC版本寄存器
#define APIC_TPR                 0x80  // Task Priority寄存器
#define APIC_APR                 0x90  // Arbitration Priority寄存器
#define APIC_PPR                 0xA0  // Processor Priority寄存器
#define APIC_EOI                 0xB0  // EOI寄存器
#define APIC_RRD                 0xC0  // Remote Read寄存器
#define APIC_LDR                 0xD0  // Logical Destination寄存器
#define APIC_DFR                 0xE0  // Destination Format寄存器
#define APIC_SVR                 0xF0  // Spurious Interrupt Vector寄存器
#define APIC_ISR                 0x100 // In-Service寄存器
#define APIC_TMR                 0x180 // Trigger Mode寄存器
#define APIC_IRR                 0x200 // Interrupt Request寄存器
#define APIC_ESR                 0x280 // Error Status寄存器
#define APIC_ICR_LOW             0x300 // Interrupt Command寄存器低位
#define APIC_ICR_HIGH            0x310 // Interrupt Command寄存器高位
#define APIC_LVT_TIMER           0x320 // LVT Timer寄存器
#define APIC_LVT_THERMAL         0x330 // LVT Thermal Sensor寄存器
#define APIC_LVT_PERF            0x340 // LVT Performance Counter寄存器
#define APIC_LVT_LINT0           0x350 // LVT LINT0寄存器
#define APIC_LVT_LINT1           0x360 // LVT LINT1寄存器
#define APIC_LVT_ERROR           0x370 // LVT Error寄存器
#define APIC_TIMER_INITIAL_COUNT 0x380 // Timer Initial Count寄存器
#define APIC_TIMER_CURRENT_COUNT 0x390 // Timer Current Count寄存器
#define APIC_TIMER_DIVIDE_CONFIG 0x3E0 // Timer Divide Configuration寄存器

// 本地APIC標誌位
#define APIC_SVR_ENABLE          0x100  // APIC軟件使能位

// LVT傳遞模式
#define APIC_LVT_DELIVERY_FIXED    0x000
#define APIC_LVT_DELIVERY_SMI      0x200
#define APIC_LVT_DELIVERY_NMI      0x400
#define APIC_LVT_DELIVERY_EXTINT   0x700
#define APIC_LVT_DELIVERY_INIT     0x500
#define APIC_LVT_DELIVERY_STARTUP  0x600

// LVT掩碼位
#define APIC_LVT_MASKED           0x10000

// APIC定時器模式
#define APIC_TIMER_ONESHOT        0x00000
#define APIC_TIMER_PERIODIC       0x20000
#define APIC_TIMER_TSC_DEADLINE   0x40000

// ICR傳遞模式
#define APIC_ICR_DELIVERY_FIXED    0x000
#define APIC_ICR_DELIVERY_LOWEST   0x100
#define APIC_ICR_DELIVERY_SMI      0x200
#define APIC_ICR_DELIVERY_NMI      0x400
#define APIC_ICR_DELIVERY_INIT     0x500
#define APIC_ICR_DELIVERY_STARTUP  0x600

// ICR目標模式
#define APIC_ICR_PHYSICAL         0x00000
#define APIC_ICR_LOGICAL          0x00800

// ICR級別
#define APIC_ICR_LEVEL_ASSERT     0x04000
#define APIC_ICR_LEVEL_DEASSERT   0x00000

// ICR觸發模式
#define APIC_ICR_TRIGGER_EDGE     0x00000
#define APIC_ICR_TRIGGER_LEVEL    0x08000

// ICR狀態
#define APIC_ICR_STATUS_IDLE      0x00000
#define APIC_ICR_STATUS_PENDING   0x01000

// ICR目標簡寫
#define APIC_ICR_DESTINATION_SELF         0x40000
#define APIC_ICR_DESTINATION_ALL          0x80000
#define APIC_ICR_DESTINATION_ALL_BUT_SELF 0xC0000

// IO APIC寄存器
#define IOAPIC_IOREGSEL           0x00  // 寄存器選擇
#define IOAPIC_IOWIN              0x10  // 寄存器窗口

// IO APIC寄存器索引
#define IOAPIC_REG_ID             0x00  // ID寄存器
#define IOAPIC_REG_VER            0x01  // 版本寄存器
#define IOAPIC_REG_ARB            0x02  // 仲裁寄存器
#define IOAPIC_REG_REDTBL_BASE    0x10  // 重定向表基址

// IO APIC重定向表項標誌位
#define IOAPIC_REDTBL_MASKED      0x10000 // 掩碼位
#define IOAPIC_REDTBL_TRIGGER_LEVEL 0x8000 // 觸發模式為電平觸發
#define IOAPIC_REDTBL_TRIGGER_EDGE  0x0000 // 觸發模式為邊緣觸發
#define IOAPIC_REDTBL_INTPOL_LOW  0x2000 // 極性為低電平有效
#define IOAPIC_REDTBL_INTPOL_HIGH 0x0000 // 極性為高電平有效
#define IOAPIC_REDTBL_DESTMOD_LOGICAL 0x800 // 邏輯模式
#define IOAPIC_REDTBL_DESTMOD_PHYSICAL 0x000 // 物理模式
#define IOAPIC_REDTBL_DELMOD_FIXED 0x000 // 固定傳遞模式
#define IOAPIC_REDTBL_DELMOD_LOWEST 0x100 // 最低優先級傳遞模式
#define IOAPIC_REDTBL_DELMOD_SMI  0x200 // SMI傳遞模式
#define IOAPIC_REDTBL_DELMOD_NMI  0x400 // NMI傳遞模式
#define IOAPIC_REDTBL_DELMOD_INIT 0x500 // INIT傳遞模式
#define IOAPIC_REDTBL_DELMOD_EXTINT 0x700 // ExtINT傳遞模式

/**
 * @brief 初始化APIC子系統
 * 
 * @return int 成功返回1，失敗返回0
 */
int apic_init();

/**
 * @brief 讀取本地APIC寄存器
 * 
 * @param reg 寄存器偏移量
 * @return uint32_t 寄存器值
 */
uint32_t apic_read(uint32_t reg);

/**
 * @brief 寫入本地APIC寄存器
 * 
 * @param reg 寄存器偏移量
 * @param value 寄存器值
 */
void apic_write(uint32_t reg, uint32_t value);

/**
 * @brief 獲取本地APIC ID
 * 
 * @return uint32_t APIC ID
 */
uint32_t apic_get_id();

/**
 * @brief 發送EOI(End of Interrupt)信號
 */
void apic_send_eoi();

/**
 * @brief 配置本地APIC定時器
 * 
 * @param vector 中斷向量
 * @param mode 定時器模式
 * @param initial_count 初始計數值
 */
void apic_configure_timer(uint8_t vector, uint32_t mode, uint32_t initial_count);

/**
 * @brief 停止本地APIC定時器
 */
void apic_stop_timer();

/**
 * @brief 讀取IO APIC寄存器
 * 
 * @param io_apic_base IO APIC基地址
 * @param reg 寄存器索引
 * @return uint32_t 寄存器值
 */
uint32_t io_apic_read(uintptr_t io_apic_base, uint8_t reg);

/**
 * @brief 寫入IO APIC寄存器
 * 
 * @param io_apic_base IO APIC基地址
 * @param reg 寄存器索引
 * @param value 寄存器值
 */
void io_apic_write(uintptr_t io_apic_base, uint8_t reg, uint32_t value);

/**
 * @brief 配置IO APIC重定向表項
 * 
 * @param io_apic_base IO APIC基地址
 * @param irq IRQ號
 * @param vector 中斷向量
 * @param delivery_mode 傳遞模式
 * @param dest_mode 目標模式
 * @param dest 目標APIC ID
 * @param flags 標誌位
 */
void io_apic_setup_redirection(uintptr_t io_apic_base, uint8_t irq, uint8_t vector, 
                              uint8_t delivery_mode, uint8_t dest_mode, uint8_t dest, 
                              uint32_t flags);

/**
 * @brief 獲取IO APIC支持的最大中斷數
 * 
 * @param io_apic_base IO APIC基地址
 * @return int 最大中斷數
 */
int io_apic_max_redirection_entry(uintptr_t io_apic_base);

/**
 * @brief 掩蔽指定IRQ
 * 
 * @param io_apic_base IO APIC基地址
 * @param irq IRQ號
 */
void io_apic_mask_irq(uintptr_t io_apic_base, uint8_t irq);

/**
 * @brief 取消掩蔽指定IRQ
 * 
 * @param io_apic_base IO APIC基地址
 * @param irq IRQ號
 */
void io_apic_unmask_irq(uintptr_t io_apic_base, uint8_t irq);

/**
 * @brief 發送處理器間中斷(IPI)
 * 
 * @param dest_apic_id 目標APIC ID
 * @param delivery_mode 傳遞模式
 * @param vector 中斷向量
 */
void apic_send_ipi(uint8_t dest_apic_id, uint32_t delivery_mode, uint8_t vector);

/**
 * @brief 廣播處理器間中斷(IPI)到所有處理器
 * 
 * @param delivery_mode 傳遞模式
 * @param vector 中斷向量
 * @param exclude_self 是否排除自己
 */
void apic_broadcast_ipi(uint32_t delivery_mode, uint8_t vector, int exclude_self);

/**
 * @brief 禁用PIC(8259A)
 * 必須在啟用APIC前禁用舊的PIC控制器
 */
void pic_disable();

/**
 * @brief 配置本地APIC中斷
 * 
 * @param lint LINT編號 (0或1)
 * @param vector 中斷向量
 * @param delivery_mode 傳遞模式
 * @param masked 是否被掩蔽
 */
void apic_configure_lint(int lint, uint8_t vector, uint32_t delivery_mode, int masked);

/**
 * @brief 取得已映射的 IO APIC 基址 (虛擬地址)
 *
 * 這個函式在 APIC 初始化完成後，回傳內核為 IO APIC
 * 建立的固定虛擬位址。其餘模組應透過此函式來讀寫
 * IO APIC，而不是使用實體位址，避免未映射造成的
 * page fault。
 *
 * @return uintptr_t
 *         - 非 0：IO APIC 的虛擬基址  
 *         - 0：尚未映射或初始化失敗
 */
uintptr_t apic_get_io_apic_base(void);

#endif /* _LUNAIX_APIC_H */