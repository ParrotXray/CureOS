#include <stdint.h>

#include <system/constants.h>
#include <system/tty/tty.h>

#include <system/mm/page.h>
#include <system/mm/pmm.h>
#include <system/mm/vmm.h>
#include <system/mm/heap.h>
#include <system/mm/heap_debug.h>
#include <system/mm/mempool.h>

#include <hal/acpi/acpi.h>
#include <hal/apic/apic.h>
#include <hal/acpi/smp.h>
#include <hal/apic/apic_timer.h>
#include <hal/apic/irq_setup.h>
#include <drivers/keyboard.h>

#include <hal/cpu.h>
#include <hal/rtc.h>

#include <arch/x86/boot/multiboot.h>
#include <arch/x86/gdt.h>
#include <arch/x86/idt.h>

#include <libc/stdio.h>
#include <libc/stdlib.h>
#include <libc/string.h>

#define BreakPoints for (int32_t i = 0; i < 100000000000; i++);

// 使用數組格式聲明標籤，這樣它們就是有效的指針而不是 void 類型
extern char __kernel_start[];
extern char __kernel_end[];
extern char __init_hhk_end[];
extern char __heap_start[];

void
_kernel_init(multiboot_info_t* mb_info)
{
    _init_idt();

    multiboot_memory_map_t* map = (multiboot_memory_map_t*)mb_info->mmap_addr;

    // TODO: 内核初始化
    //   (v) 根据memory map初始化内存管理器
    //   (v) 分配新的栈空间
    //       调整映射：
    //   ( )    + 映射 memory map （APCI，APIC，IO映射） （以后）
    //   (v)    + 释放 hhk_init 所占据的空间
    //   (v)    + 初始化堆内存管理

#pragma region INIT_MM
    // 初始化物理内存管理器
    pmm_init(MEM_1MB + mb_info->mem_upper << 10);
    vmm_init();
#pragma endregion

    // 初始化VGA
    tty_init(VGA_BUFFER_PADDR);
    tty_set_theme(VGA_COLOR_GREEN, VGA_COLOR_BLACK);

    printf("[KERNEL] === Initialization === \n");
    unsigned int map_size =
      mb_info->mmap_length / sizeof(multiboot_memory_map_t);
    printf("[MM] Mem: %d KiB, Extended Mem: %d KiB\n",
           mb_info->mem_lower,
           mb_info->mem_upper);

#pragma region MMAP_SCAN_RESERVING_KERNEL_PGS
    // 按照 Memory map 标识可用的物理页
    for (unsigned int i = 0; i < map_size; i++) {
        multiboot_memory_map_t mmap = map[i];
        printf("[MM] Base: 0x%x, len: %u KiB, type: %u\n",
               map[i].addr_low,
               map[i].len_low >> 10,
               map[i].type);
        if (mmap.type == MULTIBOOT_MEMORY_AVAILABLE) {
            // 整数向上取整除法
            uintptr_t pg = map[i].addr_low + 0x0fffU;
            pmm_mark_chunk_free(pg >> 12, map[i].len_low >> 12);
            printf("[MM] Freed %u pages start from 0x%x\n",
                   map[i].len_low >> 12,
                   pg & ~0x0fffU);
        }
    }

    // 将内核占据的页设为已占用
    size_t pg_count = (uintptr_t)(__kernel_end - __kernel_start) >> 12;
    pmm_mark_chunk_occupied(V2P(__kernel_start) >> 12, pg_count);
    printf("[MM] Allocated %d pages for kernel.\n", pg_count);
#pragma endregion
    
    size_t vga_buf_pgs = VGA_BUFFER_SIZE >> 12;
    
    // 首先，标记VGA部分为已占用
    pmm_mark_chunk_occupied(VGA_BUFFER_PADDR >> 12, vga_buf_pgs);
    
    // 重映射VGA文本缓冲区（以后会变成显存，i.e., framebuffer）
    for (size_t i = 0; i < vga_buf_pgs; i++)
    {
        vmm_map_page((void*)(VGA_BUFFER_VADDR + (i << 12)), 
                    VGA_BUFFER_PADDR + (i << 12), 
                    PG_PREM_RW, PG_PREM_RW);
    }
    
    // 更新VGA缓冲区位置至虚拟地址
    tty_set_buffer((void*)VGA_BUFFER_VADDR);

    printf("[MM] Mapped VGA to %p.\n", (void*)VGA_BUFFER_VADDR);

    // 为内核创建一个专属栈空间。
    for (size_t i = 0; i < (K_STACK_SIZE >> 12); i++) {
        vmm_alloc_page((void*)(K_STACK_START + (i << 12)), PG_PREM_RW, PG_PREM_RW);
    }
    printf("[MM] Allocated %d pages for stack start at %p\n", K_STACK_SIZE>>12, (void*)K_STACK_START);
    
    // 計算堆需要的頁數 (4MB)
    size_t heap_pages = (4 << 20) >> 12; // 4MB 轉換為頁數
    
    // 為堆區域映射物理頁面
    for (size_t i = 0; i < heap_pages; i++) {
        if (!vmm_alloc_page((void*)((uintptr_t)__heap_start + (i << 12)), PG_PREM_RW, PG_PREM_RW)) {
            // 處理錯誤，但繼續嘗試分配其他頁面
        }
    }
    
    // 測試寫入堆起始處
    *((volatile char*)__heap_start) = 0xAA;
    
    // 初始化内核堆
    kheap = heap_init((uintptr_t)__heap_start, 
                      (uintptr_t)__heap_start + (4 << 20), 
                      0xF0000000); // 起始4MB堆，最大到0xF0000000
    
    printf("[MM] Initialized kernel heap at %p\n", __heap_start);

    printf("[KERNEL] === Initialization Done === \n\n");
}

void
_kernel_post_init() {
    printf("[KERNEL] === Post Initialization === \n");
    size_t hhk_init_pg_count = ((uintptr_t)__init_hhk_end) >> 12;
    printf("[MM] Releaseing %d pages from 0x0.\n", hhk_init_pg_count);

    // 清除 hhk_init 与前1MiB的映射，但保留BIOS区域
    for (size_t i = 0; i < hhk_init_pg_count; i++) {
        // 跳过BIOS区域(0xE0000-0x100000)的映射
        if (i >= (0xE0000 >> 12) && i < (0x100000 >> 12)) {
            continue;
        }
        vmm_unmap_page((void*)(i << 12));
    }
    
    // 初始化 ACPI
    printf("[KERNEL] Initializing ACPI...\n");
    if (!acpi_init()) {
        printf("[KERNEL] Failed to initialize ACPI\n");
        // 可以繼續執行，因為有些系統可能沒有 ACPI
    }        
    

    // 初始化 APIC (依賴 ACPI)
    if (acpi_is_supported()) {
        printf("[KERNEL] Initializing APIC...\n");
        if (!apic_init()) {
            // 如果 APIC 初始化失敗，可能需要回退到 PIC 模式或處理錯誤
            printf("[KERNEL] Failed to initialize APIC\n");
        }
        
        // 初始化SMP
        printf("[KERNEL] Initializing SMP...\n");
        if (smp_init()) {
            // 啟動應用處理器
            smp_start_aps();
        }
    } else {
        printf("[KERNEL] ACPI not supported, skipping APIC/SMP initialization\n");
    }
    
    printf("[KERNEL] === Post Initialization Done === \n\n");
}

void
_kernel_main()
{
    char buf[64];  // 修改為字符數組
    
    printf("Hello higher half kernel world!\nWe are now running in virtual "
           "address space!\n\n");
    
    // 初始化鍵盤驅動程式
    keyboard_init();

    rtc_init();

    uintptr_t k_start = vmm_v2p(__kernel_start);  // 移除了&操作符
    printf("The kernel's base address mapping: %p->%p\n", __kernel_start, (void*)k_start);
    
    // 啟用堆調試
    heap_debug_enable(1);
    
    // 測試堆内存分配
    printf("\n[HEAP] Testing heap memory allocation\n");
    printf("[HEAP] Free memory: %d bytes\n", heap_get_free_size(kheap));
    
    // 打印初始堆佈局
    heap_print_layout(kheap);
    
    // 分配一些内存
    char* str1 = (char*)kmalloc(32);
    char* str2 = (char*)kmalloc(64);
    int* numbers = (int*)calloc(10, sizeof(int));
    
    if (str1 && str2 && numbers) {
        strcpy(str1, "Hello from malloc!");
        strcpy(str2, "This is another allocated string from heap.");
        
        // 檢查 calloc 是否已清零
        printf("[HEAP] str1: %s\n", str1);
        printf("[HEAP] str2: %s\n", str2);
        printf("[HEAP] numbers[0]: %d (should be 0)\n", numbers[0]);
        
        // 打印分配後的堆使用情況
        heap_print_usage(kheap);
        
        // 使用 realloc 擴展 str1
        str1 = (char*)krealloc(str1, 128);
        if (str1) {
            strcpy(str1 + strlen(str1), " Now extended with realloc!");
            printf("[HEAP] str1 after realloc: %s\n", str1);
        }
        
        // 打印堆佈局
        heap_print_layout(kheap);
        
        // 檢查堆完整性
        int errors = heap_check_integrity(kheap);
        if (errors) {
            printf("[HEAP] Found %d errors in heap integrity!\n", errors);
        } else {
            printf("[HEAP] Heap integrity check passed.\n");
        }
        
        // 釋放内存
        free(str1);
        free(str2);
        free(numbers);
        
        printf("[HEAP] All memory freed\n");
        printf("[HEAP] Free memory after freeing: %d bytes\n", heap_get_free_size(kheap));
        
        // 測試内存池
        printf("\n[MEMPOOL] Testing memory pool\n");
        
        // 創建一個能容納100個32字節塊的內存池
        mempool_t* pool = mempool_create(32, 100);
        if (pool) {
            printf("[MEMPOOL] Memory pool created successfully\n");
            
            // 獲取內存池統計信息
            size_t total, free_blocks;
            int usage_percent;
            mempool_get_stats(pool, &total, &free_blocks, &usage_percent);
            printf("[MEMPOOL] Initial stats: total=%d, free=%d, usage=%d%%\n", 
                   total, free_blocks, usage_percent);
            
            // 從內存池分配一些塊
            void* blocks[50];
            for (int i = 0; i < 50; i++) {
                blocks[i] = mempool_alloc(pool);
                if (!blocks[i]) {
                    printf("[MEMPOOL] Failed to allocate block at index %d\n", i);
                    break;
                }
                
                // 寫入一些數據
                sprintf((char*)blocks[i], "Block %d", i);
            }
            
            // 檢查內存池狀態
            mempool_get_stats(pool, &total, &free_blocks, &usage_percent);
            printf("[MEMPOOL] After allocation: total=%d, free=%d, usage=%d%%\n", 
                   total, free_blocks, usage_percent);
            
            // 讀取一些數據
            for (int i = 0; i < 5; i++) {
                printf("[MEMPOOL] Block %d content: %s\n", i, (char*)blocks[i]);
            }
            
            // 釋放一些塊
            for (int i = 0; i < 25; i++) {
                if (mempool_free(pool, blocks[i])) {
                    blocks[i] = NULL;
                }
            }
            
            // 再次檢查內存池狀態
            mempool_get_stats(pool, &total, &free_blocks, &usage_percent);
            printf("[MEMPOOL] After partial free: total=%d, free=%d, usage=%d%%\n", 
                   total, free_blocks, usage_percent);
            
            // 擴展內存池
            if (mempool_expand(pool, 50)) {
                printf("[MEMPOOL] Pool expanded successfully\n");
                
                mempool_get_stats(pool, &total, &free_blocks, &usage_percent);
                printf("[MEMPOOL] After expansion: total=%d, free=%d, usage=%d%%\n", 
                       total, free_blocks, usage_percent);
            } else {
                printf("[MEMPOOL] Failed to expand pool\n");
            }
            
            // 釋放剩餘塊
            for (int i = 25; i < 50; i++) {
                if (blocks[i]) {
                    mempool_free(pool, blocks[i]);
                }
            }
            
            // 最終內存池狀態
            mempool_get_stats(pool, &total, &free_blocks, &usage_percent);
            printf("[MEMPOOL] Final stats: total=%d, free=%d, usage=%d%%\n", 
                   total, free_blocks, usage_percent);
            
            // 銷毀內存池
            mempool_destroy(pool);
            printf("[MEMPOOL] Memory pool destroyed\n");
        } else {
            printf("[MEMPOOL] Failed to create memory pool\n");
        }
        
        // 測試堆分配性能
        printf("\n[HEAP] Testing allocation performance\n");
        
        // 進行大量小分配
        void* small_blocks[1000];
        size_t start_free = heap_get_free_size(kheap);
        
        for (int i = 0; i < 1000; i++) {
            small_blocks[i] = malloc(16);
            if (!small_blocks[i]) {
                printf("[HEAP] Failed to allocate at iteration %d\n", i);
                break;
            }
        }
        
        size_t after_alloc = heap_get_free_size(kheap);
        printf("[HEAP] Allocated 1000 small blocks, memory used: %d bytes\n", 
               start_free - after_alloc);
        
        // 釋放所有小塊
        for (int i = 0; i < 1000; i++) {
            if (small_blocks[i]) {
                free(small_blocks[i]);
            }
        }
        
        size_t after_free = heap_get_free_size(kheap);
        printf("[HEAP] After freeing all blocks, recovered: %d bytes\n", 
               after_free - after_alloc);
        printf("[HEAP] Memory overhead: %d bytes\n", 
               start_free - after_free);
    } else {
        printf("[HEAP] Memory allocation failed\n");
    }
    
    // 最終堆使用情況統計
    heap_print_usage(kheap);
    
    // 最終內存洩漏報告
    heap_print_leaks();
    
    printf("\n[KERNEL] Memory management tests completed successfully\n");
    
    // ACPI/APIC/SMP 測試代碼
    if (acpi_is_supported()) {
        printf("\n[KERNEL] ACPI/APIC Test:\n");
        
        // 顯示CPU信息
        int cpu_count = acpi_get_cpu_count();
        printf("[KERNEL] System has %d CPUs\n", cpu_count);
        
        for (int i = 0; i < cpu_count; i++) {
            int apic_id = acpi_get_cpu_apic_id(i);
            printf("[KERNEL] CPU %d: APIC ID = %d\n", i, apic_id);
        }
        
        // 顯示本地APIC信息
        uint32_t lapic_id = apic_get_id();
        printf("[KERNEL] Current CPU APIC ID: %d\n", lapic_id >> 24);
        
        // 如果多核測試
        if (smp_get_cpu_count() > 1) {
            printf("[KERNEL] This is a multi-core system. BSP is running.\n");
            
            // 獲取所有CPU信息
            for (int i = 0; i < smp_get_cpu_count(); i++) {
                cpu_info_t* info = smp_get_cpu_info(i);
                if (info && info->present) {
                    printf("[KERNEL] CPU %d: APIC ID = %d, %s\n", 
                           i, info->apic_id, 
                           info->is_bsp ? "BSP" : "AP");
                }
            }
            
            // 測試處理器間中斷
            printf("[KERNEL] Testing IPI broadcast...\n");
            // 發送測試IPI（實際上這可能不會有可見效果，因為我們還沒有為AP設置IPI處理程序）
            smp_broadcast_ipi(0x41, 1); // 向量0x41，排除自己
        }
        
        // 測試APIC計時器
        printf("[KERNEL] Testing APIC Timer...\n");
        printf("[KERNEL] Sleeping for 1 second...\n");
        cpu_enable_interrupts();

        // 嘗試睡眠
        apic_timer_sleep(1000); // 睡眠 1 秒
        printf("[KERNEL] Wake up! System uptime: %d ms\n", apic_timer_get_ms());
    }

    // 獲取當前時間
    rtc_datetime_t current_time;
    rtc_get_datetime(&current_time);

    // 使用 sprintf 避免格式問題
    char time_str[32];
    sprintf(time_str, "%u:%u:%u %u/%u/%u", 
        current_time.hour, current_time.minute, current_time.second,
        current_time.day, current_time.month, current_time.year);
    printf("\n[KERNEL] System time: %s\n", time_str);

    // 獲取並顯示時間戳
    uint32_t timestamp = rtc_get_timestamp();
    printf("[KERNEL] Unix timestamp: %u\n", timestamp);

    
    printf("\n[KERNEL] System initialization complete. Entering idle state.\n");

    
    while (1) {
        cpu_idle(); // 等待下一個中斷
    }
    
    // 不應該到達這裡
    // __asm__("int $0\n");
}