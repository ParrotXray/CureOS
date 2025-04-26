#include <hal/cpu.h>
#include <stdint.h>
#include <cpuid.h>

void cpu_get_model(char* model_out) {
    uint32_t* out = (uint32_t*)model_out;
    unsigned int eax, ebx, ecx, edx;
    
    __get_cpuid(0, &eax, &ebx, &ecx, &edx);

    out[0] = ebx;
    out[1] = edx;
    out[2] = ecx;
    model_out[12] = '\0';
}

#define BRAND_LEAF 0x80000000UL

int cpu_brand_string_supported() {
    unsigned int supported = __get_cpuid_max(BRAND_LEAF, 0);
    return (supported >= 0x80000004UL);
}

void cpu_get_brand(char* brand_out) {
    if(!cpu_brand_string_supported()) {
        brand_out[0] = '?';
        brand_out[1] = '\0';
        return;
    }
    uint32_t* out = (uint32_t*) brand_out;
    unsigned int eax, ebx, ecx, edx;
    for (uint32_t i = 2, j = 0; i < 5; i++)
    {
        __get_cpuid(BRAND_LEAF + i, &eax, &ebx, &ecx, &edx);
        out[j] = eax;
        out[j + 1] = ebx;
        out[j + 2] = ecx;
        out[j + 3] = edx;
        j+=4;
    }
    brand_out[48] = '\0';
}

reg32 cpu_r_cr0() {
    reg32 val;
    __asm__ volatile ("mov %%cr0, %0" : "=r" (val));
    return val;
}

reg32 cpu_r_cr2() {
    reg32 val;
    __asm__ volatile ("mov %%cr2, %0" : "=r" (val));
    return val;
}

reg32 cpu_r_cr3() {
    reg32 val;
    __asm__ volatile ("mov %%cr3, %0" : "=r" (val));
    return val;
}

void cpu_w_cr0(reg32 v) {
    __asm__ volatile (
        "mov %0, %%cr0"
        :: "r"(v)
    );
}

void cpu_w_cr2(reg32 v) {
    __asm__ volatile (
        "mov %0, %%cr2"
        :: "r"(v)
    );
}

void cpu_w_cr3(reg32 v) {
    __asm__ volatile (
        "mov %0, %%cr3"
        :: "r"(v)
    );
}