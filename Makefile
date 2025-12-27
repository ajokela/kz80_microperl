# MicroPerl Makefile
# Build the compiler and assemble the Z80 runtime

CARGO = cargo
Z80ASM = z80asm

# Paths
RUNTIME_DIR = runtime
EXAMPLES_DIR = examples
BUILD_DIR = target/release
RUNTIME_BIN = $(RUNTIME_DIR)/microperl.bin

# Default target
.PHONY: all
all: compiler runtime

# Build the compiler
.PHONY: compiler
compiler:
	$(CARGO) build --release

# Assemble the Z80 runtime
.PHONY: runtime
runtime: $(RUNTIME_BIN)

$(RUNTIME_BIN): $(RUNTIME_DIR)/microperl.asm
	@echo "Assembling Z80 runtime..."
	@if command -v $(Z80ASM) >/dev/null 2>&1; then \
		$(Z80ASM) -o $@ $<; \
	else \
		echo "Warning: z80asm not found, skipping runtime assembly"; \
	fi

# Build and run example
.PHONY: example
example: compiler
	$(BUILD_DIR)/microperl --bytecode $(EXAMPLES_DIR)/hello.mpl

# Generate binary from example
.PHONY: hello-bin
hello-bin: compiler
	$(BUILD_DIR)/microperl -o /tmp/hello.mpl.bin $(EXAMPLES_DIR)/hello.mpl
	@echo "Generated /tmp/hello.mpl.bin"
	@xxd /tmp/hello.mpl.bin | head -10

# Test tokenizer
.PHONY: tokens
tokens: compiler
	$(BUILD_DIR)/microperl --tokens $(EXAMPLES_DIR)/hello.mpl

# Test parser
.PHONY: ast
ast: compiler
	$(BUILD_DIR)/microperl --ast $(EXAMPLES_DIR)/hello.mpl

# Clean build artifacts
.PHONY: clean
clean:
	$(CARGO) clean
	rm -f $(RUNTIME_BIN)
	rm -f /tmp/*.mpl.bin

# Install dependencies (macOS)
.PHONY: deps-mac
deps-mac:
	brew install z80asm || true

.PHONY: help
help:
	@echo "MicroPerl Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all        - Build compiler and runtime (default)"
	@echo "  compiler   - Build Rust compiler"
	@echo "  runtime    - Assemble Z80 runtime"
	@echo "  example    - Show bytecode for hello.mpl"
	@echo "  hello-bin  - Generate binary for hello.mpl"
	@echo "  tokens     - Show tokens for hello.mpl"
	@echo "  ast        - Show AST for hello.mpl"
	@echo "  clean      - Clean build artifacts"
	@echo "  deps-mac   - Install dependencies on macOS"
