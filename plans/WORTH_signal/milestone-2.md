# worth-signal Milestone 2

> **Status:** Implemented and audit-closed
>
> **Roadmap parent:** [performance.md](./performance.md)
>
> **Related implementation surfaces:**
> - [trace.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/trace.rs)
> - [effect.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/runtime/effect.rs)
> - [artifacts.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/data/graph/diagnostics_access/artifacts.rs)
> - [recorder.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/diagnostics/runtime/recorder.rs)
> - [resolver.rs](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/crates/worth-signal/src/logic/explain/resolver.rs)
> - [performance.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/performance.md)
> - [milestone-3.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/milestone-3.md)
> - [milestone-2-field-classification.md](/C:/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth_signal/milestone-2-field-classification.md)

## Goal

Milestone 2 turns the runtime's partial hot/cold artifact split into an
explicit proof-carrying architecture with compile-time separation between:

- hot operational authority
- cold retained diagnostic authority
- cold canonical derivation
- best-effort cold recomputation
- policy-driven materialization

The implementation goal is not "make `RuntimeArtifactState` smaller." The goal
is to make the hot path structurally unable to carry fields whose primary
semantic purpose is later explanation, provenance expansion, retained reporting,
or forensic reconstruction.

The runtime must preserve:

- deterministic replay
- restore truth
- branch truth
- merge truth
- reuse truth
- diagnostic reconstructability truth

while making the hot execution lane credible for chip-simulator-grade and
geometry-kernel-grade churn.

## Why This Milestone Exists

The repo already separates some hot operational artifact facts from colder
retained artifact payloads, but the split is still too weak architecturally:

- `RuntimeArtifactState` still carries fields whose dominant consumers are cold
  explain, lineage expansion, or certification surfaces
- effect application still constructs hot and cold artifact shapes in one
  coupled path
- lineage recording still risks using hot-state mutation as a carrier for
  richer lineage semantics than the hot path should own
- read-time reconstruction still risks implicit dependence on resolver behavior
  and internal node structure instead of explicit canonical sources
- "reconstructable" is not yet separated into proof-grade derivation versus
  best-effort recomputation
- existing counters do not yet prove that the hot lane has stopped paying cold
  costs

The current design has the right instinct, but not yet the right enforcement.
Milestone 2 closes that gap.

## Adversarial Constraint

Under staged rotating-window churn, rewiring-heavy dependency replacement,
cross-identity reuse, branch restore, and partial splice pressure:

- hot writes must not allocate or clone rich cold artifacts by default
- hot node state must not carry variable-size or lineage-rich payloads whose
  primary purpose is later diagnostics
- deferred or no-retention policy must genuinely bypass cold record assembly
- replay and restore must remain deterministic from authoritative hot continuity
  truth plus canonical retained facts
- retained vs reconstructed explanation surfaces must obey a published parity
  contract

Any optimization that weakens reconstructability or turns parity into a
best-effort convention is invalid.

## Core Architecture Rule

The semantic model for Milestone 2 is:

`EvaluationEffect`
-> `(HotArtifactWrite + optional ColdArtifactIntent)`
-> `HotRuntimeArtifactState`
-> optional `ColdArtifactRecord`
-> read-time assembly

This is intentionally not:

`EvaluationEffect -> HotRuntimeArtifactState -> ColdArtifactIntent -> ...`

Cold intent must not be rediscovered later from ambient on-node hot state
through ad hoc resolver logic. Hot and cold emissions are sibling outputs of
effect interpretation. Hot commit depends only on hot truth. Cold materialation
depends only on explicit cold intent plus canonical sources.

## Authority Model

Milestone 2 depends on a strict authority model. Every artifact-side fact must
be classified into one of the following categories.

### 1. Hot Authority

These are facts required for operational correctness in the write lane and in
the direct hot consumers of committed node state.

Examples:

- output hash / compact output truth
- suppression truth
- meaningful-input-change truth
- compact changed-scope proof
- compact reuse operational basis
- continuity authority for lineage/replay/restore continuity

Rules:

- hot authority may live on the node hot artifact state
- hot authority must be sufficient for deterministic operational behavior
- hot authority must not require consulting cold retained structures to execute
  the next hot path decision

### 2. Hot Derived But Required

These are not canonical authority in the broad system sense, but they are
explicitly allowed as compact operational accelerators because later hot-path
behavior depends on them.

Examples:

- compact changed-scope proof derived from changed regions
- compact output identity token used for suppression

Rules:

- these values must be reproducible from stronger canonical facts
- they must be explicitly marked as derived accelerators
- they may live hot only if their operational consumers justify that cost

### 3. Cold Retained Authority

These are canonical cold facts whose retention preserves diagnostic truth and
enables lossless later assembly.

Examples:

- retained changed-region richness beyond the compact hot scope proof
- retained reuse certification facts when policy requires them
- retained lineage expansion facts

Rules:

- retained cold authority may be absent in policies that choose deferred or no
  retention
- if absent, any parity claim must be limited to what remains derivable from
  other canonical authority

### 4. Cold Derivable From Canonical

These are cold surfaces that can be derived exactly and deterministically from
canonical retained facts and authoritative hot continuity facts.

Examples:

- normalized explanation summary reconstructed from compact hot truth plus
  retained changed-region facts
- historical artifact records assembled from hot continuity truth plus retained
  canonical fields

Rules:

- only this class may participate in strong retained-vs-reconstructed parity
  guarantees
- derivation must not depend on ambient mutable caches, incidental ordering, or
  hidden runtime state

### 5. Cold Recomputable Best Effort

These are useful cold enrichments that can be recomputed heuristically or
conveniently, but are not guaranteed by the proof chain to be parity-safe.

Examples:

- optional formatting-oriented summaries
- convenience rankings or derived explain ordering not carried by canonical
  retention

Rules:

- this class must never be described as parity-equivalent to retained truth
- this class must not be required for replay, restore, or authoritative
  explanation semantics

## Lineage Authority Statement

Milestone 2 requires an explicit lineage authority split.

### Hot lane owns continuity authority

The hot lane owns only the continuity facts required for:

- deterministic replay continuity
- branch restore continuity
- artifact transition continuity
- merge-safe lineage continuity semantics

These facts must be sufficient for deterministic replay and restore even if no
expanded lineage artifacts are eagerly retained.

### Cold lane owns expansion authority

The cold lane owns:

- richer lineage expansion
- investigative records
- retained history detail
- provenance-oriented contextualization

These records may add detail, but they may not alter continuity semantics.

### Consequence

If hot continuity truth and cold expanded lineage ever appear inconsistent:

- hot continuity truth is authoritative for replay/restore semantics
- cold expansion must be treated as wrong, stale, or incompletely retained
- repair must happen by reconstructing cold expansion from authoritative hot and
  canonical retained facts, not by reinterpreting hot continuity

This rule is mandatory. Otherwise lineage becomes dual-sourced and fragile.

## Semantic Parity Contract

Milestone 2 must not use an undefined phrase like "identical semantic records."
Instead it defines an explicit parity contract.

Add a named parity surface:

```rust
pub struct SemanticArtifactParity { /* private fields */ }
```

The parity contract should define:

- which assembled read-time surfaces participate
- which fields require exact equality
- which fields require normalized equality
- which fields may be absent under lower-retention policy
- which fields are excluded because they are locators or storage identities
- whether ordering is canonicalized before equality

### Required parity classes

1. `HistoricalArtifactRecord` parity
2. `TraceSummary` parity
3. explanation artifact parity
4. provenance artifact parity

### Equality rules

The spec should distinguish at least:

- exact-equal
- equal-after-normalization
- excluded-from-parity
- absent-permitted-under-policy

No test or code comment should use the word "identical" without reference to
this contract.

## Hot-Carried Type Contract Tables

Every type allowed on the hot lane must publish a contract table in the spec and
in implementation comments where appropriate. "Compact" is not a vibe. It is a
bounded contract.

For each hot-carried type, the spec must define:

- semantic role
- authoritative yes/no
- exact consumer set
- fixed-size / bounded-size / variable-size classification
- clone budget expectation
- replay stability
- branch stability
- whether it is canonical truth or an accelerator

The initial Milestone 2 hot-type surfaces should be:

- `HotRuntimeArtifactState`
- `HotArtifactWrite`
- `ArtifactTransitionKey`
- `CompactChangedScopeProof`
- `ReuseOperationalBasis`
- `ContinuityAuthorityToken`

### `HotRuntimeArtifactState`

Semantic role:
committed on-node hot operational artifact truth used by hot runtime behavior.

Authoritative:
yes for hot operational continuity and compact operational artifact truth.

Consumers:

- suppression / downstream propagation logic
- hot invalidation locality checks
- restore/replay continuity logic
- branch mutation delta accounting
- selected planner/finalize consumers that only need compact truth

Boundedness:
must be fixed-size or tightly bounded except where explicitly approved by this
spec. No unbounded collections.

Clone budget:
clone-cheap enough that unavoidable clones are not structurally dominant. The
stronger rule is to avoid cloning it on hot paths at all.

Replay stability:
required.

Branch stability:
required for continuity-bearing fields.

Canonical or accelerator:
mixed. Some fields are authoritative hot continuity truth; some are explicit
compact accelerators.

### `HotArtifactWrite`

Semantic role:
single execution-lane write packet carrying only the hot operational fields
required to update node hot artifact state.

Authoritative:
yes for the hot write boundary.

Consumers:

- hot commit path only
- branch mutation delta accounting
- compact lineage transition emission

Boundedness:
fixed-size or tightly bounded. No rich payload vectors, labels, explain trees,
or variable-size diagnostic bundles.

Clone budget:
move-preferred. Any clone should be considered suspicious.

Replay stability:
required for continuity-bearing components.

Branch stability:
required.

Canonical or accelerator:
authoritative hot write form.

### `ArtifactTransitionKey`

Semantic role:
compact continuity-bearing transition token sufficient to tie before/after
artifact continuity and deterministic lineage/replay linkage.

Authoritative:
yes for continuity semantics.

Consumers:

- lineage transition recording
- replay/restore continuity
- branch artifact transition accounting

Boundedness:
fixed-size.

Clone budget:
trivially cheap.

Replay stability:
required.

Branch stability:
required.

Canonical or accelerator:
canonical hot continuity authority.

### `CompactChangedScopeProof`

Semantic role:
compact proof of partition/scope locality needed for hot suppression and
scope-aware invalidation behavior.

Authoritative:
no. It is a hot derived accelerator backed by stronger canonical region truth.

Consumers:

- scope-aware suppression
- partition-touch checks
- hot invalidation breadth logic

Boundedness:
bounded-size only. If an operation can exceed the approved bound, the type
should degrade to a summarized or overflow-classified representation rather than
silently becoming an unbounded hot vector.

Clone budget:
cheap.

Replay stability:
required if it participates in replay-visible decisions.

Branch stability:
required if used by branch restore-visible behavior.

Canonical or accelerator:
accelerator only.

### `ReuseOperationalBasis`

Semantic role:
compact operational truth sufficient for deterministic hot-lane reuse and
suppression semantics without consulting cold certification artifacts.

Authoritative:
yes for operational reuse behavior; no for full reuse explanation richness.

Consumers:

- suppression/reuse decisions
- hot result continuity logic
- branch mutation delta accounting when reuse continuity matters

Boundedness:
fixed-size or tightly bounded.

Clone budget:
cheap.

Replay stability:
required.

Branch stability:
required.

Canonical or accelerator:
canonical for operational reuse semantics; not canonical for certification
richness.

### `ContinuityAuthorityToken`

Semantic role:
compact token representing continuity truth needed for replay/restore and
artifact continuity semantics.

Authoritative:
yes.

Consumers:

- replay
- restore
- lineage continuity
- merge-sensitive continuity checks

Boundedness:
fixed-size.

Clone budget:
trivial.

Replay stability:
required.

Branch stability:
required.

Canonical or accelerator:
canonical hot continuity authority.

## Concrete Rust Surface

Milestone 2 should be implemented as a small set of explicit Rust-facing
surfaces rather than as an informal series of field deletions.

Primary types:

```rust
pub struct HotRuntimeArtifactState { /* private fields */ }
pub struct HotArtifactWrite { /* private fields */ }

pub struct ArtifactTransitionKey { /* private fields */ }
pub struct ContinuityAuthorityToken { /* private fields */ }
pub struct CompactChangedScopeProof { /* private fields */ }
pub struct ReuseOperationalBasis { /* private fields */ }

pub struct ColdArtifactIntent { /* private fields */ }
pub struct ColdArtifactRecord { /* private fields */ }

pub struct RetainedExplanationFacts { /* private fields */ }
pub struct RetainedProvenanceFacts { /* private fields */ }
pub struct RetainedLineageExpansionFacts { /* private fields */ }
pub struct RetainedReuseCertificationFacts { /* private fields */ }

pub struct SemanticArtifactParity { /* private fields */ }
```

Primary policy surface:

```rust
pub enum ArtifactRetentionMode {
    None,
    Deferred,
    Retained,
}

pub struct ArtifactRetentionPolicy {
    pub mode: ArtifactRetentionMode,
    pub retain_explanation_facts: bool,
    pub retain_provenance_facts: bool,
    pub retain_lineage_expansion_facts: bool,
    pub retain_reuse_certification_facts: bool,
    pub retain_region_richness: bool,
}
```

The coarse enum may remain the outer control plane for Milestone 2, but policy
must be structured so the runtime is not painted into a corner. Distinct cold
domains are real and must be representable.

Primary hot/cold builders:

```rust
impl SignalGraph {
    pub(crate) fn build_hot_artifact_write(
        &self,
        effect: &EvaluationEffect,
        comparison: EffectComparison,
    ) -> Result<Option<HotArtifactWrite>, SignalError>;

    pub(crate) fn build_cold_artifact_intent(
        &self,
        effect: &EvaluationEffect,
        comparison: EffectComparison,
        policy: &ArtifactRetentionPolicy,
    ) -> Result<Option<ColdArtifactIntent>, SignalError>;

    pub(crate) fn commit_hot_artifact_write(
        &mut self,
        node: NodeId,
        write: HotArtifactWrite,
    ) -> Result<HotArtifactCommitReport, SignalError>;

    pub(crate) fn materialize_cold_artifact_record(
        &mut self,
        node: NodeId,
        intent: ColdArtifactIntent,
    ) -> Result<Option<ColdArtifactRecord>, SignalError>;
}
```

Primary read-time assembly facades:

```rust
pub fn assemble_historical_artifact_record(
    node: NodeId,
    hot: Option<&HotRuntimeArtifactState>,
    cold: Option<&ColdArtifactRecord>,
    causality: Option<&CausalityMetadata>,
) -> Option<HistoricalArtifactRecord>;

pub fn assemble_trace_summary(
    hot: Option<&HotRuntimeArtifactState>,
    cold: Option<&ColdArtifactRecord>,
) -> Option<TraceSummary>;
```

## `ColdArtifactIntent` Contract

This is the most important new cold-side boundary and it must be defined
precisely.

`ColdArtifactIntent` is:

- a bounded execution-lane emission
- containing only irrecoverable canonical cold seeds
- or facts that would be disproportionately expensive or impossible to recover
  later from hot state alone
- but still materially smaller and colder than a fully assembled retained
  record

It is not:

- a partially materialized explanation artifact
- a rich provenance graph
- a bag of already-expanded labels and strings that happen to be deferred
- a generic "maybe useful later" transport object

### What `ColdArtifactIntent` may contain

- compact locators for later cold assembly
- canonical retained seeds that are not preserved in hot state
- explicit retention-domain decisions resolved by policy
- stable references needed for later deterministic cold assembly

### What `ColdArtifactIntent` must not contain

- prose-like explanations
- fully expanded provenance structures
- cloned large region payloads where compact canonical seeds suffice
- ad hoc resolver-only conveniences
- data whose only justification is "we already had it around here"

### Architectural rule

If a field proposed for `ColdArtifactIntent` could be recovered exactly and
cheaply from canonical hot plus retained facts, it should not live in
`ColdArtifactIntent`.

If a field proposed for `ColdArtifactIntent` is not required for canonical cold
assembly and only serves current resolver convenience, it should not live in
`ColdArtifactIntent`.

## Field Classification Requirements

Before any major code refactor, every current field on the affected surfaces
must be classified.

Required current surfaces:

- `RuntimeArtifactState`
- `RetainedDiagnosticArtifact`
- lineage-related facts stamped in `recorder.rs`
- explanation/provenance assembly inputs consumed in `resolver.rs`
- any compact artifact deltas or branch mutation artifact summaries

For each field, record:

- current owner type
- semantic role
- current consumers
- classification:
  - `HotAuthority`
  - `HotDerivedButRequired`
  - `ColdRetainedAuthority`
  - `ColdDerivableFromCanonical`
  - `ColdRecomputableBestEffort`
- boundedness
- replay stability
- branch stability
- whether it may remain hot after Milestone 2

The milestone is not allowed to proceed directly to implementation without this
inventory.

## Proposed Hot/Cold Ownership Direction

The exact final shape must be driven by the field inventory, but the current
architectural expectation is:

### Strong candidates to remain hot

- deterministic output hash / normalized output change truth
- recomputed flag
- dependency count
- meaningful input change count
- changed partition count
- compact changed-scope proof
- propagation suppression truth
- compact output identity / continuity authority needed for suppression and
  continuity semantics
- compact operational reuse basis
- continuity-bearing transition key or lineage continuity token

### Strong candidates to move cold or split further

- rich changed-region payloads
- labels
- keyed family / key strings
- full reuse certification records
- rich reuse boundary context if the hot lane does not require the full object
- execution/semantic segment provenance if only cold consumers use it
- lineage expansion richness beyond continuity authority

### Fields requiring explicit proof before remaining hot

- `output_identity`
- `continuity_token`
- `reuse_boundary_context`
- `execution_record_id`
- `semantic_segment_id`
- `lineage_artifact_id`
- `merge_authority`

No field in this class may remain hot by default just because it is already
there. Each one must justify its hot residency under the hot-type contract
rules.

## Resolver Restrictions

`resolver.rs` is a primary architectural risk surface and Milestone 2 must
constrain it explicitly.

### Resolver is not allowed to:

- inspect arbitrary node internals outside published hot/cold facades
- recover missing semantics through ad hoc fallback to unrelated node state
- depend on implicit ordering from runtime containers that is not part of the
  canonical assembly contract
- mix retained and live ambient state in ways that make reconstruction policy-
  dependent or timing-dependent

### Resolver is required to:

- consume only explicit canonical sources
- use facade-level assembly functions for hot/cold artifact composition
- distinguish proof-grade derivation from best-effort recomputation
- participate in the published `SemanticArtifactParity` contract

This is not a style preference. If the resolver is allowed to become a semantic
grab-bag, the hot/cold split will regress.

## Implementation Changes

### 1. Introduce explicit hot/cold artifact vocabulary

Add new artifact-domain types and stop using one mixed struct as the conceptual
carrier for all artifact-side truth.

Required new type families:

- hot authority:
  - `HotRuntimeArtifactState`
  - `HotArtifactWrite`
  - `ArtifactTransitionKey`
  - `ContinuityAuthorityToken`
  - `CompactChangedScopeProof`
  - `ReuseOperationalBasis`
- cold emission:
  - `ColdArtifactIntent`
- cold retained:
  - `ColdArtifactRecord`
  - retained domain-specific fact carriers
- parity:
  - `SemanticArtifactParity`

Semantics safeguards:

- fields remain private
- constructors remain sealed or `pub(crate)` to the owning module
- external code must not synthesize hot proof-bearing or cold canonical
  assembly seed forms directly

This change makes the invalid state "rich cold semantics leaked into hot state
by convenience" much harder to represent.

### 2. Define authority and parity contracts before refactoring hot code

Before effect-path code changes:

- publish the field classification inventory
- publish the hot-type contract tables
- publish the lineage authority statement
- publish the `SemanticArtifactParity` rules

This phase exists to prevent type names from becoming right while semantics stay
mushy.

Exit condition:

- every field on current artifact surfaces has an authority classification
- every hot-carried type has a boundedness and consumer contract
- parity means something mechanically testable

### 3. Split effect interpretation into sibling hot/cold emissions

Refactor `effect.rs` so effect interpretation produces:

- `HotArtifactWrite`
- optional `ColdArtifactIntent`

and not a coupled mixed structure.

Required hot-path rules:

- hot write construction must not require cold record construction
- deferred and none retention modes must not build `ColdArtifactRecord`
- hot commit must consume `HotArtifactWrite` only
- effect telemetry must distinguish hot write cost from cold intent or retained
  materialization cost

This is the main separation gate.

### 4. Redesign hot state around compact operational truth

Replace or narrow current `RuntimeArtifactState` into `HotRuntimeArtifactState`
with:

- only hot operational truth
- only compact continuity truth
- only compact reuse truth
- only compact locality proof

Explicit rule:

- no unbounded or convenience-rich payloads
- no fields whose dominant consumers are explain/provenance assembly unless they
  pass the hot-type contract test

This is where the current mixed artifact state becomes honest.

### 5. Move lineage transition recording onto compact continuity facts

Refactor `recorder.rs` so lineage transition recording consumes compact
continuity-bearing facts emitted by the hot commit report.

Required rules:

- hot commit report carries enough continuity authority for deterministic
  transition recording
- cold lineage expansion is optional and separately materialized
- expanded lineage may enrich, but cannot redefine continuity

This removes the tendency to smuggle cold lineage richness through hot node
state.

### 6. Redesign retained diagnostic artifacts as strict cold records

Refactor retained artifact storage into explicit cold records and retained fact
domains.

Required rules:

- retained changed-region richness is cold
- retained labels/keyed strings are cold
- retained reuse certification facts are cold
- retained lineage expansion facts are cold
- retained explanation/provenance enrichments are cold

Materialization rule:

- `Retained` policy may materialize `ColdArtifactRecord` inline
- `Deferred` policy may keep only bounded `ColdArtifactIntent` or compact
  locators
- `None` policy must genuinely bypass cold record assembly

### 7. Distinguish proof-grade derivation from best-effort recomputation

Read-time reconstruction must explicitly separate:

- `ColdDerivableFromCanonical`
- `ColdRecomputableBestEffort`

The resolver and artifact assembly surfaces must know which class they are
producing.

Required rule:

- only proof-grade derivation participates in parity assertions
- best-effort recomputation must be labeled and excluded from strong parity

### 8. Rebuild read-time assembly on canonical facades

Refactor `artifacts.rs` and `resolver.rs` to assemble:

- `HistoricalArtifactRecord`
- `TraceSummary`
- explanation artifacts
- provenance artifacts

from:

- `HotRuntimeArtifactState`
- `ColdArtifactRecord`
- explicit retained canonical sources

and not from arbitrary node internals.

Required rules:

- no hidden fallback paths
- no direct dependence on implementation details outside the facade
- normalized ordering and equality behavior defined by `SemanticArtifactParity`

### 9. Add policy-aware domain retention without hard-coding one global law

The initial coarse runtime mode may remain:

- `None`
- `Deferred`
- `Retained`

but implementation must already distinguish cold domains internally so the
system can evolve toward domain-specific retention without structural rewrite.

At minimum the design must keep separate policy toggles for:

- explanation retention
- provenance retention
- lineage expansion retention
- reuse certification retention
- changed-region richness retention

Milestone 2 does not need the full final policy matrix, but it must not make it
impossible.

### 10. Add mechanical enforcement and visibility restrictions

To enforce the architecture mechanically:

- constructors for hot proof-bearing types stay sealed
- constructors for cold retained canonical seed forms stay sealed
- hot commit functions accept only `HotArtifactWrite`
- cold materialization accepts only `ColdArtifactIntent`
- resolver facades accept only published hot/cold facade types
- internal node structures exposing mixed artifact internals should be narrowed
  or hidden

Wrong hot/cold movement should fail to compile, not merely violate convention.

## Integration Plan

### Phase 1: Field inventory and classification

Build the field inventory first.

Deliverables:

- field classification table for all current artifact and lineage surfaces
- initial proposed hot/cold ownership mapping
- hot-type contract table draft

Exit condition:

- no field remains unclassified
- no controversial hot field remains unexamined

### Phase 2: Authority and parity contracts

Build the contract layer before implementation changes.

Deliverables:

- lineage authority statement
- `SemanticArtifactParity` rules
- proof-grade derivation vs best-effort recomputation split
- explicit cold-intent semantics

Exit condition:

- parity claims are mechanically meaningful
- lineage authority is single-sourced by design

### Phase 3: Type introduction and sealed constructors

Add the new types and restrict visibility.

Deliverables:

- new hot/cold type families
- sealed constructors
- compatibility adapters where needed

Exit condition:

- new proof-bearing types exist
- old mixed artifact path is no longer the only architectural surface

### Phase 4: Hot effect-path split

Refactor effect interpretation into sibling hot/cold emissions.

Deliverables:

- `build_hot_artifact_write(...)`
- `build_cold_artifact_intent(...)`
- hot-only commit path

Exit condition:

- hot commit does not depend on cold record assembly
- cold intent is bounded and policy-aware

### Phase 5: Lineage transition compaction

Refactor recorder logic around compact continuity facts.

Deliverables:

- hot commit report carrying continuity authority
- recorder consuming compact transition facts
- cold lineage expansion moved off the hot lane

Exit condition:

- continuity and expansion are separated
- branch restore semantics stay green

### Phase 6: Cold retained artifact redesign

Refactor retained artifacts into strict cold records and domain-specific retained
fact families.

Deliverables:

- `ColdArtifactRecord`
- retained fact families
- policy-aware inline/deferred/no-retention paths

Exit condition:

- cold record assembly is structurally optional
- retained richness no longer contaminates hot state

### Phase 7: Read-time reconstruction and parity hardening

Refactor read-time assembly around canonical facades and parity rules.

Deliverables:

- facade-based assembly
- parity-aware read-time reconstruction
- best-effort recomputation explicitly marked

Exit condition:

- retained and reconstructed proof-grade surfaces satisfy parity contract
- resolver no longer depends on hidden node internals

### Phase 8: Mechanical enforcement and perf proof

Finalize enforcement and counters.

Deliverables:

- visibility restrictions
- absence tests
- counter proof tests
- before/after perf evidence

Exit condition:

- hot/cold split is mechanically enforced
- representative churn workloads show reduced hot artifact cost

## Test Plan

### Semantic certification

Add tests that prove architecture, not just behavior:

- rewiring preserves diagnostic truth under the new hot/cold split
- branch restore preserves continuity truth without requiring eager cold
  expansion
- cross-identity reuse preserves operational reuse truth while keeping rich
  certification cold
- partial splice preserves continuity and reconstructable artifact truth
- hot continuity facts are sufficient for deterministic replay/restore
- retained cold expansion never overrides hot continuity authority

### Parity certification

Add tests tied to `SemanticArtifactParity`:

- `HistoricalArtifactRecord` retained vs derived parity
- `TraceSummary` retained vs derived parity
- explanation artifact parity for proof-grade derivable surfaces
- provenance artifact parity for proof-grade derivable surfaces

Each parity test must name:

- exact fields compared
- normalization rules used
- excluded fields
- policy conditions under which absence is permitted

### Absence tests

Milestone 2 must include absence-based tests, not just result-based tests.

Required examples:

- in `Deferred`, no `ColdArtifactRecord` is constructed on the hot write path
- in `None`, no cold builder closure or cold record assembly path is invoked
- hot write path does not clone rich explain payloads
- hot write path compiles without importing cold record types where not needed
- resolver does not perform hidden fallback to unpublished node internals

These tests are mandatory because the main regression mode is accidental
re-coupling.

### Architectural visibility tests

Add compile-time or module-visibility enforcement tests that prove:

- hot proof-bearing types cannot be synthesized outside their module
- cold retained canonical seed types cannot be synthesized outside their module
- hot commit accepts only `HotArtifactWrite`
- cold materialization accepts only `ColdArtifactIntent`
- resolver must consume facade-level assembly inputs

### Counter and performance proof tests

Add exact counter assertions for:

- `hot_artifact_write_count`
- `cold_artifact_intent_count`
- `cold_artifact_retained_count`
- `cold_artifact_deferred_count`
- `lineage_transition_compact_count`
- `artifact_reconstruction_count`
- `retained_vs_reconstructed_parity_count`
- `hot_write_alloc_count`
- `hot_write_variable_bytes_estimate`
- `deferred_mode_cold_assembly_bypass_count`
- `hot_state_inline_size_bytes`
- `cold_reconstruction_time_ns`
- `eager_cold_materialization_time_ns`
- `hot_path_cold_field_access_count`

Performance acceptance must prove not only that totals improved, but that cold
cost stopped leaking into the hot lane.

### Required regression lanes

Run and keep green:

- full serial library sweep
- full parallel library sweep
- ignored perf suite with `--test-threads=1`
- rewiring diagnostics lanes
- branch restore representative lanes
- cross-identity reuse representative lanes
- partial splice representative lanes
- retained vs reconstructed parity representative lanes

## Measurement Requirements

Milestone 2 should explicitly track:

- hot write count
- hot write allocation count
- hot write variable-size byte estimate
- hot state inline size at compile time
- deferred cold assembly bypass count
- eager cold materialization time
- cold reconstruction time
- hot-path cold-field access count

These measurements exist because event counts alone do not prove separation.

## Acceptance Criteria

Milestone 2 is complete only when all of the following are true:

1. The hot execution lane updates only explicit hot operational artifact state.
2. Cold retained artifact materialization is policy-controlled and structurally
   optional.
3. `ColdArtifactIntent` is bounded and contains only irrecoverable canonical
   cold seeds or explicit cold locators.
4. Lineage continuity authority is single-sourced in the hot lane.
5. Resolver/reconstruction code consumes only published canonical facades.
6. Proof-grade retained-vs-reconstructed parity remains green under the
   published parity contract.
7. Deferred and none retention modes prove absence of cold materialization on
   the hot path.
8. Hot artifact costs fall materially on staged churn profiles.
9. No regression appears in rewiring, branch restore, cross-identity reuse, or
   partial splice truth.

## Assumptions and Defaults

- Milestone 2 is allowed to keep a coarse top-level retention mode enum if the
  underlying policy model is not boxed in.
- Some fields currently on `RuntimeArtifactState` may remain hot, but only if
  they pass the hot-type contract requirements.
- Best-effort recomputation remains allowed for convenience-oriented read
  surfaces, but it must never be confused with proof-grade derivation.
- Hot continuity truth remains authoritative even when richer retained cold
  expansion is absent.
- The resolver must be refactored toward stricter canonical-source assembly even
  if compatibility adapters exist temporarily during migration.
- The milestone is not done when the structs merely look cleaner. It is done
  when the hot lane can no longer accidentally pay for cold semantics by
  default.
