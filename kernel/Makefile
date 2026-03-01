KDIR := $(KDIR)
MDIR := $(realpath $(dir $(abspath $(lastword $(MAKEFILE_LIST)))))

$(info -- KDIR: $(KDIR))
$(info -- MDIR: $(MDIR))

.PHONY: all compdb clean format check-format

all: check_symbol
	make -C $(KDIR) M=$(MDIR) modules
	./check_symbol kernelsu.ko $(KDIR)/vmlinux
compdb:
	python3 $(MDIR)/.vscode/generate_compdb.py -O $(KDIR) $(MDIR)
clean:
	make -C $(KDIR) M=$(MDIR) clean
	rm -f check_symbol
	cargo clean --manifest-path $(MDIR)/tools/check_symbol_rs/Cargo.toml
check_symbol: tools/check_symbol_rs/src/main.rs tools/check_symbol_rs/Cargo.toml
	cargo build --release --manifest-path $(MDIR)/tools/check_symbol_rs/Cargo.toml
	cp $(MDIR)/tools/check_symbol_rs/target/release/check_symbol $(MDIR)/check_symbol
format:
	find . \( -name "*.c" -o -name "*.h" \) -print0 | xargs -0 clang-format -i
check-format:
	find . \( -name "*.c" -o -name "*.h" \) -print0 | xargs -0 clang-format --dry-run --Werror

# Keep a new line here!! Because someone may append config
