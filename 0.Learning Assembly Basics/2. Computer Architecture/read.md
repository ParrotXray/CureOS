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

Addressing Modes

1. Immediate Addressing Mode:  
   **Description:** The instruction directly includes the data to be accessed.  
   **Example:** Initializing a register to 0, the instruction directly includes the number 0.  

2. Register Addressing Mode:  
   **Description:** The instruction includes the register to be accessed rather than a memory location.  
   **Example:** Operating on the value in a register rather than data in memory.  

3. Direct Addressing Mode:  
   **Description:** The instruction includes the memory address to be accessed.  
   **Example:** Loading data at address 2002 into a register.  

4. Indexed Addressing Mode:  
   **Description:** The instruction includes the memory address and an index register, which stores the offset for that address.  
   **Example:** If the index register contains the number 4, the actual accessed address will be 2002 + 4 = 2006.  

5. Indirect Addressing Mode:  
   **Description:** The instruction includes a register storing a pointer to the data to be accessed.  
   **Example:** Using the indirect addressing mode, the value 4 in the %eax register indicates using the value at memory location 4.  

6. Based Addressing Mode:  
   **Description:** Similar to indirect addressing but requires an additional value called an offset, which is added to the value in the base register for addressing.  
   **Example:** Considering the memory structure storing customer information. If we want to access a customer's age (the eighth byte in the record), and the register holds the starting memory address of this  customer's information, we can use the based addressing mode:  

   - Use the base register as the base pointer.
   - Set the offset to 8.
   - The instruction will use the value in the base register plus the offset to obtain the actual memory address and then retrieve the data at that address, i.e., the customer's age."

### Concept of Scale Factor:

In computer architecture, the scale factor refers to the stride in accessing elements in an array or data structure using the indexed addressing mode. In x86 processor instructions, scale factors are typically values like 2, 4, 8, and so on.

Consider an example where we have an array, and each element occupies 4 bytes. If we want to access this array using the indexed addressing mode, we need to set the scale factor to 4 to correctly access each element.

Let's assume the base address is 2000, and the value in the index register is 3. Using a scale factor of 4, the formula for calculating the actual address is:

Actual Address = Base Address + (Index Register Value × Scale Factor)

Substituting the values:

Actual Address = 2000 + (3 × 4) = 2012

This means that by using the indexed addressing mode with a scale factor of 4, accessing the position of the fourth element starting from address 2000 results in an actual address of 2012.

In this context, the scale factor specifies the spacing between adjacent elements in the array. It allows programmers to conveniently access different positions in the array using the indexed addressing mode.

Simple explanation for accessing a group of numbers where each byte is 1, requiring a move of 3, but when accessing a 4-byte word, it represents a change from 1 to 4, hence 3x4.

In indexed addressing mode, the scale factor is used to specify the stride in accessing elements in an array or data structure. If each element occupies 1 byte, and you want to access a word (4 bytes) in the array, you need to set the scale factor to 4.

In essence, the scale factor tells the computer how many bytes to skip when accessing elements in an array or data structure. In your example, multiplying 3 by 4 indicates that you want a stride of 4 bytes for each access. Therefore, the computer will skip 12 bytes to access the next 4-byte element, ensuring that you access different positions in the array correctly on byte boundaries.
