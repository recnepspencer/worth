# API Map

> **Audience:** application developers and AI agents deciding which WORTH crate
> to import. **Read first:** [Philosophy](philosophy.md) and
> [How WORTH Works](how-it-works.md). **Exact terms:** [Glossary](glossary.md).

This page answers three questions:

1. Which crates may my code import?
2. Where does each capability live on those crates?
3. How do I read the reference documentation for them?

It does not repeat the per-crate guides. It routes you to them.

---

## 1. The rule in one sentence

**Application code imports `worth-query-decl` to say what things mean, and
`worth-query-host` to run them. Nothing else in the Query workspace is for
applications.**

Everything else falls into one of four groups:

| Group | Crates | Import it when |
|---|---|---|
| **Application API** | `worth-query-decl`, `worth-query-host` | You are building an application on WORTH. This is the default. |
| **Certification API** | `worth-query-replay`, `worth-query-certification` | You are writing tests or evidence that must replay or compare Query executions. Never from ordinary code. |
| **Schema contracts** | `worth-schema-core`, `worth-schema-graph` | You are writing pure, reusable meaning that must not know about Query. |
| **Standalone runtimes** | `worth-signal`, `worth-signal-wasm` (npm package `worth-signals-wasm`), `worth-relational`, `worth-server`, `worth-ui`, `worth-store` | You want one runtime on its own, or you are building a server or UI host. |

Every other crate is an **internal owner**. It holds authority that the
facades expose on its behalf. See [§7](#7-internal-crates-do-not-import).

> **For AI agents.** If you are about to add `worth-query`,
> `worth-query-execution`, `worth-query-admission`, `worth-query-installation`,
> `worth-query-publication`, `worth-query-declaration`, `worth-proof`,
> `worth-runtime-bridge`, or `worth-runtime-world` to an application's
> `Cargo.toml`, stop. The capability you need is already on
> `worth_query_host::facade` or `worth_query_decl::facade`. Search there first.

---

## 2. Stability

| Level | Meaning | Crates |
|---|---|---|
| **Stable** | The exported names are snapshotted in `tools/boundary-check/snapshots/facades.toml`. Adding, removing, or renaming an export fails boundary-check until the snapshot is updated deliberately. | `worth-query-decl`, `worth-query-host` (including `facade::primary_graph` and `facade::provisional_aftermath`), `worth-query-replay`, `worth-query-certification`, `worth-schema-core`, `worth-schema-graph` |
| **Active** | Maintained and used, with a documented entry point, but not snapshotted. Expect change. | `worth-signal`, `worth-signal-wasm`, `worth-relational`, `worth-server`, `worth-ui`, `worth-ui-native-platform`, `worth-store` |
| **Dormant** | Present, but nothing active depends on it. | `worth-server-client-generation`; nine Store crates for semantic durable programs |

Separately from status, every crate in [§7](#7-internal-crates-do-not-import) is **internal**: whatever its status, applications reach it only through the facades above.

WORTH does not yet publish numbered versions. "Stable" means that a change
to the surface is visible and deliberate, not that it cannot happen.

`facade::provisional_aftermath` on the host facade is snapshotted, but its name
says what it is: the aftermath and recovery vocabulary is still settling.

---

## 3. Application API

### 3.1 `worth-query-decl`: declare meaning

```rust
use worth_query_decl::facade::{application_capability, application_query, application_schema};
```

Declarations describe portable application intent. They do not install a
runtime, authenticate a caller, authorize a request, execute a provider, or
publish a result. Those transitions belong to the host.

| Module on `worth_query_decl::facade` | What you declare there |
|---|---|
| `application_schema` | Entities, relations, aspects, fields: the shape of the application graph |
| `application_operation` | Typed operations (mutations) and their declared ceilings |
| `application_query` | Typed queries (reads), with result and work ceilings |
| `application_capability` | Capabilities an operation or query needs |
| `application_program` | Programs: a named, versioned set of declarations that can be adopted on a branch |
| `application_aftermath` | Declared aftermath for effects that must be recovered or compensated |
| `authentication`, `identity`, `identity_authority`, `portable_identity` | Principals and how external identities map onto them |
| `binding`, `typed` | Typed bindings between your Rust types and installed meaning |
| `branch` | Branch-related declarations |
| `collection`, `schema_view`, `view_declaration` | Collections and declared views over the schema |
| `canonicalization`, `validation`, `diagnostics` | Canonical form, validation results, diagnostics |
| `authoring` | Authoring helpers |
| The `worth_query_*` macros and `CanonicalQueryArtifact` | Macro-based declaration and the canonical query artifact |

Guide: [`worth-query-decl` README](../workspaces/worth-query/crates/worth-query-decl/README.md).

### 3.2 `worth-query-host`: install and run

```rust
use worth_query_host::facade::{admission, domain, primary_graph, publication, runtime};
```

The host facade exposes the production Query authority graph. It does not
expose Query implementation modules, certification-only replay, or raw
lower-runtime internals.

| Module on `worth_query_host::facade` | What it gives you |
|---|---|
| `application_installation` | Build and install a validated program, for example `in_memory_program` |
| `application_entry` | The request API: `application.request(&principal, &scope)` via `WorthQueryApplicationRequestExt`, then `.query(..)`, mutations, `.on_branch(..)`, `.programs()` |
| `application_contribution` | Contribution-composed applications: `ApplicationSchemaContribution`, `WorthQueryApplicationContribution`, handler and invariant setup |
| `application_discovery` | Discover what an installed application offers |
| `application_invariants` | Invariant factories and their execution points |
| `installed`, `domain` | Installed application and domain handles; portable domain packages |
| `admission` | Admission decisions and denials (re-exported from the admission owner) |
| `runtime` | Runtime handles and limits |
| `primary_graph` | Typed access to the primary application graph inside handlers |
| `product` | Product branches: the published state an application reads and writes |
| `publication` | Published results, disclosure, and publication outcomes |
| `convergence_epoch` | Convergence epochs across owners |
| `provisional_aftermath` | Aftermath and recovery handles for effects that did not settle cleanly |
| `declaration` | The declaration types a host needs, without a separate decl import |

The two request shapes you will use most:

```rust
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;

let request = application.request(&external_principal, &request_scope);

// A read. Every execute() is a fresh attempt against the current product branch.
let published = request
    .query(SomeQuery::new(input))
    .limits(maximum_results, maximum_work)   // optional; may only narrow
    .execute()?;                             // Result<Published result, Query denial>
let rows = published.rows();
let sources = published.observed_sources();

// A mutation. The outcome is a type you must match; nothing is implied.
match request.mutate(SomeOperation::new(input)).idempotency(&key).execute()? {
    Committed { receipt, result } => { /* new state is published */ }
    AlreadyCommitted(receipt)     => { /* this key already committed */ }
    IdempotencyIntentDrift        => { /* same key, different intent */ }
    DomainDenied(denial)          => { /* your handler refused */ }
    Cancelled | DeadlineExceeded  => { /* stopped before commit */ }
    Commit(uncommitted)           => { /* not a clean landing: stale, denied, deferred,
                                        indeterminate, or moved-but-unsettled.
                                        Match the inner outcome; never blindly retry. */ }
}
```

The names above are shortened. The outcome type is
`WorthQueryApplicationMutationOutcome<Denial, Result>`. Mutation requests also
take `expect_source`, `without_source`, and `preconditions`. How WORTH Works
[§9](how-it-works.md#9-query-the-life-of-one-request) and
[§11](how-it-works.md#11-outcomes-every-way-a-request-can-end) explain what each outcome
means, and why `Commit(..)` is never collapsed into an error string.

Guide: [`worth-query-host` README](../workspaces/worth-query/crates/worth-query-host/README.md).
It covers contribution-composed applications, branch-local program evolution,
output demand and live reads, typed handler access, the ordinary typed query
entry, installed domain use, and application readiness.

### 3.3 Runnable examples

Each of these is a complete application that uses only the two facades:

```bash
cargo run --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --example ordinary_product_workflow
```

```bash
cargo run --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --example authored_workflow
```

```bash
cargo run --manifest-path workspaces/worth-query/Cargo.toml -p worth-query-certification --example advanced_product_branching
```

| Example | What it shows |
|---|---|
| [`ordinary_product_workflow`](../workspaces/worth-query/crates/worth-query-certification/examples/ordinary_product_workflow.rs) | Install a validated program, commit a typed mutation, read the successor, close conditional resources |
| [`authored_workflow`](../workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/main.rs) | Author, publish, and discover a workflow; reject, revise, approve, and apply; refuse a start from a superseded revision |
| [`advanced_product_branching`](../workspaces/worth-query/crates/worth-query-certification/examples/advanced_product_branching.rs) | Product branches and branch-local work |

The largest example is the **Bank reference world**
([contract](../workspaces/worth-query-bank-world/docs/public-consumer-contract.md)).
It is a complete domain with a server, an HTTP adapter, per-user client
processes, and a fault-injecting external service. It consumes Query only
through `worth-query-decl` and `worth-query-host`. Banking is incidental: it
was chosen because it has money that must not be lost, which makes it a good
test of the guarantees.

### 3.4 Deeper Query guides

| Topic | Guide |
|---|---|
| The ordinary application path | [ordinary-application-front-door.md](../workspaces/worth-query/crates/worth-query/docs/foundations/ordinary-application-front-door.md) |
| Branches and previews | [branches-and-previews.md](../workspaces/worth-query/crates/worth-query/docs/foundations/branches-and-previews.md) |
| What Query will never do | [hard-prohibitions.md](../workspaces/worth-query/crates/worth-query/docs/foundations/hard-prohibitions.md) |
| Domain capabilities catalog | [domain-capabilities/README.md](../workspaces/worth-query/crates/worth-query/docs/domain-capabilities/README.md) |
| Query workspace map and focused commands | [workspaces/worth-query/README.md](../workspaces/worth-query/README.md) |

Some pages under `workspaces/worth-query/crates/worth-query/docs/` are written
against the internal engine (`worth_query::facade`). When a guide shows a
`worth_query::` path, the application path is the matching module on
`worth_query_host::facade` or `worth_query_decl::facade`.

---

## 4. Certification API

| Crate | Entry point | Use |
|---|---|---|
| `worth-query-replay` | `worth_query_replay::facade` (`replay_installed_workflow`, `ScopedReplayBasis`, `WorthQueryCertificationReplayOutcome`, `WorthQueryReplayComparison`) | Freshly re-execute a retained installed workflow and compare exact effects, publication, observations, lineage, and the semantic result. The outcome is a `TransitionOutcome`: a completed replay carries a `WorthQueryReplayComparison` (`Equivalent` or `Diverged`), and anything else is a typed stop. It is not a receipt comparison. |
| `worth-query-certification` | `worth_query_certification::facade` | Reusable certification support: compare admitted providers, run Query-owned hostile scenarios. It has no construction authority. |

Replay is certification-only by law. Ordinary lanes must not import it: the
power to reconstruct an execution is not something application code should
hold. Guide:
[`worth-query-replay` README](../workspaces/worth-query/crates/worth-query-replay/README.md).

`worth-harness` (`worth_harness::facade`) is the lower-level scenario, capture,
replay, and comparison infrastructure that the runtimes use to certify
themselves. It is not a product API.

---

## 5. Schema contracts

| Crate | Entry point | Contents |
|---|---|---|
| `worth-schema-core` | `worth_schema_core::facade` | `Identity`, `IdentityName`, `Name` / `InvalidName`; the first measurement types, `Tolerance` / `InvalidTolerance` and `Unit` (length and angle units today) |
| `worth-schema-graph` | `worth_schema_graph::facade` | Graph-constitution meaning: `PromotionRequest`, `SubelementKey`, `CarryingArtifactIdentity`, `DurableReferenceKind`, `lower_graph_promotion_identity_basis` |

Both are pure: no Query, no runtime, and no dependencies. **`worth-schema-*`
crates must never import `worth-query`.** Put meaning here when it should be
reusable across applications and outlive any one runtime. Wire it into Query
from your application's entry crate. Guide:
[workspaces/worth-contracts/README.md](../workspaces/worth-contracts/README.md).

---

## 6. Standalone runtimes and hosts

Use these directly when you want one owner without the full Query stack, or
when you are building a host around it.

| Crate | Entry point | What for | Guide |
|---|---|---|---|
| `worth-signal` | `worth_signal::easy` (short path), `worth_signal::facade` (tiers: `core`, `runtime`, `branch`, `history`, `diagnostics`, `schema`, `adapters`, `integration`, `advanced`, `specialist`) | Deterministic incremental computation on its own | [README](../crates/worth-signal/README.md), [Getting started](../crates/worth-signal/docs/GETTING_STARTED.md), [API overview](../crates/worth-signal/docs/API_OVERVIEW.md) |
| `worth-signal-wasm` (npm package `worth-signals-wasm`) | The npm package `worth-signals-wasm` | Signal in the browser: worker-first state, resources, forms, routing, React integration | [README](../crates/worth-signal-wasm/README.md), [demo](../apps/WORTH-signal-demo/README.md) |
| `worth-relational` | `worth_relational::facade` | Authoritative graph-shaped truth on its own: branch-local MVCC, compare-and-publish, history, replay. A runtime that reacts to committed changes reads them through `facade::change_source`, as the Bridge does. | [README](../crates/worth-relational/README.md), [Quickstart](../crates/worth-relational/QUICKSTART.md) |
| `worth-server` | `worth_server::facade::{WorthServer, WorthServerBuilder}` | A typed server over a Query application: WORTH-native operations plus a strict HTTP compatibility boundary | [Docs](../crates/worth-server/docs/README.md) |
| `worth-ui` | `worth_ui::facade` (product path: `facade::app`, `facade::query_binding`) | Authored UI over a Query application | [AI discovery](../workspaces/worth-ui/AI_README.md), [architecture](../workspaces/worth-ui/docs/architecture.md) |
| `worth-ui-native-platform` | Crate root | The only native-display entry point | [UI README](../workspaces/worth-ui/README.md) |
| `worth-store` | Crate-root modules: `physical_runtime`, `integrity_observation`, `aspect_native`, `contracts`, `physical_format`, `terminal_projection` | The durable physical store and its runtime lifecycle | [Workspace](../workspaces/worth-store/README.md), [facade](../workspaces/worth-store/crates/worth-store/README.md) |

Two caveats:

- **Using Relational or Signal directly means you hold their contracts
  yourself.** Query is what composes truth, derived work, admission, and
  publication into one governed request. Standalone use gives you one owner's
  guarantees, not the platform's.
- **Store is not yet wired to Query.** No crate outside the Store workspace
  depends on it today. Query's durable execution parity is planned, not done.

---

## 7. Internal crates: do not import

These crates are the owners behind the facades. Their APIs change without
notice, and several of them mint authority that application code must never
hold.

| Crate | Owns | You reach it through |
|---|---|---|
| `worth-proof` | Legality as types: witnesses, phases, checked transitions | Nothing. You receive its artifacts from owners. |
| `worth-runtime-bridge` | Truth-to-computation correspondence | Query supplies it |
| `worth-runtime-world` | Product branches, coordinated publication | `worth_query_host::facade::product` |
| `worth-query` | The internal Query engine | `worth-query-decl` and `worth-query-host` |
| `worth-query-declaration` | Canonical declarations and binding grammar | `worth-query-decl` |
| `worth-query-installation` | Portable packages and installed contracts | `worth_query_host::facade::domain` |
| `worth-query-admission` | Basis, policy, resource, access-planning decisions | `worth_query_host::facade::admission` |
| `worth-query-execution` | Execution over admitted plans | `worth_query_host::facade::{primary_graph, provisional_aftermath}` |
| `worth-query-publication` | Disclosure and publication | `worth_query_host::facade::publication` |
| `worth-query-package-archive` | The release archive protocol | Release tooling only |

**`worth-foundational` is not on this list.** It is shared vocabulary
(canonical values, identities, digests, receipts), not authority, so an
application may import it; the Bank reference world does. Prefer a facade
re-export when one exists.

---

## 8. How the boundaries are enforced

The rules on this page are checked by a tool, not left to review.

`tools/boundary-check` reads the repository's machine-checked rules from
`tools/boundary-check/config/road1.toml` ("Road 1" is the name of the current
rule set) and fails on:

- **Facade drift.** An export was added to, removed from, or renamed on a
  stable facade without updating `snapshots/facades.toml`.
- **Audience violations.** Crates whose names follow
  `{tier}-{band}-{domain}` are checked by band, and the band decides what
  they may import. Crates with other names (for example `bank-*`,
  `worth-ui-*`, and `worth-server`) are not band-checked today; the
  dependency denials below still apply to them. The band rules:
  - `worth-query-decl` and `worth-query-host` may be used only by `entry`-band
    and `cert`-band crates;
  - `worth-query-replay` may be used only by `cert`-band crates;
  - so an application crate inside this repository is an `entry`-band crate
    (for example `worth-entry-<domain>`);
  - `schema` crates may depend on nothing, and `entry` crates only on `schema`,
    `resolver`, and `derived` crates.
- **Dependency denials.** An owner may not import what the constitution
  forbids. For example, only `worth-ui-query-binding` may depend on the
  `worth-query` engine in UI production code.
- **Identifier denials.** Some constructors may be named only inside their
  owning module, so no other module can mint that authority.
- **Tier direction.** Platform crates (`worth-*`) never depend on product
  crates built on top of the platform.

Run it before you finish any change that touches a manifest or a facade:

```bash
cargo run --manifest-path tools/boundary-check/Cargo.toml -- --root .
```

`tools/agent-context` generates each governed crate's `AGENT_CONTEXT.md` from
the same model. Do not hand-edit those files. Check them with:

```bash
cargo run --manifest-path tools/agent-context/Cargo.toml -- check
```

---

## 9. Reading the reference documentation (rustdoc)

The facades are the documented surface. Build rustdoc for the crates you use.
Add `--open` to open the result in a browser.

Application API. Name the packages explicitly, because the Query workspace's
default members are only `worth-query`, `worth-query-decl`, and
`worth-query-host`:

```bash
cargo doc --manifest-path workspaces/worth-query/Cargo.toml --no-deps -p worth-query-decl -p worth-query-host
```

Certification API:

```bash
cargo doc --manifest-path workspaces/worth-query/Cargo.toml --no-deps -p worth-query-replay -p worth-query-certification
```

Schema contracts:

```bash
cargo doc --manifest-path workspaces/worth-contracts/Cargo.toml --no-deps
```

Root runtimes (Signal, Relational, Server; add others with more `-p` flags):

```bash
cargo doc --no-deps -p worth-signal -p worth-relational -p worth-server
```

Store and UI:

```bash
cargo doc --manifest-path workspaces/worth-store/Cargo.toml --no-deps -p worth-store
```

```bash
cargo doc --manifest-path workspaces/worth-ui/Cargo.toml --no-deps -p worth-ui
```

Bank reference world:

```bash
cargo doc --manifest-path workspaces/worth-query-bank-world/Cargo.toml --no-deps -p bank-domain -p bank-server
```

Use `--document-private-items` only when you are maintaining the platform.

---

## 10. Choosing quickly

| You want to... | Import | Start at |
|---|---|---|
| Build an application with governed reads and writes | `worth-query-decl` + `worth-query-host` | [§3](#3-application-api), then the `ordinary_product_workflow` example |
| Add a multi-step process with human decisions | the same two | [Workflows guide](../workspaces/worth-query/crates/worth-query/docs/foundations/workflows.md), [authored_workflow](../workspaces/worth-query/crates/worth-query-certification/examples/authored_workflow/main.rs), [How WORTH Works §13](how-it-works.md#13-workflows) |
| Evolve your application's program on a branch, then adopt it | the same two | [How WORTH Works §12](how-it-works.md#12-branches-programs-and-adoption), [host README](../workspaces/worth-query/crates/worth-query-host/README.md) |
| Write pure, reusable schema meaning | `worth-schema-core` / `worth-schema-graph` | [§5](#5-schema-contracts) |
| Prove that an execution replays exactly | `worth-query-replay` (cert crates only) | [§4](#4-certification-api) |
| Serve an application over the network | `worth-server` | [Server docs](../crates/worth-server/docs/README.md) |
| Put a UI on an application | `worth-ui` | [UI AI discovery](../workspaces/worth-ui/AI_README.md) |
| Use incremental computation alone, native or in a browser | `worth-signal` / npm `worth-signals-wasm` | [Signal README](../crates/worth-signal/README.md) |

---

## Related docs

- [Philosophy](philosophy.md): why the boundaries exist.
- [How WORTH Works](how-it-works.md): what happens behind these facades.
- [Glossary](glossary.md): exact meanings of every term used here.
- [Documentation index](README.md).
