# Rinux Kernel Makefile

VERSION = 0
PATCHLEVEL = 1
SUBLEVEL = 0
EXTRAVERSION =
NAME = Rusty Start

ARCH ?= x86_64
TARGET = $(ARCH)-unknown-rinux
KERNEL = target/$(TARGET)/release/rinux

# Build flags
RUSTFLAGS = -C link-arg=-nostartfiles
CARGOFLAGS = --target $(TARGET) -Z build-std=core,compiler_builtins,alloc

# QEMU settings
QEMU = qemu-system-x86_64
QEMU_FLAGS = -M q35 \
             -cpu max \
             -m 256M \
             -no-reboot \
             -no-shutdown \
             -serial stdio

.PHONY: all build run clean test fmt clippy doc

all: build

build:
	@echo "Building Rinux kernel..."
	@cargo +nightly build --release $(CARGOFLAGS)

run: build
	@echo "Running Rinux in QEMU..."
	@$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL)

debug: build
	@echo "Running Rinux in QEMU with debugging..."
	@$(QEMU) $(QEMU_FLAGS) -kernel $(KERNEL) -s -S

test:
	@echo "Running tests..."
	@cargo +nightly test $(CARGOFLAGS)

fmt:
	@cargo fmt --all

clippy:
	@cargo +nightly clippy $(CARGOFLAGS)

doc:
	@cargo doc --no-deps --open

clean:
	@cargo clean
	@find . -name "*.o" -delete
	@find . -name "*.bin" -delete

help:
	@echo "Rinux Kernel Build System"
	@echo ""
	@echo "Targets:"
	@echo "  build   - Build the kernel"
	@echo "  run     - Run the kernel in QEMU"
	@echo "  debug   - Run the kernel with debugging enabled"
	@echo "  test    - Run tests"
	@echo "  fmt     - Format code"
	@echo "  clippy  - Run Clippy lints"
	@echo "  doc     - Generate documentation"
	@echo "  clean   - Clean build artifacts"
	@echo "  help    - Show this help message"
