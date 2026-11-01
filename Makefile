################################################################################
### sloperator: build, test and check with cargo.
###
### make build | test | test-integration | check | clean | help
### TESTS='<selector> ...' narrows test and test-integration (see `make help`).
################################################################################

SHELL       := bash
.SHELLFLAGS := -eu -o pipefail -c
.ONESHELL:
.DEFAULT_GOAL := help
MAKEFLAGS   += --no-builtin-rules

# Every multi-line recipe needs .ONESHELL (GNU make 3.82+). An older make, such as macOS's /usr/bin/make 3.81, would run
# each recipe line in its own shell and break the test targets partway through, so refuse it up front.
ifeq ($(filter oneshell,$(.FEATURES)),)
$(error sloperator: GNU make 3.82+ required (this is $(MAKE_VERSION)); on macOS: brew install make, then run gmake)
endif

# Cargo features for the integration tier, e.g. `make test-integration FEATURES=metal`.
FEATURES ?= model
# The NFR-3 perf pass builds in the release profile; its own target dir keeps it from replacing the model-enabled
# target/release/sloperator that `make build` produces with a --no-default-features binary.
PERF_TARGET_DIR := target/perf

RUSTUP_HINT  := install Rust: https://rustup.rs (toolchain pinned in rust-toolchain.toml)
NEXTEST_HINT := cargo install --locked cargo-nextest@0.9.150
DENY_HINT    := cargo install --locked cargo-deny@0.20.2

# TESTS -> one nextest filterset. `binary_id#test` selects one test exactly; a bare word is a substring of the binary
# id or test name. nextest rejects a `binary_id(~word)` that matches no binary id and fails the whole filterset, so
# the binary_id clause is emitted only for a word inside one of the binary ids (the lib, the bin and one
# `sloperator::<name>` per tests/*.rs).
EMPTY :=
SPACE := $(EMPTY) $(EMPTY)
HASH  := \#
SEL_TESTS := $(if $(filter undefined,$(origin TESTS)),,$(TESTS))
BINIDS    := sloperator sloperator::bin/sloperator $(patsubst tests/%.rs,sloperator::%,$(wildcard tests/*.rs))
sel_bare   = $(if $(findstring $(1),$(BINIDS)),binary_id(~$(1))|)test(~$(1))
sel_one    = $(if $(findstring $(HASH),$(1)),(binary_id(=$(firstword $(subst $(HASH), ,$(1))))&test(=$(lastword $(subst $(HASH), ,$(1))))),($(call sel_bare,$(1))))
SEL       := $(subst $(SPACE), or ,$(strip $(foreach t,$(SEL_TESTS),$(call sel_one,$(t)))))
nt_filter  = -E '$(1)$(if $(SEL), and ($(SEL)))'

# The release-profile perf pass runs only when no selector is given or a selector names `perf`: nextest builds every
# test binary before it filters, so a targeted run would otherwise pay for a full release build.
RUN_PERF := $(if $(SEL_TESTS),$(findstring perf,$(SEL_TESTS)),all)

# need <tool>,<install hint>: fail with a one-line banner when a tool is missing.
define need
command -v $(1) >/dev/null 2>&1 || { echo "sloperator: '$(1)' not found on PATH — $(2)" >&2; exit 1; }
endef

# count <junit.xml...>: fail when the reports hold no testcase, so a selector that matched nothing is not green.
define count
n=$$(cat $(1) 2>/dev/null | grep -c '<testcase' || true)
if [ "$$n" -eq 0 ]; then echo "sloperator: no test ran (TESTS='$(SEL_TESTS)')" >&2; exit 1; fi
endef

.PHONY: help build test test-integration check clean

help: ## list the targets
	@grep -E '^[a-z-]+:.*## ' $(firstword $(MAKEFILE_LIST)) | awk -F':.*## ' '{printf "  %-17s %s\n", $$1, $$2}'
	echo "  TESTS='<sel> ...'  narrow test / test-integration: 'binary_id#test' exact, a bare word is a substring"
	echo "  FEATURES=<list>    cargo features for test-integration (default: model)"

build: ## release build with default features (model)
	@$(call need,cargo,$(RUSTUP_HINT))
	cargo build --release --locked

test: ## unit tier (no model) plus the release-profile perf test
	@$(call need,cargo,$(RUSTUP_HINT))
	$(call need,cargo-nextest,$(NEXTEST_HINT))
	rm -f target/nextest/ci/junit.xml target/nextest/ci-perf/junit.xml
	rc=0
	cargo nextest run --locked --no-default-features --profile ci --no-tests=pass $(call nt_filter,not binary_id(=sloperator::perf)) || rc=$$?
	if [ -n '$(RUN_PERF)' ]; then
	  cargo nextest run --locked --no-default-features --cargo-profile release --target-dir $(PERF_TARGET_DIR) --profile ci-perf --no-tests=pass $(call nt_filter,binary_id(=sloperator::perf)) || rc=$$?
	else
	  echo "sloperator: perf pass skipped (TESTS names no perf test)" >&2
	fi
	$(call count,target/nextest/ci/junit.xml target/nextest/ci-perf/junit.xml)
	exit $$rc

test-integration: ## integration tier (real model; tests marked #[ignore = "integration tier"])
	@$(call need,cargo,$(RUSTUP_HINT))
	$(call need,cargo-nextest,$(NEXTEST_HINT))
	rm -f target/nextest/it/junit.xml
	rc=0
	cargo nextest run --locked --features $(FEATURES) --profile it --run-ignored only --no-tests=pass $(call nt_filter,all()) || rc=$$?
	$(call count,target/nextest/it/junit.xml)
	exit $$rc

check: ## fmt --check, clippy (no-default and default features, -D warnings), cargo deny check
	@$(call need,cargo,$(RUSTUP_HINT))
	$(call need,cargo-deny,$(DENY_HINT))
	cargo fmt --check
	cargo clippy --locked --all-targets --no-default-features -- -D warnings
	cargo clippy --locked --all-targets -- -D warnings
	cargo deny check

clean: ## remove target/
	rm -rf target
