# WORTH Query Workspace

This workspace owns the Query engine, its audience facades, and its explicit
cold certification package.

Application code imports only the audience facades `worth-query-decl` and
`worth-query-host`. Start with the [API Map](../../docs/api.md), then the
[`worth-query-decl` README](./crates/worth-query-decl/README.md) and the
[`worth-query-host` README](./crates/worth-query-host/README.md). Certification
code also uses `worth-query-replay` and `worth-query-certification`.

The `worth-query` crate is the internal engine. Its `worth_query::facade`
Workspace surface serves platform crates such as `worth-ui-query-binding` and
is not an application API.

For engine internals, start with:

- [`docs/AI_README.md`](./crates/worth-query/docs/AI_README.md), the internal
  architecture map of the Query engine
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

Required output progression rechecks native consumer decision facts before
following its previously consumed dependencies. Sealed child entity/field content
is excluded from the independent-fact check. Changed consumer fields or
consumer-anchored membership can disclose a fresh decision that drops an old edge. If that fresh handler
still reads a pending output, its native read carries the exact settlement and
selected source basis into the required wave. An initial producer without an
accepted consumer output uses the same handoff: the requested chain must become
Current before the caller retries its frozen disclosure. A requested chain never
settles or promotes its parent demand. A reusable cached Ready keeps its original
admitted source and installed producer under the source's existing capacity ticket
until eviction. An exact failed read can claim that cached row temporarily and use
the ordinary installed-executor wave. Caller-owned custody survives disclosure
retry and Pending, then releases after actual caller settlement, refusal or close.
No source is reconstructed from an output identity. If the source or exact row is
absent, the original read denial remains. Its `readmission_failure()` diagnostic
distinguishes basis, settlement, lineage and required-owner lookup misses without
changing the typed denial kind or recovery posture. A child's native content stays child evidence; neither
this disclosure nor the scheduling handoff certifies an output Current. This
proof covers native read refusals; it does not establish recovery when native
publication is Current but managed readiness delivery is still deferred.

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
