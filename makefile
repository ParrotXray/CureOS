include config/make-locations
include config/make-os
include config/make-cc
include config/make-debug-tool

BUILD_MODE := release
CARGO_FLAGS := --release
ifeq ($(BUILD_MODE),debug)
    CARGO_FLAGS :=
endif

RUST_TARGET := x86_64-unknown-none
BUILDER_PKG := cure-builder
DISK_IMAGE := cure-os.img

COLOR_RESET := \033[0m
COLOR_GREEN := \033[32m
COLOR_YELLOW := \033[33m
COLOR_BLUE := \033[34m
COLOR_RED := \033[31m

.PHONY: all rust-build run clean debug-bochs debug-qemu help all-debug

$(BUILD_DIR):
	@rm -rf $(BUILD_DIR)
	@mkdir -p $(BUILD_DIR)

$(BIN_DIR):
	@mkdir -p $(BIN_DIR)

all: clean-build rust-build copy-artifacts

clean-build:
	@echo "$(COLOR_YELLOW)Cleaning build directory...$(COLOR_RESET)"
	@rm -rf $(BUILD_DIR)

rust-build:
	@echo "$(COLOR_BLUE)Building CureOS...$(COLOR_RESET)"
	@cargo build -p $(BUILDER_PKG) $(CARGO_FLAGS)
	@echo "$(COLOR_GREEN)Build complete$(COLOR_RESET)"

copy-artifacts: $(BUILD_DIR) $(BIN_DIR)
	@echo "$(COLOR_BLUE)Copying build artifacts...$(COLOR_RESET)"
	@BIOS_IMG=$$(find target -name "cure-bios.img" -type f | head -1); \
	UEFI_IMG=$$(find target -name "cure-uefi.img" -type f | head -1); \
	KERNEL_BIN=$$(find target/$(RUST_TARGET)/$(BUILD_MODE) -name "cure-kernel" -type f | head -1); \
	if [ -n "$$BIOS_IMG" ]; then \
		echo "$(COLOR_YELLOW)  → $(BUILD_DIR)/cure-bios.img$(COLOR_RESET)"; \
		cp "$$BIOS_IMG" $(BUILD_DIR)/cure-bios.img; \
	fi; \
	if [ -n "$$UEFI_IMG" ]; then \
		echo "$(COLOR_YELLOW)  → $(BUILD_DIR)/cure-uefi.img$(COLOR_RESET)"; \
		cp "$$UEFI_IMG" $(BUILD_DIR)/cure-uefi.img; \
	fi; \
	if [ -n "$$KERNEL_BIN" ]; then \
		echo "$(COLOR_YELLOW)  → $(BIN_DIR)/$(OS_BIN)$(COLOR_RESET)"; \
		cp "$$KERNEL_BIN" $(BIN_DIR)/$(OS_BIN); \
	fi

	@echo "$(COLOR_GREEN)Artifacts copied$(COLOR_RESET)"

all-debug: BUILD_MODE := debug
all-debug: CARGO_FLAGS :=
all-debug: clean-build rust-build copy-artifacts

dump:
	@echo "$(COLOR_BLUE)Generating disassembly dump...$(COLOR_RESET)"
	@if [ -f "$(BIN_DIR)/$(OS_BIN)" ]; then \
		echo "$(COLOR_YELLOW)Dumping to $(BUILD_DIR)/dump.txt$(COLOR_RESET)"; \
		objdump -D $(BIN_DIR)/$(OS_BIN) > $(BUILD_DIR)/dump.txt; \
		echo "$(COLOR_GREEN)Dump complete$(COLOR_RESET)"; \
	else \
		echo "$(COLOR_RED)Error: Kernel binary not found$(COLOR_RESET)"; \
		exit 1; \
	fi

clean:
	@echo "$(COLOR_YELLOW)Cleaning all build artifacts...$(COLOR_RESET)"
	@cargo clean
	@rm -rf $(BUILD_DIR)
	@rm -f $(DISK_IMAGE) bochs.log serial.log serial-debug.log bochs-debug.log
	@echo "$(COLOR_GREEN)Clean complete$(COLOR_RESET)"

run: all
	@echo "$(COLOR_BLUE)Running with QEMU...$(COLOR_RESET)"
	@qemu-system-x86_64 -drive format=raw,file=$(BUILD_DIR)/cure-bios.img \
		-m 128M \
		-serial stdio

run-bochs: all
	@echo "$(COLOR_BLUE)Running with Bochs...$(COLOR_RESET)"
	@bochs -q -f bochs.cfg

debug-qemu: all-debug
	@echo "$(COLOR_BLUE)Starting QEMU in debug mode...$(COLOR_RESET)"
	@qemu-system-x86_64 -drive format=raw,file=$(BUILD_DIR)/cure-bios.img \
		-m 128M \
		-serial stdio \
		-s -S &
	@sleep 1
	@echo ""
	@echo "$(COLOR_YELLOW)QEMU waiting for debugger...$(COLOR_RESET)"
	@echo "$(COLOR_YELLOW)Run in another terminal:$(COLOR_RESET)"
	@echo "  gdb $(BIN_DIR)/$(OS_BIN)"
	@echo "  (gdb) target remote :1234"
	@echo "  (gdb) continue"
	@echo ""

debug-bochs: all-debug
	@echo "$(COLOR_BLUE)Running Bochs in debug mode...$(COLOR_RESET)"
	@echo "$(COLOR_YELLOW)Dump file available at: $(BUILD_DIR)/dump.txt$(COLOR_RESET)"
	@bochs -q -f bochs.cfg

show-artifacts:
	@echo "$(COLOR_BLUE)Build artifacts:$(COLOR_RESET)"
	@echo "  Build directory:  $(BUILD_DIR)/"
	@echo "  Binary directory: $(BIN_DIR)/"
	@echo ""
	@if [ -d $(BUILD_DIR) ]; then \
		echo "$(COLOR_GREEN)Files in $(BUILD_DIR):$(COLOR_RESET)"; \
		ls -lh $(BUILD_DIR)/ 2>/dev/null || echo "  (empty)"; \
	else \
		echo "$(COLOR_YELLOW)  (directory doesn't exist)$(COLOR_RESET)"; \
	fi
	@echo ""
	@if [ -d $(BIN_DIR) ]; then \
		echo "$(COLOR_GREEN)Files in $(BIN_DIR):$(COLOR_RESET)"; \
		ls -lh $(BIN_DIR)/ 2>/dev/null || echo "  (empty)"; \
	else \
		echo "$(COLOR_YELLOW)  (directory doesn't exist)$(COLOR_RESET)"; \
	fi

check:
	@echo "$(COLOR_BLUE)Checking code...$(COLOR_RESET)"
	@cargo check -p cure-kernel
	@cargo check -p $(BUILDER_PKG)
	@echo "$(COLOR_GREEN)Check complete$(COLOR_RESET)"

fmt:
	@echo "$(COLOR_BLUE)Formatting code...$(COLOR_RESET)"
	@cargo fmt
	@echo "$(COLOR_GREEN)Format complete$(COLOR_RESET)"

clippy:
	@echo "$(COLOR_BLUE)Running clippy...$(COLOR_RESET)"
	@cargo clippy
	@echo "$(COLOR_GREEN)Clippy complete$(COLOR_RESET)"

help:
	@echo "$(COLOR_BLUE)CureOS Makefile Commands:$(COLOR_RESET)"
	@echo ""
	@echo "  $(COLOR_GREEN)make all$(COLOR_RESET)            - Clean build/ then build (release)"
	@echo "  $(COLOR_GREEN)make all-debug$(COLOR_RESET)      - Clean build/ then build (debug + dump)"
	@echo "  $(COLOR_GREEN)make dump$(COLOR_RESET)           - Generate disassembly dump"
	@echo "  $(COLOR_GREEN)make run$(COLOR_RESET)            - Build and run with QEMU"
	@echo "  $(COLOR_GREEN)make run-bochs$(COLOR_RESET)      - Build and run with Bochs"
	@echo "  $(COLOR_GREEN)make debug-qemu$(COLOR_RESET)     - Debug build and run with QEMU+GDB"
	@echo "  $(COLOR_GREEN)make debug-bochs$(COLOR_RESET)    - Debug build and run with Bochs"
	@echo "  $(COLOR_GREEN)make clean$(COLOR_RESET)          - Clean all build artifacts"
	@echo "  $(COLOR_GREEN)make clean-build$(COLOR_RESET)    - Clean build/ directory only"
	@echo "  $(COLOR_GREEN)make show-artifacts$(COLOR_RESET) - Show build output locations"
	@echo "  $(COLOR_GREEN)make check$(COLOR_RESET)          - Check code"
	@echo "  $(COLOR_GREEN)make fmt$(COLOR_RESET)            - Format code"
	@echo "  $(COLOR_GREEN)make clippy$(COLOR_RESET)         - Run clippy"
	@echo "  $(COLOR_GREEN)make help$(COLOR_RESET)           - Show this help"
	@echo ""
	@echo "$(COLOR_YELLOW)Debug artifacts:$(COLOR_RESET)"
	@echo "  - $(BUILD_DIR)/dump.txt        - Kernel disassembly (debug mode only)"
	@echo "  - $(BIN_DIR)/$(OS_BIN)         - Kernel binary (for GDB)"
	@echo ""