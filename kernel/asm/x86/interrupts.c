#include <arch/x86/interrupts.h>

void isr0(isr_param* param) {
    tty_clear();
    tty_put_str("!!\tPANIC\t!!");
}

void interrupt_handler(isr_param* param) {
    switch (param->vector) {
        case 0:
            isr0(param);
            break;
    }
}