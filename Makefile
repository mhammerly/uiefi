BINARY=target/x86_64-unknown-uefi/debug/efi_hello.efi
FS=scripts/fat32.fs

.PHONY: start
start: $(BINARY) $(FS)
	./scripts/start_efi_qemu.sh

.PHONY: makefs
makefs: $(FS)

.PHONY: build
build: $(BINARY)

sources:= $(shell find src -type f -name "*.rs")

$(BINARY): $(sources) Cargo.toml Cargo.lock
	cargo build

$(FS): $(BINARY)
	mkdir -p scripts/put_on_esp/EFI/BOOT
	cp $(BINARY) scripts/put_on_esp/EFI/BOOT/BOOTX64.efi
	./scripts/make_esp.sh

.PHONY: clean
clean:
	rm -r target
