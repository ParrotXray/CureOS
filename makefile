include config/make-locations
include config/make-os
include config/make-cc
include config/make-debug-tool

BUILD_MODE := release
CARGO_FLAGS := 
RUST_TARGET := i686-unknown-linux-gnu

INCLUDES := $(patsubst %, -I%, $(INCLUDES_DIR))
SOURCE_FILES := $(shell find -name "*.[cS]")
SRC := $(patsubst ./%, $(OBJECT_DIR)/%.o, $(SOURCE_FILES))

$(OBJECT_DIR):
	@mkdir -p $(OBJECT_DIR)

$(BIN_DIR):
	@mkdir -p $(BIN_DIR)

$(ISO_DIR):
	@mkdir -p $(ISO_DIR)
	@mkdir -p $(ISO_BOOT_DIR)
	@mkdir -p $(ISO_GRUB_DIR)

# 編譯 Rust 代碼
rust-kernel:
	@echo "Building Rust kernel..."
	@cargo build $(CARGO_FLAGS)

# 編譯組合語言文件（如果還需要）
$(OBJECT_DIR)/%.S.o: %.S
	@mkdir -p $(@D)
	@echo "Compiling: $< -> $@"
	@$(CC) $(INCLUDES) -c $< -o $@

# 編譯需要的 C 文件（如果還有）
$(OBJECT_DIR)/%.c.o: %.c 
	@mkdir -p $(@D)
	@echo "Compiling: $< -> $@"
	@$(CC) $(INCLUDES) -c $< -o $@ $(CFLAGS)

# 連結
$(BIN_DIR)/$(OS_BIN): $(OBJECT_DIR) $(BIN_DIR) rust-kernel $(SRC)
	@echo "Linking..."
	@cp target/$(RUST_TARGET)/$(BUILD_MODE)/libcure.a $(OBJECT_DIR)/
	@$(CC) -T linker.ld -o $(BIN_DIR)/$(OS_BIN) $(SRC) $(OBJECT_DIR)/libcure.a $(LDFLAGS)

$(BUILD_DIR)/$(OS_ISO): $(ISO_DIR) $(BIN_DIR)/$(OS_BIN) GRUB_TEMPLATE
	@./config-grub.sh ${OS_NAME} $(ISO_GRUB_DIR)/grub.cfg
	@cp $(BIN_DIR)/$(OS_BIN) $(ISO_BOOT_DIR)
	@grub-mkrescue -o $(BUILD_DIR)/$(OS_ISO) $(ISO_DIR)

all: clean $(BUILD_DIR)/$(OS_ISO)

all-debug: BUILD_MODE := debug
all-debug: CARGO_FLAGS := --features debug
all-debug: O := -Og
all-debug: CFLAGS := -g -std=gnu99 -ffreestanding $(O) $(W) $(ARCH_OPT)
all-debug: LDFLAGS := -g -ffreestanding $(O) -nostdlib -lgcc
all-debug: clean $(BUILD_DIR)/$(OS_ISO)
	@echo "Dumping the disassembled kernel code to $(BUILD_DIR)/dump.txt"
	@$(OBJDUMP) -D $(BIN_DIR)/$(OS_BIN) > $(BUILD_DIR)/dump.txt

clean:
	@rm -rf $(BUILD_DIR)
	@cargo clean

run: $(BUILD_DIR)/$(OS_ISO)
	@qemu-system-i386 -smp 1 -m 1G -rtc base=utc -cdrom $(BUILD_DIR)/$(OS_ISO) -monitor telnet::$(QEMU_MON_PORT),server,nowait &
	@sleep 1
	@telnet 127.0.0.1 $(QEMU_MON_PORT)

debug-qemu: all-debug
	@$(OBJCOPY) --only-keep-debug $(BIN_DIR)/$(OS_BIN) $(BUILD_DIR)/kernel.dbg
	@qemu-system-i386 -smp 1 -m 1G -rtc base=utc -s -S -cdrom $(BUILD_DIR)/$(OS_ISO) -monitor telnet::$(QEMU_MON_PORT),server,nowait &
	@sleep 1
	@$(QEMU_MON_TERM) -e "telnet 127.0.0.1 $(QEMU_MON_PORT)"
	@gdb -s $(BUILD_DIR)/kernel.dbg -ex "target remote localhost:1234"

debug-bochs: all-debug
	@bochs -q -f bochs.cfg