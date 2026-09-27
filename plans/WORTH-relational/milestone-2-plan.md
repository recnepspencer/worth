# Milestone 2 Plan: Canonical Aspect-Delta Engine

## Status

Milestone 2 is the next implementation milestone after
[milestone-1-closeout.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/milestone-1-closeout.md).

This document is the implementation-spec companion to
[worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
`Milestone 2: Relational Aspect Semantics`.

It is intentionally not a short roadmap summary. It exists to lock the
semantic center, type surfaces, subsystem boundaries, and acceptance shape
before implementation begins.

## Purpose

Milestone 2 is not "add aspect support across many APIs."

Milestone 2 is:

1. declare aspect semantics in schema
2. lower those declarations into an executable per-kind plan
3. compute one canonical commit-time aspect delta artifact per committed
   aspect-observable record
4. durably encode that truth in commit artifacts
5. force every downstream consumer to consume that truth rather than
   reinterpreting aspect meaning

The governing semantic flow is one-way:

```text
schema aspect declarations
  -> lowered executable aspect plan
  -> canonical commit-time record aspect delta
  -> durable commit encoding
  -> history / query / lineage consumption
  -> diagnostics and traceability derived from the same artifacts
```

If any surface in the runtime recomputes aspect meaning independently from raw
payloads, current storage state, or ad hoc historical interpretation, the
milestone is incomplete.

## Architectural Rules

These rules are normative for the implementation.

1. Aspect meaning is schema-owned truth, not runtime policy and not query-time
   convenience.
2. `LoweredAspectPlan` is the only semantic input to commit-time aspect
   evaluation.
3. `CanonicalRecordAspectDelta` is the only authoritative internal artifact for
   committed aspect truth.
4. Replay, CDC, history, lineage-aware history, query, and diagnostics are all
   downstream consumers of canonical aspect truth.
5. Diagnostics may project or explain canonical aspect truth, but may not
   compute a second aspect interpretation.
6. Lineage may contribute resolution context for historical traversal, but may
   not reinterpret record-local aspect semantics.
7. Opaque whole-payload aspect semantics are allowed only as explicitly degraded
   precision.
8. Runtime configuration may control diagnostics and trace materialization
   policy, but not aspect semantics.

## Semantic Center

### The Core Artifact

Milestone 2 introduces one new canonical internal artifact:

```rust
pub struct CanonicalRecordAspectDelta {
    pub target: RecordRef,
    pub kind_id: KindId,
    pub plan_revision: AspectPlanRevision,
    pub structural_change: RecordStructuralChange,
    pub changed_aspects: CanonicalAspectSet,
    pub evaluated_bindings: SmallVec<[EvaluatedAspectBinding; 4]>,
    pub contains_degraded_precision: bool,
}
```

This artifact is emitted exactly once per record that contributes a committed
structural or aspect-observable delta.

It is not emitted for:

- records inspected during planning only
- records inspected during legality checks only
- records inspected during invariant execution only
- records inspected during reconciliation only
- records whose final committed effect includes neither structural change nor
  aspect change

It is emitted for:

- any record with a committed structural change even if `changed_aspects` is
  empty
- any record with a committed aspect change even if the structural classifier is
  `Updated`

That emission universe is fixed. It is not a diagnostics option.

### Why This Artifact Exists

The runtime needs one internal truth product that is strong enough to drive:

- patch aspect lists
- commit aspect summaries
- replay parity
- CDC parity
- historical aspect reads
- lineage-aware historical aspect reads
- query/projection aspect filtering
- diagnostics and traceability
- certification artifacts

Without this artifact, aspect semantics would drift across surfaces.

## Type Model

### Aspect Identity

`AspectKey` remains the single public aspect identity type. Milestone 2 does not
introduce a second public type that means "basically the same aspect name."

The existing public ergonomic API surface that uses `ProjectionAspect` is
replaced with `AspectKey`.

### Schema-Owned Declaration Types

Aspect semantics live in kind registration.

```rust
pub struct KindAspectDeclarations {
    pub plan_revision: AspectPlanRevision,
    pub aspects: Vec<DeclaredAspect>,
}

pub struct DeclaredAspect {
    pub key: AspectKey,
    pub binding: AspectBinding,
    pub comparator: AspectComparator,
    pub precision: AspectPrecision,
}
```

These declarations become part of:

```rust
pub struct EntityKindRegistration {
    pub kind_id: KindId,
    pub kind_name: String,
    pub schema_id: SchemaId,
    pub schema_version_id: SchemaVersionId,
    pub aspect_declarations: KindAspectDeclarations,
}

pub struct RelationKindRegistration {
    pub kind_id: KindId,
    pub kind_name: String,
    pub schema_id: SchemaId,
    pub schema_version_id: SchemaVersionId,
    pub payload_class: RelationPayloadClass,
    pub cross_context_policy: CrossContextPolicy,
    pub cascade_delete_policy: CascadeDeletePolicy,
    pub aspect_declarations: KindAspectDeclarations,
}
```

### AspectPlanRevision

`AspectPlanRevision` is not a runtime-local counter and not a convenient label.

```rust
pub struct AspectPlanRevision(pub u128);
```

Semantics:

- it is derived deterministically from the canonicalized declaration set for one
  kind
- identical canonicalized declarations across runtimes must produce the same
  revision value
- it is valid to use for traceability and replay equivalence claims
- it is not configurable
- it is not builder-assigned

This means `AspectPlanRevision` behaves as a declaration fingerprint, not an
incrementing number.

### Binding Categories

Milestone 2 supports exactly these schema-visible binding categories:

```rust
pub enum AspectBinding {
    EntityPayloadField { field: InternedString },
    RelationPayloadField { field: InternedString },
    RelationSourceEndpoint,
    RelationTargetEndpoint,
    LifecycleTransition,
    OpaqueWholePayload,
}
```

These are not just labels. Each one maps to a fixed execution contract.

### Comparator Categories

Milestone 2 supports exactly these comparator categories:

```rust
pub enum AspectComparator {
    JsonScalarEquality,
    EndpointIdentityEquality,
    LifecycleTransitionEquality,
    OpaquePayloadByteEquality,
}
```

These are schema-visible declarations that lower into executable operators.

### Precision Categories

Precision is a truth classification, not a diagnostics flavor.

```rust
pub enum AspectPrecision {
    Structured,
    Opaque,
}
```

`Opaque` means the runtime can truthfully say whether the declared opaque aspect
changed, but cannot claim structured semantic precision comparable to field- or
endpoint-bound aspects.

### Lowered Executable Plan

Schema declarations are not consumed directly on the commit hot path.

```rust
pub struct LoweredAspectPlan {
    pub kind_id: KindId,
    pub plan_revision: AspectPlanRevision,
    pub executable_bindings: SmallVec<[LoweredAspectBinding; 8]>,
}

pub struct LoweredAspectBinding {
    pub aspect_key: AspectKey,
    pub extractor: LoweredAspectExtractor,
    pub comparator: LoweredAspectComparator,
    pub precision: AspectPrecision,
}
```

Executable operators are deliberately separated from the declaration layer:

```rust
pub enum LoweredAspectExtractor {
    EntityJsonField { field: InternedString },
    RelationJsonField { field: InternedString },
    RelationSourceEndpoint,
    RelationTargetEndpoint,
    LifecycleTransition,
    OpaqueWholePayloadBytes,
}

pub enum LoweredAspectComparator {
    JsonScalarEquality,
    EndpointIdentityEquality,
    LifecycleTransitionEquality,
    OpaquePayloadByteEquality,
}
```

Runtime ownership:

```rust
pub struct AspectPlanCatalog {
    pub entity_plans: BTreeMap<KindId, LoweredAspectPlan>,
    pub relation_plans: BTreeMap<KindId, LoweredAspectPlan>,
}
```

This catalog is built during runtime construction and is the only semantic input
to commit-time aspect evaluation.

### Canonical Aspect Set vs Requested Aspect Set

The runtime must distinguish emitted truth from caller input.

The emitted truth type is:

```rust
pub struct CanonicalAspectSet(SmallVec<[AspectKey; 4]>);
```

Construction rules:

- sorted canonically by `AspectKey`
- deduplicated at creation
- immutable after construction
- used in `CanonicalRecordAspectDelta`, patch records, history entries, replay
  comparison surfaces, and diagnostics artifacts

Caller input should not reuse the emitted truth type directly. Query-side input
uses:

```rust
pub struct RequestedAspectSet(SmallVec<[AspectKey; 4]>);
```

This is canonicalized at the boundary into a `CanonicalAspectSet` only after the
request has been validated and normalized.

### Structural Change Classification

Commit-local structural truth uses:

```rust
pub enum RecordStructuralChange {
    Created,
    Updated,
    Deleted,
    RetainedForAudit,
}
```

This enum is strictly record-local and commit-local.

It must not encode:

- replace semantics
- split semantics
- merge semantics
- correspondence semantics
- historical continuity claims

Those belong to lineage and only appear later as resolution context.

### Binding Evaluation Evidence

The canonical hot-path artifact stores minimal proof-bearing evidence, not rich
explanatory payloads.

```rust
pub struct EvaluatedAspectBinding {
    pub aspect_key: AspectKey,
    pub changed: bool,
    pub precision: AspectPrecision,
    pub evidence: BindingEvidence,
}
```

```rust
pub enum BindingEvidence {
    JsonFieldPresenceOrValue {
        old_present: bool,
        new_present: bool,
        old_canonical_hash: Option<u64>,
        new_canonical_hash: Option<u64>,
    },
    EndpointIdentity {
        old: Option<EntityId>,
        new: Option<EntityId>,
    },
    Lifecycle {
        transition: LifecycleTransitionClass,
    },
    OpaquePayload {
        old_hash: Option<u128>,
        new_hash: Option<u128>,
    },
}
```

The design rule is:

**`EvaluatedAspectBinding` is the maximum hot-path evidence shape for v1. No
additional explanatory payload may be added without proving it is required at
the canonical artifact boundary.**

### Lifecycle Transition Evidence

Commit-local lifecycle truth uses:

```rust
pub enum LifecycleTransitionClass {
    None,
    Create,
    Update,
    Delete,
    RetainForAudit,
}
```

This classifier is permitted to describe only record-local committed state
transition. It must not be used as a continuity bridge into lineage semantics.

## Exact Evaluation Semantics

### Payload Field Bindings

For `EntityPayloadField` and `RelationPayloadField`:

- the binding participates if the declared field exists in either old or new
  state
- the extracted value is canonicalized according to the binding's lowered
  comparator contract
- the aspect changes iff canonicalized old value and canonicalized new value are
  not equal

Milestone 2 does not permit arbitrary host-defined comparator callbacks.

### Endpoint Bindings

For `RelationSourceEndpoint` and `RelationTargetEndpoint`:

- the aspect changes iff the corresponding endpoint identity changes
- endpoint bindings are valid only for relation kinds
- endpoint semantics are authoritative because source and target are already
  part of authoritative relation truth in this runtime

### Lifecycle Binding

For `LifecycleTransition`:

- the aspect changes according to the record-local `LifecycleTransitionClass`
- this binding is evaluated from commit-local state transition only
- lineage events may not alter the result of this binding

### Opaque Whole-Payload Binding

For `OpaqueWholePayload`:

- precision is always `AspectPrecision::Opaque`
- the comparator must be `OpaquePayloadByteEquality`
- equality is defined as equality of canonical durable payload bytes

This is the only allowed v1 semantics for opaque aspect bindings.

It is deliberately a degraded semantic tier and must never be represented as
equal-fidelity structured aspect truth.

## Validation Rules

Runtime construction must reject:

- duplicate `AspectKey` in one kind declaration set
- empty field names for payload-field bindings
- endpoint bindings on entity kinds
- relation payload-field bindings on `TopologyOnlyRelation`
- comparator/binding combinations that do not make semantic sense
- opaque whole-payload declarations when canonical durable bytes cannot be
  produced for that payload contract

Canonical-order behavior must be explicit and singular:

- non-canonical declaration order is accepted
- declarations are canonicalized during runtime construction
- `AspectPlanRevision` is derived from canonicalized declarations
- non-canonical input order is therefore not an error

There is no strict-vs-permissive semantic mode in Milestone 2.

## Runtime Data Flow

### Build-Time Lowering

During `RelationalRuntimeBuilder::build()`:

1. validate all aspect declarations in the schema registry
2. canonicalize declaration ordering
3. derive `AspectPlanRevision` from canonicalized declarations
4. lower each declaration set into `LoweredAspectPlan`
5. install the plans into `AspectPlanCatalog`

No commit path may execute aspect evaluation against raw schema declarations.

### Commit-Time Canonical Delta Computation

During authoritative mutation application, after old and new authoritative
record state are both available and before patch emission:

1. resolve `kind_id`
2. fetch the `LoweredAspectPlan`
3. derive `RecordStructuralChange`
4. evaluate each lowered binding against `(old_state, new_state)`
5. create `EvaluatedAspectBinding` rows
6. build `CanonicalAspectSet` from bindings with `changed = true`
7. set `contains_degraded_precision` iff any changed binding has `Opaque`
   precision
8. emit `CanonicalRecordAspectDelta` only if the record contributes committed
   structural or aspect-observable delta

At this point, aspect truth is fixed for the commit.

No downstream surface may re-open aspect meaning by rescanning payloads or
re-deriving binding semantics.

## Durable Encoding

The durable obligation for Milestone 2 is:

**canonical commit artifacts must encode changed-aspect identity, structural
classification, and degraded-precision status in a form sufficient for history,
replay, CDC, and diagnostics to recover committed aspect semantics without
payload rescans.**

`PatchRecord` becomes:

```rust
pub struct PatchRecord {
    pub kind: PatchRecordKind,
    pub target: RecordRef,
    pub structural_change: RecordStructuralChange,
    pub aspects: CanonicalAspectSet,
    pub contains_degraded_precision: bool,
    pub detail: PatchDetail,
}
```

`PatchDetail` remains a publication/detail surface. Aspect truth must not depend
on reparsing it.

## Historical and Lineage-Aware Surfaces

### Record-Local Historical Aspect Reads

Record-local history uses:

```rust
pub struct AspectHistoryEntry {
    pub origin: AspectHistoryOrigin,
    pub resolution: AspectResolutionContext,
}

pub struct AspectHistoryOrigin {
    pub commit_id: CommitId,
    pub version_id: VersionId,
    pub branch_id: BranchId,
    pub target: RecordRef,
    pub structural_change: RecordStructuralChange,
    pub changed_aspects: CanonicalAspectSet,
    pub contains_degraded_precision: bool,
}
```

For direct record-local history, resolution is:

```rust
pub enum AspectResolutionContext {
    DirectRecordHistory,
    ResolvedViaLineage {
        start_lineage_id: LineageId,
        traversed_event_ids: SmallVec<[u64; 4]>,
    },
}
```

This split is mandatory. It prevents lineage traversal from masquerading as
record-local mutation history.

### Allowed Use of Authoritative Storage

History and query surfaces are allowed to combine committed aspect truth with
authoritative identity/kind/topology state needed for target selection and
scoping.

They are not allowed to combine committed aspect truth with:

- payload rescans
- fresh aspect recomputation
- current-state reinterpretation of old committed aspect meaning

### Lineage-Aware Aspect History

Lineage-aware traversal may:

- select which committed origin entries are relevant under lineage traversal
- attach `ResolvedViaLineage` context

Lineage-aware traversal may not:

- change `changed_aspects`
- change `RecordStructuralChange`
- alter degraded-precision status
- reinterpret record-local lifecycle classification

## Query and Projection Consumption

Aspect-aware query and projection are downstream consumers only.

Projection trait declarations change from `ProjectionAspect` to `AspectKey`.

Requested filters use:

```rust
pub enum AspectFilterMode {
    Any,
    All,
}

pub struct AspectFilter {
    pub mode: AspectFilterMode,
    pub aspects: RequestedAspectSet,
}
```

The boundary normalizes `RequestedAspectSet` into canonical request form before
the request enters planning or read execution.

If an aspect-aware query cannot be answered from committed aspect truth plus
authoritative record identity/kind/topology state, it is out of scope for
Milestone 2.

## Diagnostics and Traceability

### Architectural Rule

Diagnostics are mandatory but subordinate.

Diagnostics may consume:

- canonicalized declarations
- `LoweredAspectPlan`
- `CanonicalRecordAspectDelta`
- durable patch/history artifacts
- lineage resolution metadata

Diagnostics may not consume:

- raw record payloads
- mutable record state
- ad hoc recomputation helpers
- current-state scanners that derive aspect meaning independently

This must be enforced mechanically by API shape.

### Diagnostics Policy

Diagnostics materialization belongs in config because this runtime already uses
config for diagnostics capture policy.

Add:

```rust
pub struct AspectDiagnosticsPolicy {
    pub emit_declaration_traces: bool,
    pub emit_evaluation_traces: bool,
    pub emit_emission_traces: bool,
    pub emit_history_resolution_traces: bool,
    pub max_aspect_trace_rows_per_artifact: usize,
}
```

and nest it under diagnostics config:

```rust
pub struct DiagnosticsConfig {
    pub profile: RelationalDiagnosticsProfile,
    pub aspect_policy: AspectDiagnosticsPolicy,
}
```

This policy controls only whether rich trace views are materialized and how much
of them is emitted.

It does not control:

- aspect semantics
- commit-time delta computation
- durable aspect truth
- replay equivalence

### Trace View Types

Trace views are rich-path projections over canonical truth, not a second
semantic product.

```rust
pub struct AspectDeclarationTrace {
    pub kind_id: KindId,
    pub plan_revision: AspectPlanRevision,
    pub declared_aspects: Vec<DeclaredAspectTraceRow>,
}
```

```rust
pub struct AspectEvaluationTraceView {
    pub target: RecordRef,
    pub plan_revision: AspectPlanRevision,
    pub structural_change: RecordStructuralChange,
    pub binding_rows: Vec<AspectEvaluationTraceRow>,
}
```

```rust
pub struct AspectEmissionTrace {
    pub target: RecordRef,
    pub patch_position: PatchStreamPosition,
    pub changed_aspects: CanonicalAspectSet,
    pub contains_degraded_precision: bool,
}
```

```rust
pub struct AspectHistoryResolutionTrace {
    pub requested_target: HistoryAspectQueryTarget,
    pub returned_entries: usize,
    pub traversed_commits: usize,
    pub traversed_lineage_events: usize,
}
```

The materialization rule is fixed:

**canonical commit-time aspect evaluation emits only the minimal proof-bearing
artifact required for deterministic downstream derivation. Rich trace views are
materialized from that proof-bearing artifact and the lowered plan; they are not
a second commit-time semantic product.**

## Public API Changes

### Schema Facade

Expose:

- `KindAspectDeclarations`
- `DeclaredAspect`
- `AspectBinding`
- `AspectComparator`
- `AspectPrecision`
- `AspectPlanRevision`

### Publication Facade

Expose:

- `CanonicalAspectSet`
- `RecordStructuralChange`

Extend `PatchRecord` with:

- `structural_change`
- `aspects`
- `contains_degraded_precision`

### Runtime and Query Facades

Expose:

- `RequestedAspectSet`
- `AspectFilter`
- `AspectFilterMode`
- `AspectHistoryEntry`
- `AspectHistoryOrigin`
- `AspectResolutionContext`

Remove `ProjectionAspect` from the public ergonomic aspect declaration path.

## What Is Configurable and What Is Not

### Not Configurable

These are semantic contracts and must not be runtime-configurable:

- `AspectBinding`
- `AspectComparator`
- `AspectPrecision`
- `OpaquePayloadByteEquality`
- delta emission universe
- canonical ordering of emitted aspect sets
- the rule that diagnostics cannot recompute semantics
- the rule that lineage contributes resolution but not reinterpretation
- the durable encoding obligation

### Schema-Owned

These are per-kind truth declarations:

- which aspects exist
- which binding each aspect uses
- which comparator each aspect uses
- whether each aspect is structured or opaque

### Configurable

Milestone 2 introduces only one new legitimate configuration family:

- diagnostics/trace materialization policy

Any future aspect-side storage acceleration policy belongs under `StorageConfig`
only if implementation introduces a real derived storage structure that needs
layout or retention policy.

## Implementation Program

### Phase A: Schema and Lowering

Implement:

- schema declaration types
- schema validation for aspect declarations
- deterministic `AspectPlanRevision`
- canonicalization of declaration order
- lowering into `AspectPlanCatalog`

Acceptance:

- every valid aspect-declared kind produces one deterministic lowered plan
- invalid declarations fail at runtime construction

### Phase B: Canonical Delta Engine

Implement:

- `CanonicalRecordAspectDelta`
- `CanonicalAspectSet`
- `EvaluatedAspectBinding`
- `BindingEvidence`
- `RecordStructuralChange`
- `LifecycleTransitionClass`
- commit-time evaluation pipeline

Acceptance:

- exactly one canonical delta per record contributing committed structural or
  aspect-observable truth
- no-op inspected records produce no canonical delta
- opaque comparison uses canonical durable bytes only

### Phase C: Durable Encoding and Parity

Implement:

- patch encoding from canonical deltas
- commit/publication wiring from canonical deltas
- replay parity over encoded aspect truth
- CDC parity over encoded aspect truth

Acceptance:

- patch artifacts match canonical delta truth exactly
- replay detects tampered aspect-bearing artifacts
- savepoint-abandoned or rolled-back work emits no aspect residue

### Phase D: Historical and Lineage Consumption

Implement:

- record-local historical aspect reads
- lineage-aware history traversal with origin/resolution split
- history/lineage diagnostics from durable truth only

Acceptance:

- history reads do not rescan payloads
- lineage changes resolution context only

### Phase E: Query, Projection, and Closeout

Implement:

- replace `ProjectionAspect` with `AspectKey`
- aspect-aware projection/query filters
- delete legacy payload-key aspect derivation code before milestone close

Acceptance:

- no second aspect semantic system remains
- query/projection are downstream consumers only

## Test Plan

### Schema and Lowering

Add tests for:

- duplicate aspect keys rejected
- invalid binding/comparator pairs rejected
- endpoint bindings rejected on entity kinds
- relation payload-field bindings rejected on topology-only relations
- deterministic revision stability across identical declarations
- canonicalization of declaration order preserving semantic equivalence

### Commit-Time Canonical Delta

Add tests for:

- entity field-bound aspect change
- relation endpoint-bound aspect change
- lifecycle-bound aspect change
- structural change with empty `changed_aspects`
- no-op inspected record emits no canonical delta
- opaque aspect delta carries degraded-precision flag

### Durable Encoding

Add tests for:

- patch aspects equal canonical delta aspects
- patch structural classification equals canonical delta structural change
- patch degraded-precision flag equals canonical delta flag
- replay mismatch on tampered aspect set
- replay mismatch on tampered structural change
- replay mismatch on tampered degraded-precision flag

### Historical and Lineage

Add tests for:

- record-local history returns committed origin events
- lineage-aware history preserves origin semantics and only changes resolution
- lifecycle classification is not polluted by lineage continuity
- history answers from durable truth without payload rescans

### Diagnostics Boundary

Enforce by API shape and tests:

- diagnostics builders accept only lowered plans, canonical deltas, durable
  artifacts, and lineage resolution metadata
- diagnostics builders do not accept raw payloads or mutable record state

### Certification Outputs

Milestone 2 acceptance must emit machine-checkable artifacts for:

- `patch_vs_truth_delta_report`
- `aspect_tag_accuracy_report`
- `aspect_history_digest`
- `lineage_aspect_resolution_digest`
- existing `patch_digest`
- existing `diagnostics_digest`
- existing `query_surface_digest`
- existing `replay_digest`

And it must satisfy the roadmap-required named tests:

- `Diff/CDC truth parity test`
- `Bulk query and traversal stress truth test`
- `Hostile commit/replay equivalence test`
- `Topology identity survival test`
- `Netlist rewiring identity and history test`

## Completion Standard

Milestone 2 is complete only when all of the following are true:

- schema declarations own aspect meaning
- runtime lowering produces deterministic executable plans
- canonical record aspect deltas are computed exactly once per committed
  aspect-observable record
- durable artifacts preserve enough aspect truth for replay, CDC, history, and
  diagnostics without payload rescans
- lineage-aware history adds resolution context only
- diagnostics and traceability are mechanically prevented from becoming a second
  semantic system
- legacy payload-key aspect derivation is removed

At that point the runtime will have a real canonical aspect-delta engine rather
than a collection of aspect-adjacent surfaces.

## Adversarial Alignment With Test Requirements

The milestone must be interpreted through
[test-requirements.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/test-requirements.md),
not merely alongside it.

Milestone 2 is not required to fully implement every generic ultimate test, but
it **is** required to leave the runtime in a state where the relevant hostile
tests are directly supportable and where their pass conditions are already
driving implementation choices.

### Primary Milestone 2 Acceptance Pressure

The following test requirements are direct Milestone 2 acceptance drivers:

- `1. Hostile commit/replay equivalence test`
- `6. Diff/CDC truth parity test`
- `9. Bulk query and traversal stress truth test`
- `10. Durable recovery and schema mismatch test`
- `CAD 1. Topology identity survival test`
- `Chip 1. Netlist rewiring identity and history test`

This means Milestone 2 implementation must already guarantee:

- aspect-bearing commit artifacts are sufficient for replay-equivalent recovery
  of aspect-observable truth
- patch truth, CDC truth, replay truth, and historical aspect truth all agree
  on changed-aspect identity, structural change, and degraded-precision status
- bulk query and traversal surfaces consume committed aspect truth without
  falling back to per-record semantic reconstruction loops
- durable recovery and schema mismatch handling remain explicit and fail-closed
  when aspect-bearing artifacts or declaration fingerprints disagree
- topology- and netlist-like rewiring workloads can depend on truthful endpoint-
  bound aspect semantics and historically queryable rewiring deltas

### Adversarial Constraint Tests

The following test requirements are not the main feature target of Milestone 2,
but they are adversarial constraints on the implementation and completion
standard:

- `2. Savepoint rollback fracture test`
- `4. Deterministic observability under hostile scheduling test`
- `7. Lineage/correspondence hardening test`
- `8. Merge-ready history shape test`

Milestone 2 must therefore preserve these hostile properties:

- nested savepoint rollback leaves zero aspect residue in patch artifacts, CDC,
  lineage-observable history, and diagnostics
- scheduling variability may not change emitted aspect sets, patch ordering,
  trace ordering, diagnostics summaries, or history/query-observable aspect
  truth
- lineage-aware aspect history must preserve origin semantics exactly and may
  not collapse authoritative lineage identity evolution into aspect-local
  history reinterpretation
- ordered parent lists in canonical commit envelopes, replay, diagnostics, and
  branch reasoning must remain stable and merge-ready; Milestone 2 may not
  regress the runtime into implicit linear-history assumptions

### Required Adversarial Coverage Additions

Milestone 2 closeout must explicitly cover the following hostile cases even
when they appear only as sub-scenarios inside larger certification lanes:

#### Savepoint and rollback hostility

- aspect-bearing writes created and then abandoned behind nested savepoint
  rollback
- rollback after aspect delta computation but before publication
- alternate surviving path after rollback proving zero abandoned aspect residue

#### Scheduling hostility

- same aspect-bearing workload executed under different legal preparation or
  publication schedules
- trace and diagnostic ordering stability under schedule variation
- canonical aspect collections remaining byte-for-byte equivalent across legal
  schedules

#### Recovery and mismatch hostility

- recovery from durable canonical artifacts containing aspect-bearing patch
  truth
- explicit failure on schema or kind declaration mismatch involving
  `AspectPlanRevision`
- explicit failure on partial durable artifact presence where aspect-bearing
  truth cannot be reconstructed honestly

#### History and lineage hostility

- lineage traversal over aspect-bearing replace/rewire histories without origin
  mutation
- branch-local lineage divergence that must not leak through aspect-history
  resolution
- relation endpoint rewiring histories remaining historically inspectable
  through durable truth alone

### Required Machine-Checkable Outputs Under Adversarial Runs

When Milestone 2 is exercised in hostile certification lanes, the runtime must
be able to emit or derive canonical machine-checkable artifacts sufficient to
support the relevant `test-requirements.md` pass conditions.

At minimum, Milestone 2 must preserve or produce:

- `truth_digest`
- `patch_digest`
- `replay_digest`
- `diagnostics_digest`
- `query_surface_digest`
- `patch_vs_truth_delta_report`
- `aspect_tag_accuracy_report`
- `aspect_history_digest`
- `lineage_aspect_resolution_digest`

If a hostile run cannot produce those artifacts without payload rescans or
secondary semantic reconstruction, Milestone 2 is incomplete even if the
feature APIs appear to work.

## Milestone 2 Closeout Checklist

This checklist is intentionally adversarial. It is not a list of "tests that
sound nearby." It is the explicit closeout map from Milestone 2 requirements to
current test pressure and known remaining gaps.

Status markers:

- `[Covered]` means named tests already exercise the relevant hostile property
- `[Partial]` means some pressure exists, but the full hostile requirement is
  not yet pinned sharply enough for Milestone 2 closeout
- `[Missing]` means Milestone 2 still needs explicit coverage before it can be
  considered fully closed

### Commit / Replay / CDC Truth

- `[Covered]` Patch truth equals canonical aspect truth
  Current coverage:
  `tests::transactions::core::commit_publication_exposes_aspect_evaluation_and_emission_traces`
  `tests::transactions::core::entity_patch_aspects_follow_declared_semantics_not_payload_keys`
  `tests::transactions::core::retained_relation_patch_only_emits_declared_lifecycle_delta_when_endpoints_and_payload_stay_same`

- `[Covered]` Replay rejects canonical patch drift
  Current coverage:
  `tests::history::replay::replay_contract_reports_structured_patch_drift_when_canonical_envelope_is_tampered`

- `[Covered]` Replay preserves merge-ready ordered parent history
  Current coverage:
  `tests::history::queries::merge_commit_uses_deterministic_parent_order_and_advances_target_branch`
  `tests::history::replay::replay_contract_success_preserves_merge_parent_order`

- `[Covered]` CDC excludes abandoned savepoint work
  Current coverage:
  `tests::publication::cdc::savepoint_residue::savepoint_abandoned_work_never_appears_in_subscriber_cdc`
  `tests::publication::cdc::certification::cdc_certification_savepoint_abandoned_work_never_leaks_into_stream_truth`

- `[Covered]` Hostile commit/replay equivalence over aspect-bearing mixed workloads
  Current coverage:
  `tests::history::replay::replay_contract_success_reproduces_canonical_surfaces`
  `tests::publication::cdc::replay_parity::subscriber_stream_matches_patch_stream_for_committed_history`
  `tests::history::replay::replay_and_recovery_preserve_aspect_bearing_truth_across_a_hostile_mixed_workload`

### Savepoint / Rollback Hostility

- `[Covered]` Basic rollback leaves no surviving inner creation in committed truth
  Current coverage:
  `tests::transactions::core::savepoint_rollback_discards_inner_work_only`

- `[Covered]` CDC sees zero residue from abandoned savepoint paths
  Current coverage:
  `tests::publication::cdc::savepoint_residue::savepoint_abandoned_work_never_appears_in_subscriber_cdc`
  `tests::publication::cdc::certification::cdc_certification_savepoint_abandoned_work_never_leaks_into_stream_truth`

- `[Covered]` Aspect-bearing nested savepoint fracture
  Current coverage:
  basic rollback and CDC residue lanes above
  `tests::publication::cdc::savepoint_residue::nested_savepoint_abandoned_aspect_work_leaves_zero_patch_cdc_history_and_lineage_residue`

### Deterministic Observability / Scheduling

- `[Covered]` Canonical parent order and scan order remain deterministic
  Current coverage:
  `tests::history::queries::merge_commit_uses_deterministic_parent_order_and_advances_target_branch`
  `tests::query::entity_scans::entity_kind_scans_are_deterministic_across_equivalent_insert_order`
  `tests::query::relation_scans::relation_kind_scans_are_deterministic_across_equivalent_insert_order`

- `[Covered]` Harness parity lanes preserve observable publication surfaces
  Current coverage:
  `tests::publication::observability::harness_parity_suite_matches_serial_and_staged_parallel_runs`
  `tests::publication::observability::harness_parity_suite_matches_serial_and_post_commit_parallel_runs`
  `tests::transactions::core::staged_parallel_patch_preparation_matches_serial_patch_surface`

- `[Partial]` Aspect trace and diagnostics ordering under hostile scheduling
  Current coverage:
  parity and observability lanes above
  `tests::publication::observability::aspect_traces_and_diagnostics_are_stable_across_supported_execution_models`
  Gap:
  no explicit Milestone 2 test yet proves aspect evaluation traces, aspect
  emission traces, and aspect-history diagnostics remain byte-for-byte stable
  under legal scheduling variation

### History / Lineage / Resolution Semantics

- `[Covered]` Record-local aspect history reads committed patch truth
  Current coverage:
  `tests::history::queries::record_local_aspect_history_reads_committed_patch_truth`

- `[Covered]` Lineage-aware history preserves origin semantics and marks only resolution context
  Current coverage:
  `tests::lineage::historical_resolution::lineage_aspect_history_keeps_origin_events_and_marks_resolution_context`

- `[Covered]` Branch-local lineage divergence stays branch-local
  Current coverage:
  `tests::lineage::historical_resolution::historical_lineage_resolution_is_branch_local_under_divergent_replacements`

- `[Covered]` Lineage authority remains explicit and correspondence stays advisory until promotion
  Current coverage:
  `tests::lineage::contracts::lineage_contract_correspondence_stays_advisory_until_promoted`
  `tests::lineage::contracts::lineage_contract_failure_invalid_references_do_not_promote`

- `[Partial]` Relation endpoint rewiring history as durable aspect truth
  Current coverage:
  relation endpoint aspect emission is covered on commit
  `tests::profiles::compiled_artifacts::chip_profile_declared_aspect_fanout_preserves_endpoint_history_for_netlist_like_shapes`
  Gap:
  no dedicated Milestone 2 history test yet proves source/target-bound relation
  aspect rewires remain historically queryable from durable truth alone
  Disposition:
  deferred to Milestone 7B in
  [worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
  because Milestone 2 defines endpoint-bound aspect truth but does not define
  first-class relation rewiring as an authoritative mutation/reconciliation
  capability. Milestone 2 must preserve truthful endpoint-bound aspect
  semantics for supported relation changes and must not block later rewiring
  implementation, but it is not required to invent a full rewire product
  surface ahead of the roadmap milestone that owns it.

### Bulk Query / Traversal Pressure

- `[Covered]` Projection/query surfaces fail closed on undeclared aspect requirements
  Current coverage:
  `tests::query::projections::projection_rejects_undeclared_required_aspects`

- `[Covered]` Bulk scans remain deterministic and partition-bounded where promised
  Current coverage:
  `tests::query::entity_scans::entity_kind_scans_can_be_partition_scoped_without_cross_partition_leakage`
  `tests::query::entity_scans::entity_kind_scans_preserve_historical_partition_visibility`
  `tests::query::relation_scans::relation_kind_scans_return_only_visible_relations_of_that_kind`
  complexity budget lanes under `tests::complexity::contracts::visibility_budgets`

- `[Covered]` Aspect-filtered bulk stress truth
  Current coverage:
  record-local aspect history filter coverage exists
  `tests::history::queries::bulk_like_aspect_history_filters_and_query_packets_stay_stable_after_recovery`

### Durable Recovery / Schema Mismatch

- `[Covered]` Recovery and mismatch failures are explicit in the replay/durability layer
  Current coverage:
  `tests::durability::contracts::durability_contract_failure_schema_mismatch_is_explicit`
  `tests::durability::contracts::durability_contract_failure_missing_parent_chain_is_explicit`
  `tests::history::replay::replay_contract_failure_wrong_branch_is_explicit`
  `tests::history::replay::replay_contract_failure_missing_parent_chain_is_explicit`

- `[Covered]` Durable recovery preserves merge-ready history shape
  Current coverage:
  `tests::durability::contracts::durability_contract_recovery_preserves_merge_parent_order`

- `[Covered]` Aspect-plan mismatch and aspect-bearing durable recovery
  Current coverage:
  schema mismatch coverage exists generically
  `tests::durability::contracts::durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history`
  `tests::durability::contracts::durability_contract_failure_aspect_plan_mismatch_is_explicit`

### Domain Hostility

- `[Covered]` Topology identity survival pressure
  Current coverage:
  `tests::profiles::compiled_artifacts::compiled_artifact_rejects_stale_topology_after_later_commit`
  `tests::profiles::compiled_artifacts::chip_profile_branch_local_topology_pressure_preserves_relation_history_isolation`

- `[Partial]` Netlist rewiring identity and history pressure
  Current coverage:
  relation endpoint aspect semantics and history foundations are implemented
  `tests::profiles::compiled_artifacts::chip_profile_declared_aspect_fanout_preserves_endpoint_history_for_netlist_like_shapes`
  Gap:
  the roadmap-named netlist rewiring lane is not yet represented by a dedicated
  hostile scenario proving historically queryable rewiring deltas and
  connectivity-parity outputs
  Disposition:
  deferred to Milestone 7B in
  [worth_relational_roadmap.md](/Users/Esther/Documents/Programming/WORTH_workspace/WORTH/plans/worth-relational/worth_relational_roadmap.md)
  because the roadmap places first-class rewiring/merge/reconciliation
  semantics later than Milestone 2. Milestone 2 owns the endpoint-bound aspect
  truth foundation that netlist rewiring will consume, but not the full
  domain-grade rewiring capability itself.

### Closeout Standard

Milestone 2 closeout should not claim completion until all `[Partial]` items
above are either:

1. upgraded to `[Covered]` by named tests or certification lanes, or
2. explicitly deferred into the roadmap with a written reason that does not
   compromise the semantic claims of Milestone 2 itself

For avoidance of doubt, the two rewiring-oriented partials above are now
explicitly deferred rather than left unresolved:

- relation endpoint rewiring history as a full first-class capability is owned
  by Milestone 7B
- netlist rewiring identity/history pressure as a product/domain lane is owned
  by Milestone 7B and later domain certification work

Milestone 2 still owns and must preserve all prerequisite truth semantics for
that later work:

- endpoint-bound aspect declaration and lowering
- endpoint-bound canonical commit-time delta computation
- durable storage/publication of endpoint-bound aspect changes for supported
  relation mutations
- historical/query/diagnostic consumption of committed endpoint-bound aspect
  truth without payload rescans

## Milestone 2 Closeout Program

The remaining Milestone 2 work is not open-ended feature expansion. It is a
deliberate closeout program focused on converting the adversarial `[Partial]`
items above into named, machine-checkable proof.

This closeout program exists so Milestone 2 can be finished honestly rather
than declared complete on the basis of architectural intent plus scattered
coverage.

### Closeout Objective

Closeout is complete when the runtime can make the following claims without
semantic softness:

- canonical aspect truth survives hostile replay and durable recovery
- abandoned work leaves zero aspect residue anywhere observable
- legal scheduling variation cannot perturb aspect-observable outputs
- relation endpoint rewiring remains historically queryable from durable truth
- aspect-aware bulk reads remain canonical and proportionate under stress
- aspect-bearing schema/recovery mismatches fail explicitly
- topology- and netlist-like rewiring scenarios are covered by named hostile
  lanes, not only by generic relational tests

### Workstreams

Milestone 2 closeout is split into seven explicit workstreams.

#### Workstream 1: Hostile Aspect-Bearing Commit / Replay Equivalence

Purpose:
upgrade the generic hostile commit/replay requirement into an aspect-bearing
Milestone 2 certification lane.

Required scenario shape:

- long deterministic workload
- entity aspect changes
- relation aspect changes
- relation endpoint rewires
- deletes and retained-for-audit transitions
- branch creation and branch-local commits
- lineage-affecting replacements
- nested savepoints
- rollback injections
- snapshot capture
- durable recovery

Required comparisons:

- original authoritative run
- replay from canonical commit envelopes
- replay from snapshot plus suffix commit envelopes
- fresh runtime recovered from durable canonical artifacts

Required outputs:

- `truth_digest`
- `patch_digest`
- `replay_digest`
- `diagnostics_digest`
- `query_surface_digest`
- `patch_vs_truth_delta_report`
- `aspect_tag_accuracy_report`
- `aspect_history_digest`
- `lineage_aspect_resolution_digest`

Pass condition:

- all compared executions produce equivalent aspect-observable truth
- no aspect set, structural change, degraded-precision flag, history output, or
  diagnostic digest diverges across equivalent histories

Suggested implementation lane:

- new certification test under `tests/history` or `tests/harness`
- use named aspect-heavy fixture builders from `tests/support.rs`

#### Workstream 2: Nested Savepoint Aspect-Residue Fracture

Purpose:
prove that nested rollback leaves zero authoritative or historical aspect
residue.

Required scenario shape:

- one outer transaction
- nested savepoint A
- nested savepoint B
- entity and relation aspect mutations before and after each savepoint
- at least one relation endpoint rewire in an abandoned path
- rollback to B, alternate path
- rollback to A, alternate path
- final commit on surviving path

Required verifications:

- no abandoned path appears in patch truth
- no abandoned path appears in CDC
- no abandoned path appears in aspect history
- no abandoned path appears in lineage-aware aspect history
- no abandoned path appears in diagnostics or trace digests
- surviving path remains exact

Required outputs:

- abandoned mutation residue report
- patch fragment inclusion report
- `patch_vs_truth_delta_report`
- `aspect_history_digest`
- `lineage_aspect_resolution_digest`

Pass condition:

- rolled-back aspect-bearing work leaves zero authoritative residue

Suggested implementation lane:

- dedicated transaction/history/publication certification test
- likely under `tests/publication/cdc` plus one direct transaction/history test

#### Workstream 3: Scheduling-Stable Aspect Observability

Purpose:
prove that legal scheduling differences cannot perturb aspect-observable output.

Required scenario shape:

- same aspect-bearing workload under multiple legal execution/scheduling modes
- serial authority lane
- staged parallel preparation lane
- post-commit parallel consumption lane where applicable
- legal variation in fragment preparation and diagnostic materialization order

Required verifications:

- patch ordering stable
- `CanonicalAspectSet` contents stable
- aspect evaluation trace ordering stable
- aspect emission trace ordering stable
- history resolution trace ordering stable
- diagnostics digest stable

Required outputs:

- per-run canonical artifact bundle
- mismatch matrix by observable surface
- `patch_digest`
- `diagnostics_digest`
- `query_surface_digest`

Pass condition:

- schedule variation does not change any aspect-observable surface

Suggested implementation lane:

- extend existing observability/harness parity matrix
- add aspect-specific assertions rather than relying only on generic bundle
  equality

#### Workstream 4: Aspect-Bearing Durable Recovery and Plan-Mismatch Failure

Purpose:
prove that durable recovery is exact when valid and fail-closed when
aspect-bearing declaration truth diverges.

Required scenario shape:

- persist aspect-bearing committed history
- recover cleanly
- replay recovered history
- induce aspect-bearing mismatch cases:
  - schema mismatch
  - kind declaration mismatch
  - `AspectPlanRevision` mismatch
  - partial durable artifact presence where aspect-bearing truth is incomplete

Required verifications:

- valid recovery reconstructs exact aspect-observable truth
- invalid recovery fails explicitly with structured failure class
- no silent fallback to payload reinterpretation occurs

Required outputs:

- durable artifact completeness report
- recovery truth digest
- schema/kind/plan mismatch report
- recovery failure taxonomy summary

Pass condition:

- recovery is exact when valid and loudly rejected when invalid

Suggested implementation lane:

- extend durability contract tests with aspect-bearing declaration fixtures
- add replay-side mismatch assertions involving `AspectPlanRevision`

#### Workstream 5: Durable Relation Rewire History

Purpose:
prove that endpoint-bound aspect semantics stay historically queryable from
durable truth, not just commit-time patch emission.

Required scenario shape:

- create relation with source/target-bound aspects
- perform multiple rewires across commits
- include at least one branch-local divergence
- include at least one lineage-affecting replacement on an endpoint or adjacent
  entity where relevant

Required verifications:

- direct relation aspect history records source/target aspect changes correctly
- lineage-aware history preserves origin semantics and only changes resolution
- historical reads do not require payload rescans
- rewire events remain visible after durable recovery/replay

Required outputs:

- relation aspect history digest
- lineage aspect resolution digest for rewire lineage where applicable
- patch/history parity report for rewire commits

Pass condition:

- endpoint rewiring is historically inspectable as durable aspect truth

Suggested implementation lane:

- new direct history test under `tests/history`
- one recovery/replay companion under `tests/durability` or `tests/history`

#### Workstream 6: Aspect-Filtered Bulk Stress Truth

Purpose:
prove that aspect-aware bulk reads behave like first-class APIs under graph
stress rather than stitched single-record loops.

Required scenario shape:

- large cyclic graph
- multiple entity and relation kinds
- relation aspects and endpoint-bound aspects
- branch-local deltas
- historical versions
- hot and cold graph regions
- mixed `Any` and `All` aspect filters

Required verifications:

- result parity across query paths
- canonical result order
- snapshot isolation under later mutation
- no hidden semantic reconstruction from payload scans
- work remains proportional where the runtime promises bounded surfaces

Required outputs:

- query result digests
- path parity matrix
- snapshot isolation matrix
- touched-state/work-packet metrics

Pass condition:

- aspect-filtered bulk surfaces remain canonical, isolated, and proportionate

Suggested implementation lane:

- extend bulk query/traversal tests under `tests/query`
- connect to complexity/visibility budget assertions where relevant

#### Workstream 7: Domain Hostility Lanes

Purpose:
close the gap between generic relational aspect semantics and the two
domain-critical hostile scenarios already named in the plan.

Sub-lane A: topology identity survival pressure

Required emphasis:

- topology-adjacent relation rewiring
- replacement/split-like history pressure where supported today
- historically queryable adjacency/relation truth
- branch-local history isolation

Required outputs:

- topology-oriented truth snapshot bundle
- lineage ancestry graph for selected entities
- relation-history report for selected topology relations

Sub-lane B: netlist rewiring identity and history pressure

Required emphasis:

- connectivity rewires through relation endpoint changes
- branch-local alternate rewires
- historically queryable net/cell connectivity deltas
- CDC/connectivity parity outputs

Required outputs:

- connectivity truth snapshot bundle
- relation-history / lineage report for selected rewires
- CDC/connectivity parity report

Pass condition for both:

- the domain lane can show that Milestone 2 aspect semantics survive a hostile
  structurally meaningful scenario rather than only toy relational fixtures

Suggested implementation lane:

- initial representation may live in profile/domain tests rather than a new
  standalone product subsystem

### Execution Order

The closeout program should be executed in this order:

1. Workstream 4: durable recovery and plan-mismatch failure
2. Workstream 2: nested savepoint aspect-residue fracture
3. Workstream 5: durable relation rewire history
4. Workstream 1: hostile aspect-bearing commit/replay equivalence
5. Workstream 3: scheduling-stable aspect observability
6. Workstream 6: aspect-filtered bulk stress truth
7. Workstream 7: domain hostility lanes

Rationale:

- Workstream 4 protects semantic trust in artifacts before broader hostile runs
- Workstream 2 closes residue lies early
- Workstream 5 closes the most important history gap for rewiring semantics
- Workstream 1 then proves the full canonical flow under hostile replay
- Workstream 3 and 6 harden determinism and query stress once truth is already
  proven
- Workstream 7 finishes with domain-hostile evidence rather than generic-only
  confidence

### Deliverable Shape

Each workstream should end with:

- one or more named tests or certification lanes
- machine-checkable artifact outputs, not only assertions over human-readable
  logs
- an explicit mapping from produced outputs to the relevant
  `test-requirements.md` pass condition
- an update to the `[Covered]` / `[Partial]` checklist above

### Honest Closeout Rule

Milestone 2 must not be marked complete simply because the runtime APIs are in
place and the semantic center is sound.

Milestone 2 is complete only when this closeout program has either:

1. converted the adversarial partials into explicit covered lanes, or
2. produced a written roadmap deferral that does not weaken the truth claims
   Milestone 2 makes today
