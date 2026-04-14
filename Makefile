# Makefile for Vela (tvela)

BINARY_NAME=vela
TARGET_NAME=tvela
INSTALL_DIR=$(HOME)/.local/bin

.PHONY: all build install uninstall clean

all: build

build:
	cargo build

install: build
	mkdir -p $(INSTALL_DIR)
	install -m 755 target/debug/$(BINARY_NAME) $(INSTALL_DIR)/$(TARGET_NAME)
	@echo "Installed $(TARGET_NAME) to $(INSTALL_DIR)/$(TARGET_NAME)"
	@echo "Make sure $(INSTALL_DIR) is in your PATH."

uninstall:
	rm -f $(INSTALL_DIR)/$(TARGET_NAME)
	@echo "Uninstalled $(TARGET_NAME)"

clean:
	cargo clean
