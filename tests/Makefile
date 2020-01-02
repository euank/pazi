# We use 'sudo' below, so cargo might not be in our path anymore
CARGO_BIN:=$(shell which cargo)

.PHONY: help
help:
	@echo "pazi integration tests and benchmarks"
	@echo ""
	@echo "Usage:" 
	@echo "    make [target]"
	@echo ""
	@echo "Targets:"
	@echo "    integ: Runs all integ tests"
	@echo "    integ-nightly: Runs all integ tests, and runs benchmarks once each as integ tests (requires rust nightly)"
	@echo "    bench: runs all benchmarks. Requires rust nightly and linux with unified cgroups mounted and available. Uses sudo"


.PHONY: integ
integ: pazi
	$(CARGO_BIN) test -- --test-threads=1

# Don't use PAZI_TEST_CGROUP to avoid requiring root and so it runs on macos
.PHONY: integ-nightly
integ-nightly: pazi jump
	$(CARGO_BIN) test --features=nightly -- --test-threads=1

# Use PAZI_TEST_CGROUP because it's quicker and more accurate
.PHONY: bench
bench: pazi jump
	sudo -E PAZI_TEST_CGROUP=true $(CARGO_BIN) bench --features=nightly

.PHONY: jump
jump:
	make -C ./testbins/jump

.PHONY: pazi
pazi:
	cd .. && $(CARGO_BIN) build --release
