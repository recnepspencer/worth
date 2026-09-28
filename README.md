# WORTH

WORTH is a Rust platform for applications where state, authority, derived work,
durability, and user-visible consequences need to remain explicit and
inspectable as the system grows.

The project is built around a simple premise: important facts should not be
reconstructed from strings, booleans, logs, or convention. Meaning is declared,
authority is admitted by its owner, phase progression is visible in types, and
the evidence needed by a later phase is carried forward instead of rediscovered.

WORTH is domain-neutral infrastructure. It does not know what your application
is about: orders, ledgers, documents, designs, or anything else. What it knows
is what may change, who may change it, what a change was based on, and what
exactly happened.

WORTH is under active construction. This repository is a platform development
tree, not yet a single-command application framework with numbered versions. The
major runtimes have their own workspaces, documentation, tests, and lifecycle.

## Start here

| If you want to... | Read |
|---|---|
| Understand why WORTH exists and why it is shaped this way | [Philosophy](docs/philosophy.md) |
| Understand how the platform works, from the substrate up to one request | [How WORTH Works](docs/how-it-works.md) |
| Look up an exact term | [Glossary](docs/glossary.md) |
| **Build an application: declare features and a program, install the application graph, author workflows, adopt a new program** | [**Build an Application**](docs/build-an-application.md) |
| Find which crate to import for any job | [API Map](docs/api.md) |
| Contribute to the platform itself | [Coding Guidelines](docs/coding-guidelines/) and [AGENTS.md](AGENTS.md) |
| Browse every public document | [Documentation index](docs/README.md) |

**AI agents:** read [Philosophy](docs/philosophy.md), then
[How WORTH Works](docs/how-it-works.md), before you write application code.
Then write it from [Build an Application](docs/build-an-application.md): the
exact calls to declare features and a program, install the application graph,
run it, author and run workflows, and adopt a new program on a branch.
A Query application imports only `worth-query-decl` and `worth-query-host`.
Servers, UIs, and standalone runtimes have their own entry points; see
[API Map §6](docs/api.md#6-standalone-runtimes-and-hosts). Everything else is
how the platform keeps its promises.

## The platform at a glance

WORTH is a set of cooperating owners. Each one holds exactly one kind of
authority, and no other crate may mint it.

```text
                ┌───────────────────────────────────────────────────┐
  Application   │  your schema, operations, handlers, workflows     │
                └───────────────────────────────────────────────────┘
                     │ worth-query-decl        │ worth-query-host
                ┌───────────────────────────────────────────────────┐
  Query         │  declare → install → admit → execute → commit →   │
                │  publish; workflows; aftermath and recovery        │
                └───────────────────────────────────────────────────┘
                ┌───────────────────────────────────────────────────┐
  Composition   │  Runtime World: product branches over exact        │
                │  component bases; coordinated publication          │
                └───────────────────────────────────────────────────┘
                ┌────────────┐   ┌────────────────┐   ┌─────────────┐
  Runtimes      │ Relational │──▶│ Runtime Bridge │──▶│   Signal    │
                │ truth      │   │ correspondence │   │ derived     │
                └────────────┘   └────────────────┘   │ computation │
                                                      └─────────────┘
                ┌───────────────────────────────────────────────────┐
  Substrate     │  worth-foundational   shared meaning              │
                │  worth-proof          legality as types           │
                └───────────────────────────────────────────────────┘

  Beside the stack:  Store (durable physical survival) · UI (authored
  applications and hosts) · Server (transport) · Contracts (pure schema)
```

The arrows between the runtimes show how a committed change *flows* at run
time: from Relational truth, through the Bridge, into Signal. The compile-time
dependency order is different. See [Dependency order](#dependency-order).

These are cooperating owners, not interchangeable layers:

- **Relational** owns authoritative graph-shaped truth.
- **Signal** owns incremental computation.
- **Runtime Bridge** owns the causal link between truth changes and computation.
- **Runtime World** owns product branches across both.
- **Query** owns application-facing composition and admission.
- **Store** owns durable physical survival.
- **UI** owns authored and mounted application presentation.

No diagnostic, serialized record, or equivalent-looking identifier is allowed
to impersonate another subsystem's authority.

## Crate map

Status legend:

- **Stable**: the public export list is snapshotted, and boundary enforcement
  fails on drift.
- **Active**: maintained and in use; the API may still change.
- **Dormant**: present, but nothing active depends on it.

Audience legend:

- **Consumer**: application code may import it. The two Query consumer crates
  are the *audience facades* for applications.
- **Internal**: a platform owner; applications reach it only through a facade.
- **Certification**: tests and evidence.
- **Tooling**: build and enforcement tools.

### Substrate and runtimes (`crates/`)

| Crate | What it does | Audience | Status | Start with |
|---|---|---|---|---|
| `worth-proof` | Legality as types: authority witnesses, phase-typed artifacts, checked transitions, freshness and readmission, performed evidence, linear lifecycles. It has zero dependencies and no clock or counter. | Internal | Active | [README](crates/worth-proof/README.md) |
| `worth-foundational` | The shared dictionary: canonical values, aspect contracts, canonical basis and SHA-256 digests, identity categories, branch-reference nouns, provenance, lineage, receipts, diagnostics, profiles. | Internal | Active | [README](crates/worth-foundational/README.md) |
| `worth-relational` | Authoritative truth for graph-shaped state: entities, relations, aspects, branch-local MVCC over immutable roots, compare-and-publish, settlement, history, replay. | Internal (standalone use possible) | Active | [README](crates/worth-relational/README.md) |
| `worth-signal` | Deterministic incremental derived computation: dependency tracking, scoped invalidation, recompute, rollback, condition decisions, performed execution receipts. It is never truth. | Consumer (standalone) and internal | Active | [README](crates/worth-signal/README.md) |
| `worth-signal-wasm` | A worker-first WebAssembly and TypeScript browser package for Signal, published to npm as `worth-signals-wasm`. | Consumer (npm) | Active | [README](crates/worth-signal-wasm/README.md) |
| `worth-runtime-bridge` | The causal protocol between Relational and Signal: installed correspondence, exact routing of committed changes, lowering of conditional meaning. | Internal | Active | [README](crates/worth-runtime-bridge/README.md) |
| `worth-runtime-world` | Product branches over exact Relational and Signal bases, immutable composite history, and coordinated publication across component owners. | Internal | Active | [README](crates/worth-runtime-world/README.md) |
| `worth-server` | A typed server facade and transport boundary: WORTH-native operations plus a strict HTTP compatibility boundary. | Consumer | Active | [Docs](crates/worth-server/docs/README.md) |
| `worth-server-client-generation` | Renders TypeScript and Python remote clients from a server protocol catalog. | Tooling | Dormant | — |
| `worth-harness` | Scenario, capture, replay, comparison, and workload infrastructure for certification. It is not a product API. | Certification | Active | [README](crates/worth-harness/README.md) |

### Query (`workspaces/worth-query/`)

Query turns application meaning into governed, bounded work. It is not a
database, identity provider, or policy truth store: it composes those owners.

| Crate | What it does | Audience | Status |
|---|---|---|---|
| [`worth-query-decl`](workspaces/worth-query/crates/worth-query-decl/README.md) | Declaration facade: schema, operations, queries, capabilities, workflows, and programs, as types and macros. It adds no behavior. | **Consumer** | **Stable** |
| [`worth-query-host`](workspaces/worth-query/crates/worth-query-host/README.md) | Host facade: installation, requests, admission, execution, commit outcomes, publication, branches, programs, workflows, and aftermath. | **Consumer** | **Stable** |
| [`worth-query-replay`](workspaces/worth-query/crates/worth-query-replay/README.md) | Replay and reconstruction, for certification code only. | Certification | Stable |
| `worth-query-certification` | Reusable certification support, and runnable examples of complete applications. | Certification | Stable |
| `worth-query-declaration` | Owns canonical declarations, validation, and binding grammar. | Internal | Active |
| `worth-query-installation` | Owns portable packages, installed contracts, graph obligations, and touch contracts. | Internal | Active |
| `worth-query-admission` | Owns basis, policy, resource, and access-planning decisions. | Internal | Active |
| `worth-query-execution` | Owns execution over admitted plans, the primary graph, product branches, and program runtimes. | Internal | Active |
| `worth-query-publication` | Owns the request API, disclosure, and publication. | Internal | Active |
| `worth-query-package-archive` | The deterministic archive format for publishing portable Query packages. | Internal (package publishing) | Active |
| `worth-query` | The internal Query engine, used by the UI binding and the server. | Internal | Active |

Workspace guide: [workspaces/worth-query/README.md](workspaces/worth-query/README.md).

### Contracts (`workspaces/worth-contracts/`)

| Crate | What it does | Audience | Status |
|---|---|---|---|
| `worth-schema-core` | Pure schema vocabulary: stable identities and validated names, plus the first measurement types (a tolerance, and length and angle units). | Consumer (schema) | Stable |
| `worth-schema-graph` | Pure graph meaning: how a part inside a larger published record keeps a stable identity when it is promoted to a record of its own. Runtime authority stays with the adopting owners. | Consumer (schema) | Stable |

Schema crates never import Query. Guide:
[workspaces/worth-contracts/README.md](workspaces/worth-contracts/README.md).

### Store (`workspaces/worth-store/`)

Store is the durable physical foundation. It makes accepted physical records
survive process failure, reopens persisted roots in a fresh process, and
reports corruption or indeterminate outcomes without inventing semantic truth.
Store does not decide whether a Query operation is legal, does not redefine
Relational truth, and does not treat a checksum as proof of authenticity.
Today no crate outside the Store workspace depends on it.

| Group | Crates | Status |
|---|---|---|
| Public facade and physical runtime lifecycle | `worth-store` | Active |
| Shared vocabulary and boundary claims | `worth-store-contracts`, `-aspect-native`, `-authority`, `-readiness` | Active |
| Physical format and media | `-physical-format`, `-physical-backend`, `-buffer-pool`, `-io-scheduler`, `-blob-chunks`, `-lsm-authority`, `-layout-indexes` | Active |
| Durability and recovery | `-wal`, `-modes`, `-recovery-physics`, `-recovery-runtime` | Active |
| Integrity, isolation, security | `-physical-integrity`, `-physical-isolation`, `-security`, `-reclaim-policy` | Active |
| Data lifecycle and operations | `-retention`, `-tiering`, `-replication`, `-operations`, `-budgets`, `-maintenance`, `-compatibility` | Active |
| Semantic durable programs | `-snapshots`, `-branch-deltas`, `-schema-lineage`, `-live-query`, `-subscription-support`, `-bulk`, `-extensions`, `-analysis`, `-claim-boundaries` | Dormant |
| Independent evidence | `-offline-verifier`, `-offline-integrity-observer`, `-formal-models`, `-physical-certification`, `-certification`, `-test-support` | Active (certification) |

Guides: [workspace README](workspaces/worth-store/README.md) and
[facade README](workspaces/worth-store/crates/worth-store/README.md).

### UI (`workspaces/worth-ui/`)

The product-facing UI platform. It owns authored UI meaning, compilation,
active application state, planning, mounting, interaction and intent
admission, runtime services, host exchange, native lifecycle, Query-backed
views, and read-only inspection. Native input is not automatically user
intent, UI admission is not domain admission, and inspection receipts do not
grant mutation authority.

| Crate | What it does | Audience |
|---|---|---|
| `worth-ui` | The public UI facade | Consumer |
| `worth-ui-native-platform` | The sole native-display entry point | Consumer |
| `worth-ui-runtime` | Active application, planning, mounting, intent admission, publication, runtime services | Internal |
| `worth-ui-dsl` | Authored syntax, diagnostics, normalization, sealed semantic packages | Internal |
| `worth-ui-query-binding` | The only crate that turns Query products into UI registrations and observations | Internal |
| `worth-ui-host-contract`, `-host-native`, `-host-headless` | Host protocol and the native and headless hosts | Internal |
| `worth-ui-inspection`, `-text`, `-retained-order` | Inspection, text qualification and layout, bounded sequences | Internal |
| `worth-ui-certification`, `-test-support` | Lifecycle and anti-bypass proofs | Certification |
| `apps/platform-pulse` | The permanent reference native application | Reference app |

Guide: [WORTH UI AI Discovery](workspaces/worth-ui/AI_README.md).

### Reference applications

| Path | What it is |
|---|---|
| [`workspaces/worth-query-bank-world`](workspaces/worth-query-bank-world/docs/public-consumer-contract.md) | A complete banking domain, server, HTTP adapter, user node, and fault-injecting external service. It consumes Query **only** through `worth-query-decl` and `worth-query-host`. This is the best end-to-end example of the consumer boundary. |
| `workspaces/worth-ui/apps/platform-pulse` | A native application that exercises the UI and Query path |
| [`apps/WORTH-signal-demo`](apps/WORTH-signal-demo/README.md) | A browser demo of `worth-signals-wasm` |

## Dependency order

This is the compile-time order, from the bottom up. It differs from the
run-time flow in the diagram above.

```text
worth-proof
  └─ worth-foundational
       ├─ worth-relational
       └─ worth-signal
            └─ worth-runtime-bridge      (depends on Relational and Signal)
                 └─ worth-runtime-world
                      └─ Query authority crates
                           └─ worth-query-host, worth-query (engine)
                                └─ worth-server, worth-ui, reference apps
```

`worth-query-decl` depends only on `worth-query-declaration`.
`worth-query-host` sits on the five Query authority crates and does not depend
on the internal engine. Neither consumer facade depends directly on
`worth-proof`. Applications receive authority from its owners; they never
construct it.

Relational and Signal know nothing of each other or of the Bridge. The Bridge
depends on both. It defines the truth-source traits (`CommittedPatchSource`,
`SnapshotReadSource`, `TruthBranchHeadSource`) and owns the Relational adapter
that implements them over Relational's public `change_source` API.

## Repository map

| Path | Contents |
|---|---|
| [`crates`](crates) | Substrate, runtimes, server, and harness |
| [`workspaces/worth-query`](workspaces/worth-query) | Query: declarations, installation, admission, execution, publication, facades, replay, certification |
| [`workspaces/worth-contracts`](workspaces/worth-contracts) | Pure public schema contracts |
| [`workspaces/worth-store`](workspaces/worth-store) | Durable physical store, recovery, integrity, operations, certification |
| [`workspaces/worth-ui`](workspaces/worth-ui) | UI DSL, runtime, Query binding, hosts, native platform, reference app |
| [`workspaces/worth-query-bank-world`](workspaces/worth-query-bank-world) | End-to-end reference domain for the Query consumer boundary |
| [`apps`](apps) | Demonstrations |
| [`docs`](docs) | Public documentation: philosophy, how it works, glossary, API map, coding guidelines |
| [`plans`](plans) | Internal milestone plans, visions, and roadmaps: intent, not documentation |
| [`tools`](tools) | Boundary enforcement (`boundary-check`), generated agent context (`agent-context`), package publishing and qualification tools |
| [`scripts`](scripts) | CI guards, WebAssembly packaging, quality and migration scripts |
| [`automation`](automation) | Local orchestration for long implementation runs |
| [`skills`](skills) | Agent skill definitions |

The root `Cargo.toml` is a thin orchestrator. Each workspace has its own
manifest. Use `--manifest-path` and `-p` for focused commands, for example:

```bash
cargo test --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-host
```

## Engineering model

The repository is governed by [AGENTS.md](AGENTS.md) and the documents under
[`docs/coding-guidelines`](docs/coding-guidelines). The important themes are:

- one authoritative owner for each decision and truth source;
- compiler-visible phase and authority progression;
- public facades rather than deep imports;
- reconstructible derived state;
- bounded ordinary paths and explicit recovery paths;
- tests that reach the real boundary claimed; and
- physical module structure that preserves semantic ownership.

These laws are enforced mechanically. `tools/boundary-check` rejects illegal
dependencies, band violations, and drift in the public facade exports.
`tools/agent-context` generates each crate's `AGENT_CONTEXT.md` from the same
model. Both are required local gates (see [AGENTS.md](AGENTS.md)):

```bash
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
```

```bash
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
```

Subsystem READMEs contain their focused development and verification commands.

## License and commercial use

WORTH is **source available** under the
[Business Source License 1.1](LICENSE). It is not presently OSI open source.

- Reading, evaluation, development, testing, modification, forking, and
  redistribution are permitted by the license.
- Production use is free while the combined annual revenue of the user and its
  affiliates is below **US $10 million**. After crossing the threshold, the
  grant continues for 90 days after the end of that fiscal year.
- Organizations at or above that threshold need a commercial license for
  production use.
- Each version converts to the Apache License 2.0 no later than four years after
  its first public distribution.

See [Commercial Licensing](COMMERCIAL-LICENSING.md) or contact
**goldenspencerh@gmail.com**.

Third-party assets and dependencies remain governed by their own license files
and notices.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Contributors retain ownership of their
work while granting the project the rights needed to maintain the public,
commercial, and eventual Apache-2.0 licensing model.
