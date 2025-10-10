include config/make-locations
include config/make-os
include config/make-debug-tool

BUILD_MODE := release
CARGO_FLAGS := --release

# Rust target for bootloader
RUST_TARGET := x86_64-unknown-none

.PHONY: all all-debug clean run debug-qemu debug-bochs rust-kernel

$(BUILD_DIR):
	@mkdir -p $(BUILD_DIR)

$(BIN_DIR):
	@mkdir -p $(BIN_DIR)

$(ISO_DIR):
	@mkdir -p $(ISO_DIR)
	@mkdir -p $(ISO_BOOT_DIR)
	@mkdir -p $(ISO_GRUB_DIR)

# 構建 Rust kernel + bootloader，提取 bin 文件
rust-kernel: $(BUILD_DIR) $(BIN_DIR)
	@echo "Building Rust kernel with bootloader..."
	@cargo build -p cure-builder $(CARGO_FLAGS)
	@echo "Finding kernel binary..."
	@KERNEL_BIN=$$(find target/$(RUST_TARGET)/$(BUILD_MODE) -name "cure-kernel" -type f | head -1); \
	if [ -z "$$KERNEL_BIN" ]; then \
		echo "Error: Kernel binary not found!"; \
		exit 1; \
	fi; \
	echo "Copying kernel binary to $(BIN_DIR)/$(OS_BIN)"; \
	cp "$$KERNEL_BIN" $(BIN_DIR)/$(OS_BIN)

# 創建 ISO（使用 GRUB Multiboot2）
$(BUILD_DIR)/$(OS_ISO): $(ISO_DIR) rust-kernel
	@echo "Creating ISO image..."
	@./config-grub.sh $(OS_NAME) $(ISO_GRUB_DIR)/grub.cfg
	@cp $(BIN_DIR)/$(OS_BIN) $(ISO_BOOT_DIR)/$(OS_BIN)
	@echo "Running grub-mkrescue..."
	@grub-mkrescue -o $(BUILD_DIR)/$(OS_ISO) $(ISO_DIR) 2>&1 || \
	(echo "Error: grub-mkrescue failed. Installing grub-pc-bin..."; \
	sudo apt install -y grub-pc-bin xorriso mtools && \
	grub-mkrescue -o $(BUILD_DIR)/$(OS_ISO) $(ISO_DIR))
	@echo "ISO created at $(BUILD_DIR)/$(OS_ISO)"

all: $(BUILD_DIR)/$(OS_ISO)

all-debug: BUILD_MODE := debug
all-debug: CARGO_FLAGS :=
all-debug: $(BUILD_DIR)/$(OS_ISO)
	@echo "Debug build completed"

clean:
	@rm -rf $(BUILD_DIR)
	@cargo clean
	@echo "Cleaned build artifacts"

# 使用 QEMU 運行 ISO
run: $(BUILD_DIR)/$(OS_ISO)
	@echo "Running $(OS_ISO) with QEMU..."
	@qemu-system-x86_64 -cdrom $(BUILD_DIR)/$(OS_ISO) -m 512M -serial stdio

# 使用 Bochs 運行 ISO
run-bochs: $(BUILD_DIR)/$(OS_ISO)
	@echo "Running ISO with Bochs..."
	@bochs -q -f bochs.cfg

# QEMU 調試模式
debug-qemu: all-debug
	@echo "Starting QEMU in debug mode..."
	@qemu-system-x86_64 -cdrom $(BUILD_DIR)/$(OS_ISO) -m 512M \
		-serial stdio -s -S &
	@sleep 1
	@echo "QEMU waiting for GDB connection on localhost:1234"
	@echo "Run: gdb target/$(RUST_TARGET)/debug/cure-kernel"
	@echo "Then: (gdb) target remote localhost:1234"

# Bochs 調試模式
debug-bochs: all-debug
	@echo "Running Bochs in debug mode..."
	@bochs -q -f bochs.cfg

# 幫助信息
help:
	@echo "CureOS Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all           - Build ISO image (release mode)"
	@echo "  all-debug     - Build ISO image (debug mode)"
	@echo "  clean         - Clean all build artifacts"
	@echo "  run           - Build and run ISO with QEMU"
	@echo "  run-bochs     - Build and run ISO with Bochs"
	@echo "  debug-qemu    - Run QEMU in debug mode"
	@echo "  debug-bochs   - Run Bochs in debug mode"
	@echo "  help          - Show this help message"