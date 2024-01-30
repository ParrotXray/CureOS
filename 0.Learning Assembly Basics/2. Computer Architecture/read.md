## Computer Architecture

### Memory Architecture
1. Computer memory can be likened to a room filled with mailboxes of identical size, each mailbox representing a storage unit with a fixed and numbered location.  
2. For instance, in a 1k memory, there are `1 * 1024` storage units. The key distinction is that while mailboxes can store various items, storage units can only hold a single number.
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
|                              |                                                                                                                         |
|------------------------------|-------------------------------------------------------------------------------------------------------------------------|
|`Program Counter`             |The Program Counter is used to inform the computer from where to fetch the next instruction. It holds the memory address of the next instruction to be executed. The CPU checks the Program Counter first, fetches the data stored at the specified address, and then passes it to the Instruction Decoder.                                          |
|`Instruction Decoder`         |The Instruction Decoder is responsible for interpreting instructions, including operations like addition, subtraction, multiplication, data movement, and interactions with memory units involved in the process. It typically consists of actual instructions and a list of memory units needed for executing those instructions.                |
|`Data Bus`                    |Data Bus is a physical pathway connecting the CPU and memory, used to fetch computational data stored in memory units.   |
|`General Registers`           |Used for performing primary operations such as addition, subtraction, multiplication, comparison, etc. Most of the data is stored in the main memory, and it is only fetched into general registers when needed. After processing, the results are then returned to the memory.                                                                    |
|`Arithmetic Logic Unit (ALU)` |The ALU is the actual execution unit responsible for processing the data fetched by the CPU and decoding the instructions. The computed results are transmitted through the data bus to the corresponding memory unit or register specified by the instruction.                                                                                      |

### Terminology
1. Computer memory consists of a series of numbered, fixed-size storage units, each with a unique address and a size of one byte. On x86 processors, one byte corresponds to a number between 0 and 255.

2. Computers interpret numbers using specialized hardware (such as a graphics card) to convert them into text, graphics, or other forms, often following the ASCII code table, where each number displays as the corresponding character on the screen.

3. For numbers greater than 255, multiple bytes can be combined. For example, two bytes represent a range from 0 to 65535 [`256 * 256`], and four bytes represent a range from 0 to 4294967295[`256 * 256 * 256 * 256`].

4. Registers are special storage units inside the computer used to hold data needed during calculations. Each register is 4 bytes in size, and on x86 processors, the word size is 4 bytes.

5. Addresses have a length of 4 bytes, suitable for storage in registers. Addresses in memory are also referred to as pointers, pointing to different locations in memory.

6. Computer instructions are stored in memory, and instructions are loaded from the address pointed to by the instruction pointer at a given moment.

7. Computers cannot distinguish between programs and other types of data; instructions and data are stored in memory in the same way.

### Understanding Memory

1. Computers faithfully execute programmers' instructions, whether these instructions make sense or not. Programmers need to have a precise understanding of how data is stored in memory.

2. Computers can only store numbers. Text, images, music, etc., exist in the computer in numeric form, and only the corresponding programs know how to interpret them.

3. Customer information can be stored by specifying the starting address of each field in memory. For example, the storage format in memory for fields like name, address, age, and customer ID.

4. Adopting a fixed-length storage method may limit field lengths. This can be addressed by using pointers to point to information in records, allowing flexible storage of different data lengths.

5. Using pointers, records can include addresses pointing to actual data, without being restricted by fixed lengths. This increases storage flexibility.

6. When the field length in records is variable, determining where the next field starts may become more complex.

- If you want to store customer information in memory, one way is to first set the maximum amount of space for customer name and address (e.g., each occupying 50 ASCII characters, or 50 bytes), and then represent customer age and customer ID with numbers. Using this storage method, the memory space occupied by a customer record would be as follows:

- Record Start:  
Customer Name (50 bytes) -> Name Record Start  
Customer Address (50 bytes) -> Address Record Start + 50 bytes (Customer Name)  
Age (1 word -> 4 bytes) -> Age Record Start + 100 bytes (Customer Name + Address)  
Customer ID (1 word -> 4 bytes) -> ID Record Start + 104 bytes (Customer Name + Address + Age)

- In this storage method, as long as you have the starting address of the customer record, you can determine where the other data for that customer is located. However, this storage method imposes limits on customer name and address, restricting them to 50 ASCII characters, which may not meet practical requirements.

- How to eliminate such limitations? We can adopt another storage method: using pointers to information within records. For example, you can replace the customer name field with a pointer pointing to the customer's name.

- Record Start:  
Customer Name Pointer (1 word) -> Name Record Start  
Customer Address Pointer (1 word) -> Address Record Start + 4 bytes  
Customer Age (1 word) -> Age Record Start + 8 bytes  
Customer ID (1 word) -> ID Record Start + 12 bytes  
