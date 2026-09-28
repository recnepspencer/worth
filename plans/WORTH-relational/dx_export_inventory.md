# WORTH Relational DX Export Inventory

## Purpose

This is the strict public-surface inventory for `worth-relational`.

Not the academic version.

This doc exists so we can answer, in plain English:

- what is actually public today
- what should anchor the public story
- what is advanced but still legit
- what is specialist stuff
- what is cluttering the public story

The standard here is architectural, not paternalistic.

We are not classifying surfaces by whether they are "for dumb users."

We are classifying them by:

- whether they are foundational to the product identity
- whether they are raw or guided
- whether they should sit on the main path or a contained path
- whether they are architecturally real versus support-only leakage

Complicated surfaces are not a failure.

The failure mode is exposing complicated surfaces in a way that makes the crate
feel random, hostile, or harder to understand than it really is.

This is an inventory, not a promise that all of it should stay public.

---

## Live Code Scan Snapshot

This inventory is anchored to the live code in
[`facade.rs`](../../crates/worth-relational/src/facade.rs),
not just older docs.

The full symbol-level reference now lives in
[`dx_export_exhaustive_audit.md`](./dx_export_exhaustive_audit.md).

Current public facade module count:

- `21`

Current direct re-export counts by module:

| Module | Export Count |
| --- | ---: |
| `config` | 26 |
| `commit_strategies` | 70 |
| `diagnostics` | 10 |
| `durability` | 28 |
| `errors` | 5 |
| `history` | 30 |
| `identity` | 12 |
| `inspection` | 44 |
| `indexes` | 10 |
| `lineage` | 37 |
| `merge` | 92 |
| `runtime` | 44 |
| `payloads` | 5 |
| `publication` | 26 |
| `query` | 25 |
| `replay` | 20 |
| `schema` | 94 |
| `snapshots` | 4 |
| `storage` | 1 |
| `symbols` | 5 |
| `transactions` | 65 |

That count snapshot matters because it shows where the real surface pressure is:

- `schema`, `merge`, `commit_strategies`, and `transactions` are not just
  conceptually broad, they are literally broad
- `runtime` is central but also fairly wide
- `identity`, `payloads`, `errors`, `snapshots`, `storage`, and `symbols` are
  much easier to reason about directly

This doc is still not yet a fully exhaustive symbol-by-symbol judgment pass.

It is now grounded in the real facade export surface instead of relying on
outdated documentation summaries.

---

## Deep Audit Findings

After walking the backing `mod.rs`, `facade.rs`, `data`, and `logic` files, a
few things are now clearer.

### 1. The Crate Boundary Is Still Real

From [`lib.rs`](../../crates/worth-relational/src/lib.rs),
the only root public module is:

- `facade`

That matters.

Some backing modules have lots of internal `pub` types and public-looking
subtrees, but they do not automatically leak outside the crate. The real
external contract is still the top-level facade re-export set.

So the DX problem is not "too many root entrypoints."

The DX problem is:

- the facade is wide
- some facade modules collapse multiple architectural jobs
- some public names are export buckets instead of clear product lanes

### 2. Some Facade Areas Are Pure Data Re-Exports

These are relatively straightforward:

- `config`
- `diagnostics`
- `durability`
- `errors`
- `history`
- `identity`
- `payloads`
- `query`
- `replay`
- `schema`
- `snapshots`
- `symbols`

That does not automatically mean they are good.

It means the cleanup there is mostly about:

- naming
- grouping
- boundary shape

Not "mystery logic leaked through by accident."

### 3. Some Facade Areas Are Access/Authority Surfaces In Disguise

The deeper audit makes this much more explicit.

These modules are really facade doors into access/authority objects and method
surfaces:

- `inspection`
- `indexes`
- `publication`
- `merge`
- `transactions`

And to a lesser extent:

- `history`
- `replay`

That matters because type counts alone understate how much API weight is really
there.

### 4. Access/Authority Method Surface Counts Matter

Direct method counts from the backing access/authority facades:

| Surface | Public Methods |
| --- | ---: |
| `HistoryAccess` | 15 |
| `HistoryAuthority` | 4 |
| `IndexAccess` | 4 |
| `IndexAuthority` | 3 |
| `PublicationAccess` | 9 |
| `ReplayAccess` | 3 |
| `ReplayAuthority` | 3 |
| `StorageAccess` | 10 |
| `MergeAccess` | 3 |
| `RelationalTransaction` | 5 |
| `CommitStrategiesFacade` + authority | 6 |

That tells us:

- `history` is heavier than it first looks
- `publication` is a real operator/programmatic surface, not just a bag of data
  types
- `transactions` and `commit_strategies` are already drifting toward guided
  workflow facades, which is good
- `merge` is currently small in method count but huge in data vocabulary

### 5. The Biggest Public-Surface Risk Is Mixed Jobs, Not Hidden Leakage

The deeper pass did not reveal a secret second public boundary.

What it revealed is a more specific problem:

- one public module often stands in for several different architectural jobs

Examples:

- `schema` = authoring + integrity + continuity + reconciliation + transition
- `transactions` = user intent + bulk planning + trace/report artifacts
- `publication` = diagnostics + patch streams + subscriber recovery + bundle
  lifecycle
- `history` = plain branch history + aspect history + merge substrate helpers
- `merge` = planning + policy + identity + conflict classification + execution
  artifacts

That is what the next docs need to solve.

### 6. `harness` Still Stands Out As The Only Easy Removal

The deep pass made the other modules feel more legitimate, not less.

`harness` is still the main exception.

It was the clearest example of support-oriented surface that did not earn its
place in the public facade story.

---

## Boundary Legend

- `Core`
  - foundational public surface that should define the product memory shape
- `Advanced`
  - real API that should stay public, but should not crowd the main path
- `Specialist`
  - architecturally real power surface that should be exposed deliberately and
    guided well
- `Leak`
  - public right now, but probably not something we want shaping the product

These labels do not mean "easy" versus "hard."

They mean:

- what should define the main facade story
- what should be reached through a guided escalation path
- what is structurally real but should be contained instead of sprayed across
  the first impression
- what is support leakage rather than product API

---

## Root Public Entry

From [`lib.rs`](../../crates/worth-relational/src/lib.rs):

- `worth_relational::facade`

That is good news.

Unlike Signal, Relational is not already split across several root public entry
stories. The cleanup problem is inside the facade, not at the crate root.

Also relevant:

- [`RelationalRuntimeApi`](../../crates/worth-relational/src/presentation/api.rs)
  - `builder()`

Quick take:

- `facade` is the real public boundary
- `RelationalRuntimeApi` is the quick-start helper
- that matches the repo rule that the facade is the public surface and internal
  structure should not become the contract

---

## Big Picture

Current facade modules from [`facade.rs`](../../crates/worth-relational/src/facade.rs):

- `config`
- `commit_strategies`
- `diagnostics`
- `durability`
- `errors`
- `history`
- `identity`
- `inspection`
- `indexes`
- `lineage`
- `merge`
- `runtime`
- `payloads`
- `publication`
- `query`
- `replay`
- `schema`
- `snapshots`
- `storage`
- `symbols`
- `transactions`

Quick take:

- this is already a serious product
- this is also too many equally-loud doors
- the main DX job is not "add more API"
- the main DX job is to make the facade honest about what is primary, what is
  contained, and what is leaking

---

## Entry Surfaces

## `facade::runtime`

Classification:

- `Core`

What it is:

- the heart of the runtime story
- builder, runtime object, read views, projections, invariants, execution model

Most important exports:

- `RelationalRuntimeBuilder`
- `RelationalRuntime`
- `RelationalRuntimeApi`
- `RelationalReadView`
- `RelationalRuntimeConfig`
- `RelationalExecutionModel`

What feels good:

- this is the obvious center of gravity
- if someone asks "where do I start?" the answer should probably live here

What feels messy:

- some runtime exports are guided-product material
- some are policy/control material
- some are basically certification/runtime-contract material
- they are all mixed together today

Pressure:

- this module needs to stay central
- it also needs thinning and grouping so the facade teaches the right runtime
  architecture instead of reflecting raw export accumulation

## `facade::transactions`

Classification:

- `Core`

What it is:

- how truth actually changes
- intents, batches, commits, rollback, summaries, conflict results

Most important exports:

- `RelationalTransaction`
- `MutationIntent`
- `CreateIntent`
- `UpdateEntityIntent`
- `DeleteEntityIntent`
- `DeleteRelationIntent`
- `WorkerIntentBatch`
- `CommitResult`
- `CommitSummary`
- `RollbackOutcome`
- `TransactionOptions`

What feels good:

- this is a real product surface, not side noise
- the names are mostly grounded in real jobs

What feels messy:

- there are a lot of artifact and trace types mixed into the same namespace
- commit summaries, aspect traces, merge commit artifacts, and planning-heavy
  stuff all sit near the main mutation path

Pressure:

- this should become one super-clear mutation story
- the trace-heavy and specialist stuff should remain available through contained
  surfaces or guided escalation, not mixed flatly into the main mutation story

## `facade::query`

Classification:

- `Core`

What it is:

- bulk query and traversal planning/execution surface

Most important exports:

- `CanonicalQueryResult`
- `QueryExecutionOutcome`
- `QueryExecutionShape`
- `QueryScope`
- `QueryAccessPath`

What feels good:

- query is a real product feature for this runtime
- it belongs in the first-class story

What feels messy:

- a lot of the exposed types sound like internal execution planning
- `DeterministicQueryFragmentKey`, `QueryPlanContextId`, and
  `QueryWorkerFragment` should stay available without being mistaken for the
  main query mental model

Pressure:

- keep query public and important
- keep the real planning internals available
- contain them so the main query story is not forced to start at the lowest
  execution vocabulary

## `facade::identity`

Classification:

- `Core`

What it is:

- the runtime's base truth vocabulary for records and versions

Most important exports:

- `EntityId`
- `RelationId`
- `KindId`
- `PartitionId`
- `VersionId`
- `StructuralFingerprint`

What feels good:

- compact
- memorable
- very teachable

What feels messy:

- almost nothing here is bad

Pressure:

- this is one of the cleanest public areas

## `facade::schema`

Classification:

- `Core` leaning `Advanced`

What it is:

- where truth rules and aspect semantics get declared

Most important exports:

- `RelationalSchemaRegistry`
- `EntityKindRegistration`
- `RelationKindRegistration`
- `DeclaredAspect`
- `KindAspectDeclarations`
- `RelationIntegrityDeclarations`
- `SchemaId`
- `SchemaVersionId`

What feels good:

- schema is foundational in Relational
- it should not be hidden or treated like niche garnish

What feels messy:

- this namespace is huge
- it mixes normal registration concepts with transition/reconciliation/deep
  compatibility machinery
- `SchemaBridgeDescriptor`, `SchemaReconciliationDescriptor`, and
  `ValidatedSchemaTransition` are architecturally real, but they should not be
  the first schema story people have to learn

Pressure:

- keep schema first-class
- split schema authoring from schema evolution and reconciliation so one facade
  door is not carrying several different architectural jobs at once

## `facade::payloads`

Classification:

- `Core`

What it is:

- payload shapes and payload policy basics

Most important exports:

- `RecordPayload`
- `PayloadPolicy`
- `PayloadEncoding`

What feels good:

- small
- understandable

What feels messy:

- nothing major

Pressure:

- keep simple

## `facade::config`

Classification:

- `Core` leaning `Advanced`

What it is:

- runtime profile and policy knobs

Most important exports:

- `RelationalRuntimeProfile`
- `DurabilityPolicy`
- `PublicationConfig`
- `MvccConfig`
- `CompiledLanePolicy`
- `RuntimeProfileBoundaryPolicy`

What feels good:

- profile-based setup is the right story
- the underlying config model is structurally strong
- config is broken into real sections:
  - `ExecutionConfig`
  - `DiagnosticsConfig`
  - `HistoryConfig`
  - `SchemaConfig`
  - `CommitStrategiesConfig`
  - `IdentityConfig`
  - `StorageConfig`
  - `VisibilityConfig`
  - `PublicationRuntimeConfig`
  - `DurabilityConfig`
- policy types are also explicit and named well
- `RelationalRuntimeConfig::resolved(...)` and config provenance make the system
  feel like a serious architecture instead of a pile of defaults

What feels messy:

- there are enough knobs here to overwhelm people fast
- provenance/debug-style config metadata is mixed into regular runtime setup
- the public facade does not expose the full config story in one place
- `facade::config` exports policy and profile vocabulary, but not the section
  config structs
- `RelationalRuntimeConfig` itself is exported from `facade::runtime`, not
  `facade::config`
- the nested override section structs exist in code but are not directly
  re-exported by the facade config surface
- builder coverage is strong, but it does not expose every config axis directly
  as a fluent method

Deep audit note:

Builder methods currently cover:

- runtime name
- execution model
- planning
- commit authority
- durability mode
- diagnostics profile
- schema registry
- invariant catalog
- custom invariants
- commit strategies
- commit strategy executors
- entity capacity
- relation capacity
- MVCC
- storage layout
- publication policy
- payload policy
- symbol policy
- visibility cache policy
- durable log policy
- durability policy
- durable store layout
- adjacency policy
- cross-context policy
- cascade delete policy
- compiled lane policy
- relation-integrity scope budget

Builder methods do not currently give first-class direct entry to things like:

- `history.version_graph_policy`
- `history.retention`
- `history.main_branch`
- `schema.descriptor_semantics_policy`
- `schema.descriptor_canonicalization_policy`
- `identity.symbol_table`
- direct `storage.retention`

Pressure:

- teach profiles first
- keep deep config surgery available as a deliberate escalation path
- keep config grouped by subsystem truth, in line with the repo rule that
  configuration should mirror architecture instead of collapsing into a bag of
  knobs
- unify the config story so people do not have to learn:
  - builder config in one place
  - resolved config in `runtime`
  - policy vocabulary in `config`
  as if they are three unrelated systems

---

## Operator And Power-User Surfaces

## `facade::diagnostics`

Classification:

- `Advanced`

What it is:

- diagnostic artifacts, entries, scopes, policy

Most important exports:

- `RelationalDiagnosticsFacade`
- `RelationalDiagnosticArtifact`
- `RelationalDiagnosticsEntry`
- `DiagnosticCode`

What feels good:

- diagnostics are a real moat here

What feels messy:

- this namespace is mostly artifact types, not job-oriented entry points
- it reads more like raw output vocabulary than a friendly operator surface

Pressure:

- keep strong
- productize around real operator questions while preserving the underlying
  artifact vocabulary for consumers that need it

## `facade::inspection`

Classification:

- `Advanced`

What it is:

- graph inspection, commit inspection, retention inspection, historical
  inspection, structural identity inspection

Most important exports:

- `InspectionAccess`
- `GraphInspectionSummary`
- `ConnectivityInspectionSummary`
- `CommitInspection`
- `HistoricalSnapshotView`
- `StructuralIdentityComparison`

What feels good:

- this is useful and unique

What feels messy:

- "inspection" is meaningful to us, but not automatically obvious as a product
  door
- this area may need a better product story than just "inspection exists"

Pressure:

- should probably stay public
- needs job-based framing so the facade makes observation feel phase-typed and
  intentional instead of incidental

## `facade::history`

Classification:

- `Advanced`

What it is:

- branch and commit history, aspect history, merge inspection, version graph

Most important exports:

- `BranchId`
- `CommitId`
- `BranchHead`
- `VersionGraphSnapshot`
- `AspectHistoryQueryResult`
- `MergeInspection`

What feels good:

- this is real product value

What feels messy:

- current truth, historical truth, and merge-adjacent history all blur together

Pressure:

- keep public
- separate ordinary historical access from replay and merge escalation without
  pretending they are unrelated systems

## `facade::publication`

Classification:

- `Advanced`

What it is:

- patches, CDC, subscriber state, publication bundles

Most important exports:

- `RelationalPatchRecord`
- `PatchRecord`
- `PatchStreamRequest`
- `SubscriberCheckpoint`
- `SubscriberResumeRequest`
- `PublicationBundle`

What feels good:

- publication is a core capability of the runtime

What feels messy:

- it is not the first surface most users should have to parse
- patch vocabulary, subscriber vocabulary, and publication lifecycle vocabulary
  are all mixed together

Pressure:

- keep visible
- make the publication story reachable without forcing patch-stream and
  subscriber-recovery vocabulary into the center of the facade

## `facade::snapshots`

Classification:

- `Advanced`

What it is:

- snapshot handles and read policy

Most important exports:

- `SnapshotHandle`
- `SnapshotId`
- `SnapshotReadPolicy`

What feels good:

- compact

What feels messy:

- almost nothing

Pressure:

- easy to teach as the history escalation path
- good candidate for a contained but very understandable public surface

## `facade::indexes`

Classification:

- `Advanced`

What it is:

- derived index definitions and generations

Most important exports:

- `DerivedIndexDefinition`
- `DerivedIndexBuildRequest`
- `DerivedIndexGeneration`
- `DerivedIndexPublicationStatus`

What feels good:

- important capability

What feels messy:

- not something that should sit on the main path before query and truth reads

Pressure:

- keep public, but contained
- good example of a real subsystem that should be accessible through the facade
  without being mistaken for the main product identity

---

## Specialist Surfaces

## `facade::merge`

Classification:

- `Specialist`

What it is:

- merge planning and execution universe

Quick take:

- very real
- very valuable
- way too loud for the default story

What stands out:

- this namespace is massive
- the names are highly specific and proof-ish
- people should not have to mentally parse this whole namespace just to trust
  the crate's main story

Pressure:

- keep
- contain aggressively
- eventually expose a guided merge workflow over the raw type cloud
- this is exactly the kind of hard surface the repo standards say we should
  expose better, not erase

## `facade::replay`

Classification:

- `Specialist`

What it is:

- replay requests, verification modes, mismatch surfaces, replay outcomes

Quick take:

- important
- better as a contained escalation path than a first impression

Pressure:

- keep public
- teach after history and publication
- make replay feel like a principled authority-derived surface, not an isolated
  bag of verification types

## `facade::lineage`

Classification:

- `Specialist`

What it is:

- correspondence, divergence, lineage graph, historical resolution

Quick take:

- powerful and real
- should be learned through a deliberate story, not as ambient facade noise

Pressure:

- keep public
- frame as advanced identity evolution / correspondence tooling
- keep the lineage truth explicit, because hiding it would violate the repo's
  bias toward honest, named architecture

## `facade::durability`

Classification:

- `Specialist`

What it is:

- checkpoints, recovery, compaction, durable store semantics

Quick take:

- crucial capability
- operator- and systems-heavy

Pressure:

- keep public
- should not crowd the first success path
- expose it as a real subsystem facade, not as ambient noise mixed into runtime
  setup or ordinary reads

## `facade::commit_strategies`

Classification:

- `Specialist`

What it is:

- pluggable strategy execution and reconciliation machinery

Quick take:

- this is probably one of the coolest surfaces
- this is also not the right starting point for understanding the runtime as a
  whole

What feels messy:

- giant namespace
- a lot of schema, digest, packet, and lowering language

Pressure:

- keep public
- absolutely contain from the default story
- this should probably graduate toward a better declarative story over time,
  because the repo strongly prefers declaration over scattered orchestration

---

## Small Support Surfaces

## `facade::errors`

Classification:

- `Core`

Quick take:

- good and necessary
- small enough to not be a DX problem

## `facade::symbols`

Classification:

- `Advanced`

Quick take:

- useful support vocabulary
- not something that should define the product memory shape

## `facade::storage`

Classification:

- `Advanced`

Quick take:

- tiny right now
- probably fine as contained support vocabulary

---

## Obvious Leak

## `facade::harness`

Classification:

- `Leak`

What it is:

- fixtures, harness adapters, harness plans, expectations

Quick take:

- useful for us
- not a product identity surface
- it has now left the non-test public facade and should stay that way

Why this is a stronger claim than the others:

- it fails the facade-only standard
- it exposes support and certification-oriented structure directly in the public
  boundary
- it does not clarify the runtime architecture for outside consumers

This is still the clearest support-oriented surface in the crate, but it is no
longer part of the non-test public facade.

---

## Biggest DX Pressure Points

These are the places where the public shape is most likely to violate the repo
standards while still technically "working."

### 1. Too Many Equal-Weight Doors

Right now the facade reads like:

- every subsystem is equally important

That is technically honest, but product-wise it is bad.

It also cuts against the facade rule.

A facade should expose one deliberate contract, not a flat transcript of the
codebase's current module graph.

### 2. Runtime Is The Center, But Not Yet Curated Enough

`runtime` is clearly the heart of the crate, but it still mixes:

- setup
- runtime contracts
- read projections
- invariants
- complexity machinery
- some support-observability material that still needs better grouping

That weakens the mental model the facade is supposed to protect.

### 3. Transactions Are Great, But Noisy

The mutation story is strong, but the namespace contains both:

- what people actually do
- a lot of artifact/trace/execution detail

That is the exact kind of "scattered coordination versus declarative flow"
problem the repo standards tell us to collapse.

### 4. Schema Is Foundational, But Too Wide

Schema needs to stay important, but the current namespace bundles:

- ordinary kind/aspect/contract authoring
- schema evolution
- schema reconciliation
- compatibility and bridge-ish semantics

That is too much to force into one undifferentiated public door.

It also violates the naming and boundary standard by collapsing several
different architectural jobs into one surface.

### 5. Specialist Stuff Is Real, But Not Contained Enough

The big four here are:

- `merge`
- `replay`
- `lineage`
- `commit_strategies`

They should not disappear. They should just stop competing with the daily-use
story.
They should become easier to approach through better guided exposure, not
harder to access.

That is the actual standard:

- contain
- guide
- preserve power

Not:

- flatten
- hide
- pretend the architecture is simpler than it is

### 6. Harness Is Public-Story Pollution

This used to be the cleanest easy win.

That cleanup has now happened in code:

- `facade::harness` is test-only
- `presentation::harness` is test-only

---

## Provisional Classification Summary

### `Core`

- `runtime`
- `transactions`
- `query`
- `identity`
- `schema`
- `payloads`
- `config`
- `errors`

### `Advanced`

- `diagnostics`
- `inspection`
- `history`
- `publication`
- `snapshots`
- `indexes`
- `symbols`
- `storage`

### `Specialist`

- `merge`
- `replay`
- `lineage`
- `durability`
- `commit_strategies`

### `Leak`

- `harness`

---

## Bottom Line

The good news:

- `worth-relational` already has a real public boundary
- it already has strong center-of-gravity candidates
- the API is not random

The bad news:

- the facade still feels like a map of the codebase more than a guided product
- specialist power is competing with the daily-use story
- one obvious support surface is leaking into the product boundary

Restated in repo terms:

- the facade contract is not yet curated enough
- some subsystem boundaries are exposed honestly but not accessibly
- some support structure is public without earning that exposure

That is exactly why the next docs should be:

1. export decision matrix
2. canonical surface spec
3. boundary spec

Those three docs are how we stop "cool subsystem sprawl" from becoming the
official public vibe.
