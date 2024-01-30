## Computer Architecture

### Memory Architecture
1. Computer memory can be likened to a room filled with mailboxes of identical size, each mailbox representing a storage unit with a fixed and numbered location.  
2. For instance, in a 1k memory, there are `1*1024` storage units. The key distinction is that while mailboxes can store various items, storage units can only hold a single number.
3. Computers store all computation results in memory. Essentially, anything that needs to be executed is typically loaded into memory first, Not only data but also the program controlling the computer should be stored in memory. Both programs and data are accessed in the same way, but the CPU interprets them differently.

**Storage Unit**
|Word Address  |   |   |   |   |
|--------------|---|---|---|---|
|`0`           |0  |1  |2  |3  |
|`4`           |4  |5  |6  |7  |
|`8`           |8  |9  |10 |11 |
|`12`          |12 |13 |14 |15 |
- Each small rectangle represents a storage unit, with the large rectangle representing memory. Storage units are typically one byte each. Therefore, when representing memory, one grid cell often corresponds to one byte of storage space.

### CPU Architecture
1. CPU fetches and executes one instruction at a time from memory, following the instruction cycle, which includes the fetch-decode-execute steps, often abbreviated as the fetch-execute cycle.

**Components involved in CPU execution include**
|                              |                |
|------------------------------|----------------|
|`Program Counter`             |The Program Counter is used to inform the computer from where to fetch the next instruction. It holds the memory address of the next instruction to be executed. The CPU checks the Program Counter first, fetches the data stored at the specified address, and then passes it to the Instruction Decoder.                                          |
|`Instruction Decoder`         |The Instruction Decoder is responsible for interpreting instructions, including operations like addition, subtraction, multiplication, data movement, and interactions with memory units involved in the process. It typically consists of actual instructions and a list of memory units needed for executing those instructions.                |
|`Data Bus`                    |                |
|`General Registers`           |                |
|`Arithmetic Logic Unit (ALU)` |                |
