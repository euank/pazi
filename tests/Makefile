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
	@echo "    integ:"
	@echo "        Runs all integ tests"
	@echo ""
	@echo "    integ-nightly:"
	@echo "        Runs all integ tests, and runs benchmarks once each as integ"
	@echo "        tests (requires rust nightly)"
	@echo ""
	@echo "    integ-nightly-cgroups:"
	@echo "        As above, but requires unified cgroups and sudo. This is known"
	@echo "        to be less flaky for non-pazi autojumpers"
	@echo ""
	@echo "    bench:"
	@echo "        Runs all benchmarks. Requires rust nightly and linux with "
	@echo "        unified cgroups mounted and available. Uses sudo"


.PHONY: integ
integ: pazi
	$(CARGO_BIN) test -- --test-threads=1

.PHONY: integ-nightly
integ-nightly: pazi jump
	$(CARGO_BIN) test --features=nightly -- --test-threads=1

.PHONY: integ-nightly-cgroups
integ-nightly-cgroups: pazi jump
	sudo -E PAZI_TEST_CGROUP=true $(CARGO_BIN) +nightly test --features=nightly -- --test-threads=1

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
