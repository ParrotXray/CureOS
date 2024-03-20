## Interduction

### Process of Handling Signals in a Computer:
- Keyboard Signal: `Keyboard -> Kernel -> Window System -> Application`

- File Reading: `User Input -> Application -> Kernel -> File System -> Hardware -> Memory -> Application Operations`

### Functions of the Kernel:
Controls the flow of information between processes
Acts as the gateway for programs to the outside world
Manages the sending and receiving of messages
Restricts unintended access to data by programs
Protects the system from the impact of malicious programs

### Taking GNU/Linux operating system as an example:
Composed of a kernel (Linux) and user applications (from the GNU project and other sources)
The kernel is the core of computer operation but cannot function independently; it requires assistance from applications.

### Classification of Computer Languages:
- Machine Language: Language directly recognized and processed by the computer, given in the form of numbers or numerical strings
- Assembly Language: Similar to machine language but uses easily memorable alphabetical sequences instead of numerical commands
- High-Level Language: Designed to make programming easier, expressed in a form close to natural language

### Learning Content:
Assembly language requires working directly with the machine and is closer to hardware compared to high-level languages.
High-level languages make programming easier, where one high-level language command usually corresponds to several assembly language commands.

<!-- https://stackoverflow.com/questions/48323666/running-32-bit-app-on-64-bit-ubuntu-with-ld-library-path -->
<!-- https://stackoverflow.com/questions/13178501/compiling-32-bit-assembler-on-64-bit-ubuntu -->
### How to compile a program
- install the library
```sh=
sudo apt install binutils
sudo apt update
sudo apt install lib32z1
dpkg -L libc6-i386 | grep -w ld 
sudo apt install libc6-dev-i386
```
- compile
```sh=
i686-elf-gcc -c -g my_program.S
ld -melf_i386 my_program.o -o f2c -lc -I /lib32/ld-linux.so.2
./my_program
echo $?
```
