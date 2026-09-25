# WORTH build targets
#
# UI builds land in target-ui/, kernel builds in target/. Both can run
# simultaneously without Cargo lock contention.
#
# Usage:
#   make ui          â€” build the native Platform Pulse binary
#   make ui-run      â€” run the native Platform Pulse binary
#   make ui-test     â€” run the WORTH UI workspace tests
#   make kernel      â€” build all kernel crates
#   make kernel-test â€” run all kernel tests
#   make test        â€” run everything
#   make trace-view  â€” open trace viewer GUI
#   make relational-allocation-probes - run the worth-relational allocation-slope lane

UI_MANIFEST := workspaces/worth-ui/Cargo.toml
UI_APP      := worth-ui-platform-pulse
UI_TARGET   := $(CURDIR)/target-ui
QUERY_MANIFEST := workspaces/worth-query/Cargo.toml

WORTH_LOG        ?= compact
WORTH_TRACE_DIR  ?=

# â”€â”€ UI targets (isolated target dir) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

.PHONY: ui
ui:
	CARGO_TARGET_DIR=$(UI_TARGET) cargo build --manifest-path $(UI_MANIFEST) -p $(UI_APP) $(ARGS)

.PHONY: ui-release
ui-release:
	CARGO_TARGET_DIR=$(UI_TARGET) cargo build --manifest-path $(UI_MANIFEST) -p $(UI_APP) --release $(ARGS)

.PHONY: ui-run
ui-run:
	CARGO_TARGET_DIR=$(UI_TARGET) cargo run --manifest-path $(UI_MANIFEST) -p $(UI_APP) --bin worth-ui-platform-pulse $(ARGS)

.PHONY: ui-test
ui-test:
	CARGO_TARGET_DIR=$(UI_TARGET) cargo test --manifest-path $(UI_MANIFEST) --workspace $(ARGS)

.PHONY: ui-check
ui-check:
	CARGO_TARGET_DIR=$(UI_TARGET) cargo check --manifest-path $(UI_MANIFEST) --workspace --all-features $(ARGS)

# â”€â”€ Kernel targets (default target dir) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

.PHONY: kernel
kernel:
	cargo build $(ARGS)

.PHONY: kernel-test
kernel-test:
	WORTH_LOG=$(WORTH_LOG) \
	WORTH_TRACE_DIR=$(WORTH_TRACE_DIR) \
	cargo test $(ARGS)

.PHONY: worth-fast
worth-fast: query-fast spatial-fast

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
	cargo test --manifest-path $(QUERY_MANIFEST) -p worth-query-execution --features allocation-probes --lib $(ARGS) -- --test-threads=4
	cargo test --manifest-path $(QUERY_MANIFEST) -p worth-query-certification -p worth-query-replay $(ARGS)

.PHONY: relational-allocation-probes
relational-allocation-probes:
	bash scripts/ci/check_relational_allocation_probes.sh

.PHONY: spatial-fast
spatial-fast:
	cargo check -p worth-spatial --tests --message-format short
	cargo test -p worth-spatial --tests --no-run --message-format short
	cargo test -p worth-spatial --lib -- --format terse
	cargo test -p worth-spatial --test ui -- --format terse

.PHONY: query-workflow-history-scale
query-workflow-history-scale:
	powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ci/run_query_workflow_history_scale.ps1

.PHONY: query-closeout
query-closeout: query-cold-certification
	cargo test --manifest-path $(QUERY_MANIFEST) --workspace --exclude worth-query-certification --exclude worth-query-replay -- --format terse

.PHONY: spatial-public-api-closeout
spatial-public-api-closeout:
	cargo test -p worth-spatial --test public_api_contract -- --format terse

.PHONY: spatial-closeout
spatial-closeout:
	cargo test -p worth-spatial --tests -- --format terse

.PHONY: kernel-check
kernel-check:
	cargo check $(ARGS)

# â”€â”€ Combined â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

.PHONY: test
test: kernel-test ui-test query-closeout

.PHONY: check
check: kernel-check ui-check determinism-guards determinism-golden signal-runtime-guards line-caps boundary-check agent-context-check

# â”€â”€ Trace tooling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

.PHONY: trace-view
trace-view:
	cargo run -p worth-view --bin worth-trace-viewer $(DIR)

.PHONY: trace-issues
trace-issues:
	cargo run -p worth-view --bin worth-trace-cli -- issues $(DIR)

.PHONY: trace-list
trace-list:
	cargo run -p worth-view --bin worth-trace-cli -- list $(DIR)

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

# â”€â”€ Helpers â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€

.PHONY: clean-ui
clean-ui:
	rm -rf $(UI_TARGET)

.PHONY: clean
clean:
	cargo clean
	rm -rf $(UI_TARGET)
