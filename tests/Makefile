CARGO_BIN:=$(shell which cargo)

.PHONY: integ
integ:
	cd .. && $(CARGO_BIN) build --release
	$(CARGO_BIN) test

.PHONY: integ-all
integ-all: jump
	cd .. && $(CARGO_BIN) build --release
	$(CARGO_BIN) test --features=nightly -- --test-threads=1

.PHONY: bench
bench: jump
	$(CARGO_BIN) bench --features=nightly

.PHONY: bench-all
bench-all: jump
	sudo -E $(CARGO_BIN) bench --features=nightly,cgroups2

.PHONY: jump
jump:
	make -C ./testbins/jump
