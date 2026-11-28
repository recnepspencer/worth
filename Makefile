# WORTH build targets
#
# Usage:
#   make platform      - build the root platform crates
#   make platform-test - run the root platform crate tests
#   make test          - run everything
#   make relational-allocation-probes - run the worth-relational allocation-slope lane

QUERY_MANIFEST := workspaces/worth-query/Cargo.toml

WORTH_LOG        ?= compact
WORTH_TRACE_DIR  ?=

# -- Platform crate targets (default target dir) ------------------------------

.PHONY: platform
platform:
	cargo build $(ARGS)

.PHONY: platform-test
platform-test:
	WORTH_LOG=$(WORTH_LOG) \
	WORTH_TRACE_DIR=$(WORTH_TRACE_DIR) \
	cargo test $(ARGS)

.PHONY: query-declaration-check
query-declaration-check:
	cargo check --manifest-path $(QUERY_MANIFEST) -p worth-query-declaration --message-format short

.PHONY: query-declaration-test
query-declaration-test:
	cargo test --manifest-path $(QUERY_MANIFEST) -p worth-query-declaration $(ARGS)

.PHONY: query-installation-check
query-installation-check:
	cargo check --manifest-path $(QUERY_MANIFEST) -p worth-query-installation --message-format short

.PHONY: query-installation-test
query-installation-test:
	cargo test --manifest-path $(QUERY_MANIFEST) -p worth-query-installation $(ARGS)

.PHONY: query-check
query-check:
	cargo check --manifest-path $(QUERY_MANIFEST) -p worth-query --tests --message-format short

.PHONY: query-test
query-test:
	cargo test --manifest-path $(QUERY_MANIFEST) -p worth-query $(ARGS)

.PHONY: query-fast
query-fast: query-test

.PHONY: query-cold-certification
query-cold-certification:
	cargo test --manifest-path $(QUERY_MANIFEST) -p worth-query-execution --lib $(ARGS) -- --test-threads=4
	cargo test --manifest-path $(QUERY_MANIFEST) -p worth-query-certification -p worth-query-replay $(ARGS)

# Every suite, or SUITES=relational,bank for a rerun. See the script's header.
.PHONY: query-gate
query-gate:
	python3 scripts/ci/query_gate.py $(SUITES)

.PHONY: relational-allocation-probes
relational-allocation-probes:
	bash scripts/ci/check_relational_allocation_probes.sh

.PHONY: query-workflow-history-scale
query-workflow-history-scale:
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ci/run_query_workflow_history_scale.ps1

.PHONY: query-closeout
query-closeout: query-cold-certification
	cargo test --manifest-path $(QUERY_MANIFEST) --workspace --exclude worth-query-certification --exclude worth-query-replay -- --format terse

.PHONY: platform-check
platform-check:
	cargo check $(ARGS)

# -- Combined -----------------------------------------------------------------

.PHONY: test
test: platform-test query-closeout

.PHONY: check
check: platform-check determinism-guards determinism-golden signal-runtime-guards line-caps boundary-check agent-context-check

# -- Guards -------------------------------------------------------------------

.PHONY: determinism-guards
determinism-guards:
	python3 scripts/ci/check_determinism_guards.py

.PHONY: determinism-golden
determinism-golden:
	bash scripts/ci/check_determinism_golden.sh

.PHONY: signal-runtime-guards
signal-runtime-guards:
	bash scripts/ci/check_signal_runtime_guards.sh

.PHONY: line-caps
line-caps:
	bash scripts/ci/check_workspace_rust_line_caps.sh

.PHONY: boundary-check
boundary-check:
	cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root . --config tools/boundary-check/config/road1.toml

.PHONY: agent-context-check
agent-context-check:
	cargo run --manifest-path tools/agent-context/Cargo.toml -- check --root . --config tools/boundary-check/config/road1.toml

# -- Helpers ------------------------------------------------------------------

.PHONY: clean
clean:
	cargo clean
