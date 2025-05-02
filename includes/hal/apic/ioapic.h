#ifndef _SYSTEM_IOAPIC_H
#define _SYSTEM_IOAPIC_H

#include <stdint.h>

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
 * @brief 初始化IO APIC
 * 
 * @param io_apic_id IO APIC ID
 * @param io_apic_addr IO APIC物理地址
 * @return int 成功返回1，失敗返回0
 */
int ioapic_init(uint8_t io_apic_id, uintptr_t io_apic_addr);

/**
 * @brief 取得已映射的 IO APIC 基址 (虛擬地址)
 *
 * 這個函式在 IO APIC 初始化完成後，回傳內核為 IO APIC
 * 建立的固定虛擬位址。其餘模組應透過此函式來讀寫
 * IO APIC，而不是使用實體位址，避免未映射造成的
 * page fault。
 *
 * @return uintptr_t
 *         - 非 0：IO APIC 的虛擬基址  
 *         - 0：尚未映射或初始化失敗
 */
uintptr_t ioapic_get_base(void);

/**
 * @brief 獲取IO APIC ID
 * 
 * @return uint8_t IO APIC ID
 */
uint8_t ioapic_get_id(void);

/**
 * @brief 讀取IO APIC寄存器
 * 
 * @param reg 寄存器索引
 * @return uint32_t 寄存器值
 */
uint32_t ioapic_read(uint8_t reg);

/**
 * @brief 寫入IO APIC寄存器
 * 
 * @param reg 寄存器索引
 * @param value 寄存器值
 */
void ioapic_write(uint8_t reg, uint32_t value);

/**
 * @brief 獲取IO APIC支持的最大中斷數
 * 
 * @return int 最大中斷數
 */
int ioapic_max_redirection_entry(void);

/**
 * @brief 配置IO APIC重定向表項
 * 
 * @param irq IRQ號
 * @param vector 中斷向量
 * @param delivery_mode 傳遞模式
 * @param dest_mode 目標模式
 * @param dest 目標APIC ID
 * @param flags 標誌位
 */
void ioapic_setup_redirection(uint8_t irq, uint8_t vector, 
                             uint8_t delivery_mode, uint8_t dest_mode, 
                             uint8_t dest, uint32_t flags);

/**
 * @brief 掩蔽指定IRQ
 * 
 * @param irq IRQ號
 */
void ioapic_mask_irq(uint8_t irq);

/**
 * @brief 取消掩蔽指定IRQ
 * 
 * @param irq IRQ號
 */
void ioapic_unmask_irq(uint8_t irq);

#endif /* _SYSTEM_IOAPIC_H */