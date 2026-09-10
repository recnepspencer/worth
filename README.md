# WORTH

WORTH is a Rust platform for applications where state, authority, derived work,
durability, and user-visible consequences need to remain explicit and
inspectable as the system grows.

The project is built around a simple premise: important facts should not be
reconstructed from strings, booleans, logs, or convention. Meaning is declared,
authority is admitted by its owner, phase progression is visible in types, and
the evidence needed by a later phase is carried forward instead of rediscovered.

WORTH is under active construction. This repository is a platform development
tree rather than a release-stable, single-command application framework. The
major runtimes have their own workspaces, documentation, tests, and lifecycle.

## The platform at a glance

```text
application declarations and domain meaning
                    |
                    v
              WORTH Query
          admission and execution
             /      |       \
            v       v        v
   WORTH Relational |   external effects
   authoritative    |
   graph truth      |
            \       |
             v      v
          Runtime Bridge ---> WORTH Signal
          truth-to-compute     incremental derived work
                    |
                    v
                 WORTH UI
          authored applications and hosts

WORTH Store persists and reconstructs the durable physical world beneath
the authoritative runtimes. WORTH Foundational and WORTH Proof supply the
portable vocabulary and proof progression shared across boundaries.
```

These are cooperating owners, not interchangeable layers. Relational owns
authoritative graph-shaped truth. Signal owns incremental computation. Query
owns application-facing composition and admission. Store owns durable physical
survival. UI owns authored and mounted application presentation. No diagnostic,
serialized record, or equivalent-looking identifier is allowed to impersonate
another subsystem's authority.

## Major components

### WORTH Foundational

[`worth-foundational`](./crates/worth-foundational) owns portable meaning that
must remain identical across runtime boundaries: aspect contracts, canonical
values, identities, provenance, lineage, receipts, and shared diagnostic and
performance vocabulary. It deliberately does not own live execution authority.

Start with its [README](./crates/worth-foundational/README.md).

### WORTH Proof

[`worth-proof`](./crates/worth-proof) provides reusable proof-bearing types and
phase progression: authority and capability witnesses, checked transitions,
freshness, boundary readmission, proof sets, and fixed-shape invariants. Runtime
owners use this substrate to build concrete authority; generic proof cannot open
an owner-specific operation.

Start with its [README](./crates/worth-proof/README.md).

### WORTH Relational

[`worth-relational`](./crates/worth-relational) is the authoritative runtime for
graph-shaped state. It owns entities and relations, transactional writes,
branch-local MVCC, immutable roots, snapshots, history, validation, merge,
publication, and replayable truth. It publishes exact semantic changes without
deciding what an application principal is allowed to do.

Start with its [README](./crates/worth-relational/README.md).

### WORTH Signal

[`worth-signal`](./crates/worth-signal) is a deterministic incremental runtime
for derived work. It owns dependency tracking, producer-local invalidation,
transactional recomputation, rollback, suppression, diagnostics, and historical
execution evidence. Application state remains owned by the application or its
authoritative runtime; Signal owns how declared derived work progresses.

Start with its [README](./crates/worth-signal/README.md). Browser applications
can use the worker-first WebAssembly surface in
[`worth-signal-wasm`](./crates/worth-signal-wasm).

### WORTH Runtime Bridge

[`worth-runtime-bridge`](./crates/worth-runtime-bridge) is the causal protocol
boundary between Relational truth and Signal computation. It installs exact
correspondence between portable semantic dependencies and runtime-local Signal
targets, routes committed changes, and preserves branch, basis, precision, and
causal evidence without giving either runtime the other's authority.

Start with its [README](./crates/worth-runtime-bridge/README.md).

### WORTH Query

[`workspaces/worth-query`](./workspaces/worth-query) is the application-facing
composition runtime. It turns typed application declarations plus evidence from
the runtimes that own truth into admitted, bounded operations. It coordinates
installation, authentication and principal binding, capability admission,
access planning, execution, publication, external-effect posture, aftermath,
idempotency, and recovery.

Query is not a database, identity provider, or policy truth store. It composes
those owners through typed audience facades:

- `worth-query-decl` for application declarations;
- `worth-query-host` for installation, admission, execution, and publication;
- `worth-query-replay` for certification-only reconstruction.

The canonical architectural orientation is
[WORTH Query Orientation for AI Agents](./workspaces/worth-query/crates/worth-query/docs/AI_README.md).
The workspace [README](./workspaces/worth-query/README.md) maps focused packages
and verification lanes.

### WORTH Store

[`workspaces/worth-store`](./workspaces/worth-store) is the durable physical
foundation for WORTH. It is responsible for making accepted physical records
survive process failure, reopening persisted roots in a fresh process, and
reporting corruption or indeterminate outcomes without inventing semantic
truth.

Store includes:

| Responsibility | Main owners |
|---|---|
| Shared physical vocabulary and claim boundaries | `worth-store-contracts`, `worth-store-readiness`, `worth-store-claim-boundaries` |
| Byte layout and media mechanics | `worth-store-physical-format`, `worth-store-physical-backend`, `worth-store-buffer-pool` |
| Durability and bounded execution | `worth-store-wal`, `worth-store-io-scheduler`, the thin `worth-store` facade |
| Restart and recovery | `worth-store-recovery-physics`, `worth-store-recovery-runtime` |
| Integrity and isolation | `worth-store-physical-integrity`, `worth-store-physical-isolation`, `worth-store-security` |
| Durable semantic support | `worth-store-authority`, `worth-store-snapshots`, `worth-store-branch-deltas`, `worth-store-schema-lineage`, `worth-store-live-query` |
| Data lifecycle and scale | `worth-store-retention`, `worth-store-tiering`, `worth-store-replication`, `worth-store-bulk`, `worth-store-blob-chunks` |
| Operations and resource governance | `worth-store-maintenance`, `worth-store-operations`, `worth-store-budgets` |
| Independent evidence | `worth-store-offline-verifier`, `worth-store-offline-integrity-observer`, `worth-store-formal-models`, and the certification crates |

Store does not decide whether a Query operation is legal, redefine Relational
truth, or treat a checksum as authenticity. Its acknowledgements describe
physical truth under a qualified backend and remain separate from semantic
commit authority.

Start with the workspace [README](./workspaces/worth-store/README.md) and the
public facade [README](./workspaces/worth-store/crates/worth-store/README.md).

### WORTH UI

[`workspaces/worth-ui`](./workspaces/worth-ui) is the product-facing UI platform.
It owns authored UI meaning, compilation, active application state, planning,
mounting, interaction and intent admission, runtime services, host exchange,
native lifecycle, Query-backed views, and read-only inspection.

The DSL, runtime, Query binding, host contracts, native host, headless host, and
native platform are separate owners. Native input is not automatically user
intent, UI admission is not domain admission, and inspection receipts do not
grant mutation authority.

Start with [WORTH UI AI Discovery](./workspaces/worth-ui/AI_README.md).

### WORTH Server and browser delivery

[`worth-server`](./crates/worth-server) exposes WORTH-native server operations
and a strict HTTP compatibility boundary for reads, mutations, streams, uploads,
and downloads. Compatibility routes normalize transport-shaped input into
canonical execution rather than allowing HTTP metadata to manufacture basis or
authority.

[`worth-signal-wasm`](./crates/worth-signal-wasm) packages worker-first browser
state, resources, forms, routing, local branch truth, and React integration.
The default path does not silently fall back to the main thread.

### Supporting crates

- [`worth-harness`](./crates/worth-harness) provides shared scenario,
  certification, parity, diagnostics, and hostile-workload infrastructure.
- Crates and packages retaining the older `forge-*` name are migration or
  compatibility surfaces, not the preferred vocabulary for new integrations.

## Repository map

| Path | Contents |
|---|---|
| [`crates`](./crates) | Shared runtime, protocol, foundation, and delivery crates |
| [`workspaces/worth-contracts`](./workspaces/worth-contracts) | Pure public schema contracts and graph-constitution meaning |
| [`workspaces/worth-query`](./workspaces/worth-query) | Query declarations, installation, admission, execution, publication, facades, replay, and certification |
| [`workspaces/worth-store`](./workspaces/worth-store) | Durable physical store, recovery, integrity, operations, and certification |
| [`workspaces/worth-ui`](./workspaces/worth-ui) | UI DSL, runtime, Query binding, hosts, native platform, and product application |
| [`workspaces/worth-query-bank-world`](./workspaces/worth-query-bank-world) | End-to-end banking domain and adapter world for Query integration evidence |
| [`apps`](./apps) and [`packages`](./packages) | Demonstrations and packaged delivery surfaces |
| [`tools`](./tools) and [`scripts`](./scripts) | Boundary enforcement, generated context, release checks, and workspace tooling |
| [`automation`](./automation) | Milestone and task orchestration support |
| [`_docs`](./_docs) | Architecture, specifications, roadmaps, and engineering laws |

## Engineering model

The repository is governed by [AGENTS.md](./AGENTS.md) and the documents under
[`_docs/coding_guidelines`](./_docs/coding_guidelines). The important themes are:

- one authoritative owner for each decision and truth source;
- compiler-visible phase and authority progression;
- public facades rather than deep imports;
- reconstructible derived state;
- bounded ordinary paths and explicit recovery paths;
- tests that reach the real boundary claimed; and
- physical module structure that preserves semantic ownership.

Subsystem READMEs contain their focused development and verification commands.
The root is intentionally a thin orchestrator; use each subsystem's manifest
for focused commands rather than assuming every workspace is a root member.

## License and commercial use

WORTH is **source available** under the
[Business Source License 1.1](./LICENSE). It is not presently OSI open source.

- Reading, evaluation, development, testing, modification, forking, and
  redistribution are permitted by the license.
- Production use is free while the combined annual revenue of the user and its
  affiliates is below **US $10 million**.
- Organizations at or above that threshold need a commercial license for
  production use.
- Each version converts to the Apache License 2.0 no later than four years after
  its first public distribution.

See [Commercial Licensing](./COMMERCIAL-LICENSING.md) or contact
**goldenspencerh@gmail.com**.

Third-party assets and dependencies remain governed by their own license files
and notices.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md). Contributors retain ownership of their
work while granting the project the rights needed to maintain the public,
commercial, and eventual Apache-2.0 licensing model.
