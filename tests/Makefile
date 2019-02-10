CARGO_BIN:=$(shell which cargo)

.PHONY: integ
integ: pazi
	$(CARGO_BIN) test -- --test-threads=1

integ-nightly: pazi jump
	$(CARGO_BIN) test --features=nightly -- --test-threads=1

.PHONY: integ-all
integ-all: pazi jump
	test_bin=$$($(CARGO_BIN) test --no-run --features=nightly,cgroups2 --message-format=json | jq -r "select(.profile.test == true) | .filenames[]") ;\
		sudo CARGO_MANIFEST_DIR=$$PWD $$test_bin --test-threads=1

.PHONY: bench
bench: pazi jump
	$(CARGO_BIN) bench --features=nightly

.PHONY: bench-all
bench-all: pazi jump
	sudo -E $(CARGO_BIN) bench --features=nightly,cgroups2

.PHONY: jump
jump:
	make -C ./testbins/jump

.PHONY: pazi
pazi:
	cd .. && $(CARGO_BIN) build --release
