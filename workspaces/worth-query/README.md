# WORTH Query Workspace

This workspace owns the Query engine, its audience facades, and its explicit
cold certification package.

For architecture and usage, start with:

- [`docs/AI_README.md`](./crates/worth-query/docs/AI_README.md) for the authority
  map and current public conventions
- [Runtime-Installed Domains And Operations](./crates/worth-query/docs/domain-capabilities/runtime-installed-domains.md)
- [Conditional Installed Operations](./crates/worth-query/docs/domain-capabilities/conditional-installed-operations.md)
- [Installed Operation Re-Execution And Replay](./crates/worth-query/docs/domain-capabilities/installed-operation-reexecution-and-replay.md)
- [Typed Stops And Remediation Guidance](./crates/worth-query/docs/domain-capabilities/typed-stops-and-remediation-guidance.md)
- [Installed Operation Lineage And Promotion](./crates/worth-query/docs/domain-capabilities/installed-operation-lineage-and-promotion.md)
- [Application Aftermath, External Effects, And Recovery](./crates/worth-query/docs/execution/application-aftermath-and-recovery.md)
- [Granular Live Invalidation](./crates/worth-query/docs/runtime-surfaces/granular-live-invalidation.md)
- [Ordinary Product Workflow](./crates/worth-query-certification/examples/ordinary_product_workflow.rs)
- [Advanced Product Branching](./crates/worth-query-certification/examples/advanced_product_branching.rs)

Product branches, exact composite history, retained observations, live recovery
handles, and pending cleanup are memory-resident. Process loss releases those
capabilities. Restart durability belongs to Store and requires fresh owner
readmission; Query does not serialize live authority.

Use the smallest package that owns the change. Declaration work does not build
installation, execution, publication, replay, or certification:

```text
cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-declaration
```

Installation work does not build execution, publication, replay, or
certification:

```text
cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-installation
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-installation
```

For behavior owned by the main `worth-query` runtime package, choose either a
check or a test command. Do not pre-run check and `--no-run` before the test:

```text
cargo check --manifest-path workspaces/worth-query/Cargo.toml -p worth-query --tests
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query
```

Admission, execution, aftermath progression, publication, and host facade
behavior have separate package owners. Test those owners directly so their
unit tests and doctests run rather than merely compiling as dependencies:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-admission
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-publication
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-host
```

Allocation and reconstruction certification are cold and must be selected only
when their boundary changes or at closeout. Allocation probes install a
process-wide measuring allocator, so they are deliberately absent from the
ordinary execution test binary. The unbounded legacy compiler matrices were
removed rather than retained as an unsupported archive:

```text
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-execution --features allocation-probes --lib -- --test-threads=4
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification -p worth-query-replay
```

`make query-declaration-test`, `make query-installation-test`, and `make
query-test` are short forms of their corresponding ordinary Cargo commands.
`make query-cold-certification` is the sole short form for the complete cold
portfolio above. These targets add no runner, cache, or selection protocol.

The repository root remains an orchestrator and may consume Query through path
dependencies. It does not own Query package membership.
