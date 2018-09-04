CARGO_BIN:=$(shell which cargo)

.PHONY: integ
integ:
	cd .. && cargo build --release
	$(CARGO_BIN) test

.PHONY: bench
bench: jump
	$(CARGO_BIN) bench --features=nightly

.PHONY: bench-all
bench-all: jump
	sudo -E $(CARGO_BIN) bench --features=nightly,cgroups2

.PHONY: jump
jump:
	make -C ./testbins/jump
