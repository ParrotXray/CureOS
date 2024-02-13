## Writing the First Program

### First Programming Experience
- exit.S Program Structure
```assembly=
# Purpose: A simple program to exit and return a status code to the Linux kernel

# Input: None

# Output: Returns a status code, which can be read using echo $?

# Variables:
#       %eax saves the system call number
#       %ebx saves the return status

.section .data # Defines the data section, used to store data in the program.
.section .text # Defines the text section, used to store the program's instructions.
.globl _start # Declares a global symbol _start, representing the entry point of the program. In Linux programs, _start is the starting execution point of the program.

_start: 
    movl $1, %eax # Linux kernel exit system call number
    movl $0, %ebx # Status code to return to the system

    int $0x80  # Trigger a software interrupt, enter the kernel to execute the corresponding system call, complete the program's exit
```

- Compile
```sh=
i686-elf-gcc -c exit.S

ld exit.o -m elf_i386 -o exit

./exit

echo $?
```
```sh=
output: 0
```
-gdb debug
```sh=
i686-elf-gcc -c -g exit.S
gdb ./exit.S
run
```

### x86 general-purpose registers

| Register | Description                                                                                                                   |
|----------|-------------------------------------------------------------------------------------------------------------------------------|
| `%eax`   | Accumulator register, used for storing function return values, function parameters, and general temporary data.               |
| `%ebx`   | Base register, typically used for storing the base address of data.                                                           |
| `%ecx`   | Counter register, often used in counting operations such as loops.                                                            |
| `%edx`   | Data register, typically used for storing the result of division operations.                                                  |
| `%edi`   | Destination address pointer register, used for storing destination addresses, such as in string operations.                   |
| `%esi`   | Source address pointer register, used for storing source addresses, such as in string operations.                             |
| `%ebp`   | Base pointer register, typically used to point to the current function's stack frame.                                         |
| `%esp`   | Stack pointer register, used to point to the current top of the stack.                                                        |
| `%eip`   | Instruction pointer register, used for storing the address of the next instruction to be executed.                            |
| `%eflags`| Flags register, stores information about the program's execution status, such as condition codes and interrupt enable bits.   |
