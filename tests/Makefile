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
	@echo "        Runs pazi integ tests"
	@echo ""
	@echo "    integ-nightly:"
	@echo "        Runs all integ tests, and runs benchmarks once each as integ"
	@echo "        tests (requires rust nightly, root, and cgroupsv2)"
	@echo ""
	@echo "    bench:"
	@echo "        Runs all benchmarks. Requires rust nightly and linux with "
	@echo "        unified cgroups mounted and available. Uses sudo"


.PHONY: integ
integ: pazi
	$(CARGO_BIN) test -- --test-threads=1

.PHONY: integ-nightly
integ-nightly: pazi
	nix run '.#runTestsRoot'

.PHONY: bench
bench: pazi
	nix run '.#runBenches'

.PHONY: pazi
pazi:
	cd .. && $(CARGO_BIN) build --release
