# Milestone 9.3.4 Engineering Spec: Declared Projection Consumption And Materialized Fact Receipts

> **Status:** Draft
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-9.3.3.md](./milestone-9.3.3.md)
>
> **Foundational precedent:** [milestone-9.md](./milestone-9.md)
> established authorized projection as the execution-facing result of policy
> masking. Milestone 9.3.4 must turn authorized and materialized projection
> surfaces into one typed fact-consumption lifecycle rather than leaving
> consumers to rediscover fact meaning from authority state or host caches.
>
> **Next milestone:** [Milestone 9.3.5](./worth_query_roadmap.md#milestone-935-intent-admission-decision-lattice-and-decision-trace)
> will unify projection-consumption admission with the broader Query-crossing
> decision lattice, but 9.3.4 must first make consumed projection facts
> declared, typed, and receipt-backed.
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make every consumed projection fact derive
> from one declared materialization contract and one receipt-backed fact set
> rather than from direct source-authority rediscovery.
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [composition_laws.md](../../docs/coding-guidelines/composition_laws.md)
> - [domain_structure_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [dx_laws.md](../../docs/coding-guidelines/dx_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md)
> - [milestone-9.md](./milestone-9.md)
> - [milestone-9.1.md](./milestone-9.1.md)
> - [milestone-9.3.2.md](./milestone-9.3.2.md)
> - [milestone-9.3.2-closeout.md](./milestone-9.3.2-closeout.md)
> - [milestone-9.3.3.md](./milestone-9.3.3.md)
> - [milestone-9.3.3-closeout.md](./milestone-9.3.3-closeout.md)

## Goal

Make materialized Query projection consumption a single proof-widening
lifecycle, grounded in the Query surfaces that already exist today:

```text
ProjectionConsumptionDeclaration
  -> ProjectionConsumptionEligibility
  -> MaterializedProjectionContract
  -> ConsumedProjectionFactSet
  -> ProjectionConsumptionReceipt
  -> SelfDescribingProjectionConsumptionEnvelope
  -> ProjectionConsumptionCertificationBundle
```

The abstract phase names above must map onto concrete Query-owned artifacts that
bind:

- the canonical query/result-shape identity
- the authorized projection identity from Milestone 9
- the admitted materialization basis from Milestone 9.3.2
- the materialization source artifact from read execution, effect execution, or
  later runtime-backed projection maintenance
- the exact fact families the caller intends to consume

Milestone 9.3.4 may introduce new sealed wrappers and public facade helpers,
but it must not invent a second unrelated vocabulary for facts that already
have Query-owned names. Existing materialization, authorized projection,
basis-lifecycle, and effect-receipt artifacts should be reused directly or
wrapped thinly rather than duplicated under parallel public concepts.

## Why This Milestone Exists

Milestone 9 made policy-aware authorized projection a first-class execution
artifact. Milestone 9.3.2 then made basis use explicit and receipt-backed, and
Milestone 9.3.3 made Query-authored effect execution produce one lowered,
inspectable receipt/envelope story.

What remains open is the consumer side of materialized facts.

Without Milestone 9.3.4, ordinary callers can still receive a materialized
projection and then immediately reopen relational truth, bridge-managed preview
state, signal aftermath, or host-local caches to rediscover the identities,
memberships, labels, topology endpoints, shape facts, or derived fields they
actually depend on. That failure mode hides authority leaks behind apparently
successful materialization. The projection looks typed at the read boundary, but
the consumer contract remains ambient and unverifiable.

This milestone exists to close that seam before the admission lattice,
lower-runtime routing, runtime API freeze, and later temporal/durable
milestones build on top of projection-derived behavior.

## Governing Summaries

- `MENTALITY.md`: the hard case is not "read a projected field." It is
  guaranteeing that every consumer-visible fact is declared up front and
  derives from one materialized contract rather than from opportunistic
  authority rediscovery.
- `arch_laws.md`: materialized fact use must be a proof chain with typed
  admission, typed denial, sealed receipt construction, and a facade-owned
  surface that makes invalid consumption unrepresentable or uncompilable.
- `composition_laws.md`: declaration, eligibility, contract binding, fact-set
  extraction, receipt shaping, DX helpers, and certification must stay separate
  responsibilities rather than collapsing into one projection helper bucket.
- `domain_structure_laws.md`: projection-consumption code must be physically
  locatable as its own lifecycle, with clear separation between fact taxonomy,
  source adapters, receipt/envelope shaping, inventories, and certification.
- `perf_laws.md`: projection consumption may not hide authority reopen scans or
  host-side recomputation. Exact counters and slope proofs must show costs are
  bounded by declared fact width, materialized row width, and source evidence
  width.
- `dx_laws.md`: the common path must read like "consume these facts from this
  materialization," while advanced APIs may expose lower layers explicitly.
  Expensive source reinspection and unsupported fact families must look
  expensive or fail typed.
- `worth_query_vision.md`: aspect projection is explicit, materialization is
  bounded, typed result shapes are canonical, and consumers should use query
  shaped facts rather than raw storage or ad hoc recomputation.
- `worth_query_roadmap.md`: 9.3.4 sits after basis lifecycle and effect
  execution because materialized fact receipts must bind to admitted basis and
  effect/read materialization artifacts, and before 9.3.5/9.3.6 because later
  decision traces and lower-runtime routing need one honest consumed-fact seam.
- `test-requirements.md`: closure requires a named certification suite,
  canonical machine-checkable bundles, hostile denial lanes, compile-fail
  boundaries, and exact counters.
- `milestone-9.md`: authorized projection is already the execution-facing
  result of policy masking. 9.3.4 must consume that artifact instead of letting
  callers rediscover visible/masked meaning locally.
- `milestone-9.1.md`: causal inspection already materializes Query-facing
  artifacts with explicit policy/result-shape context. 9.3.4 should reuse that
  discipline for projection facts rather than making post-read explanation a
  separate fact-discovery path.
- `milestone-9.3.2.md`: materialization basis is already a typed capability
  lane, and basis receipts already declare `ProjectionConsumption` as a
  permitted next transition. 9.3.4 must close that named transition.
- `milestone-9.3.3.md`: effect receipts explicitly defer projection fact
  receipts to 9.3.4. This milestone must make effect-produced materializations
  consumable without reopening execution authority.

## Adversarial Constraint

Under policy-masked detail and collection projections, bounded topology-style
materializations, branch/preview/historical basis variation, query-context
materializations, effect-produced writeback aftermath, and host attempts to
cache or recompute "helpful" view facts locally, the same declared consumed
fact request must produce the same admitted fact contract, the same typed fact
set, and the same receipt/envelope meaning regardless of whether the
materialization came from read execution, effect execution, or compatible
runtime-backed maintenance.

Unsupported, masked, stale, basis-incompatible, source-mismatched,
policy-drifted, or overclaiming fact requests must fail typed and early before
the consumer receives a fact set. No public or internal caller may treat raw
entity rows, arbitrary JSON payloads, bridge preview internals, signal state,
or host-local caches as substitute authority for consumed projection facts.

## Product Decision Lock

The product surface is one Query-owned consumed-fact lifecycle.

The public runtime API must let downstream domains:

- declare which projection facts they intend to consume
- bind those facts to one admitted materialization contract
- receive one typed fact set and one receipt/envelope story
- inspect denial, support, and cost posture without rediscovering fact meaning

without:

- re-reading relational truth or preview internals to recover identifiers,
  memberships, labels, topology edges, or derived values
- pattern-matching raw materialized JSON as the canonical public fact surface
- treating effect receipts, materialized read rows, or causal artifacts as
  ambient permission to consume any convenient fact
- rebuilding policy, basis, or result-shape meaning locally after
  materialization

Crate ownership is load-bearing:

- `worth-query` owns projection-consumption declarations, eligibility, contract
  binding, typed consumed fact sets, public receipts/envelopes, support
  metadata, DX helpers, and certification.
- `worth-relational` remains authoritative for truth identities, relation
  topology, snapshot meaning, commit history, and branch/head semantics.
- `worth-runtime-bridge` remains authoritative for preview/writeback lifecycle,
  bridge-owned route/effect evidence, and any lower-runtime writeback artifacts
  referenced by consumed facts.
- `worth-signal` may contribute downstream invalidation or explanation evidence,
  but it must not become the canonical source of consumed projection facts.
- `worth-store` remains deferred for persisted consumed-fact receipts, durable
  projection reload, store-backed reconstruction parity, and portable receipt
  archives.

Forbidden in `worth-query`:

- exposing raw `serde_json::Value`, ad hoc row maps, or host-specific structs as
  the canonical consumed-fact artifact
- accepting "materialized projection plus some callback/lookups" as equivalent
  to a declared consumed-fact contract
- re-reading source authority after materialization just to recover facts that
  the declaration should have named
- flattening admitted, admitted-with-warnings, denied, deferred, and
  source-mismatch fact
  postures into one loose bag with optional fields
- allowing one materialized source family to overclaim facts that only another
  source family can prove
- claiming persisted consumed-fact reload, restart-stable fact receipts, or
  store-backed reconstruction before Milestones 10 and 11 close

## Existing Surfaces To Consolidate

Milestone 9.3.4 does not start from nothing. It must consolidate and
strengthen existing Query-owned projection/materialization surfaces:

- authorized projection artifacts from Milestone 9, especially:
  - `AuthorizedProjectionArtifact`
  - `AuthorizedProjectionIdentity`
  - `MaskedProjectionArtifact`
  - `PolicyFieldInfluenceSet`
- materialization basis and next-transition evidence from Milestone 9.3.2,
  especially:
  - `evaluate_basis_materialization_eligibility`
  - `scope_basis_for_materialization`
  - `emit_materialization_basis_receipt`
  - `BasisUseReceiptKind::Materialization`
  - `BasisNextTransition::ProjectionConsumption`
- effect-produced materialization sources and transition rules from Milestone
  9.3.3, especially:
  - `WORTHQueryWriteReceipt`
  - `WORTHQueryBatchWriteReceipt`
  - `WORTHQueryIntentExecution`
  - `EffectReceiptTransitionRules`
  - `SelfDescribingEffectEnvelope`
- runtime-backed materialization helpers already present in Query, especially:
  - runtime materialized read-view construction in
    `runtime/read_composition_materialization.rs`
  - `QueryContextExecutionArtifact`-backed materialized rows
  - causal-inspection materialization artifacts where already admitted
- existing canonical query/result-shape and view-shape artifacts, especially:
  - `CanonicalQueryArtifact`
  - `CanonicalResultShapeArtifact`
  - `ValidatedResultShapeArtifact`
  - `DeclarativeLiveViewShape`

Normative consequence:

- if a materialized source already has a Query-owned receipt, envelope,
  authorized-projection identity, or basis receipt, 9.3.4 must consume that
  artifact rather than reopening lower-runtime state
- if a source has no honest Query-owned materialization artifact yet, 9.3.4
  must deny or defer that consumed-fact lane rather than speculating over raw
  source state
- if multiple source families can produce the same consumed fact, the fact
  contract must bind which source family proved it

## Required Crate Changes

This milestone is not "whatever crate ends up being convenient." It requires
explicit changes at explicit boundaries.

### `worth-query`: required changes

`worth-query` is the owning crate for Milestone 9.3.4. It must add:

- a new `projection_consumption` subdomain under `crates/worth-query/src`
- `ProjectionConsumptionDeclaration`,
  `ProjectionConsumptionEligibility | DeniedProjectionConsumption`,
  `MaterializedProjectionContract`, `ConsumedProjectionFactSet`,
  `ProjectionConsumptionReceipt`,
  `SelfDescribingProjectionConsumptionEnvelope`, and
  `ProjectionConsumptionCertificationBundle`
- a typed fact taxonomy covering identity, endpoint, membership, label/display,
  derived scalar, shape, view-local identity, and source-reference families
- source adapters that consume existing Query-owned artifacts:
  - `WORTHQueryReadReceipt`
  - `WORTHQueryWriteReceipt`
  - `QueryContextExecutionArtifact`
  - admitted causal-inspection materialization artifacts where the current
    public surface already exposes them honestly
- facade exports for the new lifecycle
- compile-fail boundaries proving external callers cannot mint admitted
  contracts, fact sets, receipts, envelopes, or certification artifacts
- the named 9.3.4 certification suite and all required verification outputs

`worth-query` must also tighten existing public surfaces so 9.3.4 does not rely
on raw payload bags as its final public contract:

- `WORTHQueryDerivedViewMaterialization` may remain an internal/source-facing
  runtime materialization holder, but it must not become the canonical
  consumed-fact API
- any new common-path projection-consumption APIs must return the 9.3.4 proof
  chain, not raw `Vec<Value>`, raw `Vec<String>`, or ad hoc `WORTHQueryEntity`
  parsing helpers

### `worth-runtime-bridge`: required changes

`worth-runtime-bridge` must remain the owner of bridge truth-view
materialization and bridge writeback provenance. For 9.3.4 it must expose, and
Query must consume, bridge-owned source artifacts rather than reconstructing
them.

Required bridge boundary for this milestone:

- Query must consume `MaterializedTruthViewObservation`,
  `BridgeMaterializedRowSetArtifact`, and `BridgeGroupedTruthViewArtifact` as
  bridge-owned source evidence when a consumed-fact lane is bridge-backed
- Query must consume existing bridge writeback authority artifacts already
  reachable through `WORTHQueryWriteReceipt` provenance/evidence instead of
  reopening bridge protocol state locally

Required bridge code changes:

- none, unless an implementation pass proves one specific source artifact needed
  by the Query adapter is not yet publicly reachable through the current bridge
  facade

If such a gap appears, the allowed bridge change is narrow:

- add one facade-exported bridge source artifact or accessor that exposes
  already-authoritative bridge materialization provenance

Forbidden bridge changes:

- moving consumed-fact declaration, eligibility, or receipt ownership into
  bridge
- teaching bridge about Query policy narrowing, Query fact taxonomy, or Query
  certification rows

### `worth-relational`: required changes

`worth-relational` must remain the owner of authoritative row identity, field
binding, grouped truth, snapshot meaning, and historical/merge truth.

Required relational boundary for this milestone:

- Query must treat `RelationalAuthoritativeRowSetArtifact` and
  `RelationalGroupedProjectionArtifact` as lower-runtime authority evidence,
  not as structures to be recreated in Query
- grouped/topology-style consumed facts must bind to relational row identity,
  field binding, snapshot identity, and grouped projection digest where those
  lanes are relationally sourced

Required relational code changes:

- none for the first 9.3.4 slice if the initial hostile lane can be expressed
  using the already-exported row-set and grouped-projection artifacts

Allowed relational follow-up if the topology/Worth hostile lane needs one more
authoritative hook:

- add one explicit facade-level grouped/topology authority artifact or accessor
  in `worth-relational` rather than rebuilding that authority in Query

Forbidden relational changes:

- adding Query-shaped consumed-fact receipts to relational
- moving Query declaration/eligibility/certification logic into relational

### `worth-signal`: required changes

`worth-signal` is not an owner of consumed projection fact authority in 9.3.4.

Required signal boundary for this milestone:

- no new source-of-truth or consumed-fact APIs move into `worth-signal`
- Query may reference signal-owned invalidation, diagnostics, replay, or
  history evidence only as downstream explanation/support context

Required signal code changes:

- none for Milestone 9.3.4

Forbidden signal changes:

- adding projection-consumption declarations, contracts, fact sets, or receipts
- treating signal snapshots/history as a substitute materialization authority
  for Query consumed facts

### `worth-store`: required changes

- none in 9.3.4
- persisted consumed-fact receipts, durable reload, and store-backed
  reconstruction remain later-milestone work exactly as the roadmap says

## Typed Phase Progression Lock

Milestone 9.3.4 must introduce or certify this progression:

```text
ProjectionConsumptionDeclaration
  -> ProjectionConsumptionEligibility | DeniedProjectionConsumption
  -> MaterializedProjectionContract
  -> ConsumedProjectionFactSet
  -> ProjectionConsumptionReceipt
  -> SelfDescribingProjectionConsumptionEnvelope
  -> ProjectionConsumptionCertificationBundle
```

The public phase names above are normative. The implementation may wrap or
rename internal helpers, but the user-visible lifecycle must preserve the same
ordered proof steps.

Minimum semantic meaning of each phase:

- `ProjectionConsumptionDeclaration`
  - names the fact families the caller wishes to consume
  - binds to one canonical query/result-shape/view-shape identity
  - binds to one materialization source family and one admitted basis/materialization receipt
- `ProjectionConsumptionEligibility`
  - proves the requested fact families are visible, supported, source-valid,
    and phase-valid for the declared materialization
  - distinguishes admitted, admitted-with-warnings, denied, deferred, and
    source-mismatch outcomes
- `MaterializedProjectionContract`
  - freezes the admitted consumed-fact meaning for one materialization
  - binds query, result shape, authorized projection, basis, policy,
    tenant/schema, source receipt, and source materialization digests
- `ConsumedProjectionFactSet`
  - exposes only the typed facts declared and admitted by the contract
  - is proof-bearing, not a raw materialized row bag
- `ProjectionConsumptionReceipt`
  - records exactly which source materialization proved which admitted facts,
    which denials/deferred neighbors were encountered, and what work occurred
- `SelfDescribingProjectionConsumptionEnvelope`
  - derives inspection/support/integrity summaries from the receipt
  - does not become a second canonical fact source
- `ProjectionConsumptionCertificationBundle`
  - proves canonical identity, typed denials, public-boundary enforcement,
    support parity, DX closure, and cost contracts

## Fact Taxonomy Lock

Consumed facts are not one generic "projection value" family. Milestone 9.3.4
must introduce or certify a typed taxonomy that can distinguish at least:

- entity identity facts
- relation identity or endpoint facts
- membership facts for collection/grouped/topology-style materializations
- label/name/display facts already admitted by the authorized projection
- derived scalar facts backed by declared query/result-shape meaning
- shape facts such as grouping bucket, ordering anchor, or view-local row role
- view-local identity facts needed to address stable rows/cards/edges without
  reopening source authority
- source-reference facts that point at an effect receipt, materialized read
  digest, or query-context artifact without reauthoritizing lower-runtime state

The taxonomy must be:

- closed enough that the certification suite can enumerate admitted and denied
  families explicitly
- openable through later milestones without renaming or replacing 9.3.4 facts
- separate from raw result-shape fields, because not every emitted field is a
  first-class consumed fact and not every consumed fact is just a scalar field

Forbidden taxonomy shortcuts:

- one universal `ProjectionFact { key: String, value: Value }`
- boolean flags that collapse membership, identity, label, and derived meaning
  into one bag
- source-family-specific public fact types that bypass the shared lifecycle
- host-local enum/string overlays treated as the canonical public fact model

Practical taxonomy consequences:

- row identity is not the same thing as consumed entity identity:
  - row identity names one materialized row inside one source artifact
  - entity identity fact names the domain identity the caller may consume
  - a source family may prove one, both, or neither; the contract must say
    which one is admitted
- grouping membership is not "the caller noticed two rows share a label":
  - membership facts must come from an admitted grouped/materialized source
    contract
  - host-side comparison of arbitrary row payloads does not create an admitted
    membership fact
- derived scalar facts are not free-form JSON fields:
  - a derived scalar fact must name the result-shape field or computation family
    that proved it
  - two fields with the same payload but different semantic provenance are
    different facts
- source-reference facts are references, not authority:
  - they may point to a receipt, digest, or source artifact identity
  - they do not grant permission to reopen the lower-runtime source directly

## Initial Source-Family Lock

Milestone 9.3.4 must not begin with an open-ended "any materialization source"
story. The first implementation slice must lock a concrete source-family set.

Required source families for the first 9.3.4 implementation:

- `QueryReadReceiptSource`
  - authoritative Query source artifact:
    `WORTHQueryReadReceipt`
  - companion source payload:
    `WORTHQueryReadResult`
  - initial admitted fact families:
    entity identity, label/display, derived scalar, view-local identity
- `QueryWriteReceiptSource`
  - authoritative Query source artifact:
    `WORTHQueryWriteReceipt`
  - initial admitted fact families:
    target identity, source-reference, effect-aftermath relation/continuity
    facts already proven by the write receipt and its bridge-backed evidence
- `QueryContextExecutionSource`
  - authoritative Query source artifact:
    `QueryContextExecutionArtifact`
  - initial admitted fact families:
    source-reference, result identity, and narrow context-proven payload facts
    explicitly named by the Query contract
- `RelationalRowSetSource`
  - authoritative lower-runtime source artifact:
    `RelationalAuthoritativeRowSetArtifact`
  - initial admitted fact families:
    row identity, entity identity, label/display, derived scalar where the
    bound field provenance is explicit
- `RelationalGroupedProjectionSource`
  - authoritative lower-runtime source artifact:
    `RelationalGroupedProjectionArtifact`
  - initial admitted fact families:
    membership, grouping/shape, relation-endpoint-adjacent grouped facts, and
    view-local identity
- `BridgeTruthViewRowSetSource`
  - authoritative lower-runtime source artifact:
    `BridgeMaterializedRowSetArtifact`
  - initial admitted fact families:
    row identity, entity identity, label/display, derived scalar where the
    bridge truth-view observation already proves the field binding
- `BridgeGroupedTruthViewSource`
  - authoritative lower-runtime source artifact:
    `BridgeGroupedTruthViewArtifact`
  - initial admitted fact families:
    membership, grouping/shape, and bridge-backed grouped identity facts

Explicitly not in the first 9.3.4 implementation:

- raw `WORTHQueryDerivedViewMaterialization` rows as a public source family
- arbitrary causal-inspection payload parsing as a generic fact source family
- signal snapshot/history materialization as a fact authority family
- portable/store-backed/restart-stable source families

Any source family outside the list above must be denied or deferred in the
first implementation slice.

## Mechanical Enforcement Lock

The spec must not rely on reviewers remembering the right way to consume facts.
The following enforcement shape is required:

- each phase artifact must have private fields and non-public constructors
- admitted and denied/deferred/source-mismatch artifacts must be separate types,
  not one struct with optional fields
- warnings must decorate admission rather than become a junk-drawer middle
  posture:
  - `ProjectionConsumptionEligibility::Admitted(AdmittedProjectionConsumption)`
  - `ProjectionConsumptionEligibility::AdmittedWithWarnings(
        AdmittedProjectionConsumption,
        ProjectionConsumptionWarnings,
    )`
  - `ProjectionConsumptionEligibility::Denied(DeniedProjectionConsumption)`
  - `ProjectionConsumptionEligibility::Deferred(DeferredProjectionConsumption)`
  - `ProjectionConsumptionEligibility::SourceMismatch(
        SourceMismatchedProjectionConsumption,
    )`
- `ProjectionConsumptionWarnings` must remain narrow:
  - it may describe caveats on an otherwise admitted fact consumption path
  - it must not stand in for partial support, uncertified support, or
    "denied but politely"
  - if the caller cannot honestly proceed to contract binding and extraction,
    the outcome is not warning-bearing admission; it is denied, deferred, or
    source-mismatch
- source-family binding must be type-visible:
  - either distinct source-family witness types
  - or a sealed source-family enum plus phase-specific wrappers
  - but not loose strings passed between phases
- `ConsumedProjectionFactSet` must expose typed accessors per admitted fact
  family rather than one raw collection of untyped values
- compile-fail fixtures must prove:
  - raw source artifacts cannot be passed where a
    `MaterializedProjectionContract` is required
  - a contract from one source family cannot be used with a fact extractor for
    another source family
  - denied/deferred/source-mismatch artifacts cannot be used as admitted fact
    sets
  - raw payload rows cannot be treated as public consumed facts
- support inventories and transition rules must derive from executable
  admission/extraction code paths, not a hand-maintained side table

### Support Matrix Derivation Mechanism

The support matrix must be mechanically generated from the same source-family
evaluators used by eligibility and extraction. "Derived from behavior" is not
satisfied by a separately maintained reporting table.

The implementation must include a Query-owned support/evaluation seam with
roughly this responsibility split:

```rust
trait ProjectionConsumptionSourceEvaluator {
    const SOURCE_FAMILY: ProjectionSourceFamily;

    fn supported_fact_families(&self) -> ProjectionFactFamilySet;

    fn evaluate_fact_family(
        &self,
        family: ProjectionFactFamily,
        context: &ProjectionConsumptionEvaluationContext,
    ) -> ProjectionFactFamilySupport;
}
```

The exact names may differ, but the mechanism is load-bearing:

- the evaluator used by admission must be the evaluator used to generate the
  support matrix
- support rows must be emitted by iterating source families Ã— fact families
  through these evaluators
- support output may cache or materialize results, but it must not invent a
  second semantic decision table
- if extraction requires a narrower proof than support alone, the evaluator
  result must say so explicitly rather than silently overclaiming support

`ProjectionFactFamilySupport` should be structurally aligned with eligibility
posture:

- admitted
- admitted-with-warnings
- denied
- deferred
- source-mismatch or not-applicable, whichever better matches the final public
  shape

The support layer is allowed to summarize these outcomes. It is not allowed to
reinterpret them.

## Target Finished Code Shape

Milestone 9.3.4 is not done when the proof chain merely exists. It is done
when the public code shape makes the right thing feel natural and the wrong
thing feel structurally alien.

The finished code should read like this:

- the common path reads like "consume these facts from this materialization"
- the advanced path reads like "declare, check support, bind a contract, then
  extract and inspect"
- denial, defer, source-mismatch, and warning-bearing admission paths are
  first-class values, not
  comments, booleans, or log-driven conventions
- source artifacts stay visible at the boundary where their semantics matter,
  but raw rows and raw payload bags disappear behind Query-owned fact accessors

### Common Path DX

The ordinary caller path should be short, typed, and semantically obvious.

Read-backed materialization consumption should look like:

```rust
let consumed = read_receipt
    .consume_projection_facts(
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .labels()
            .derived_scalars([UserProfileView::display_name()]),
    )?;

for identity in consumed.facts().entity_identities() {
    // ordinary domain work
}

let receipt = consumed.receipt();
let envelope = receipt.to_self_describing_envelope();
```

Effect-backed materialization consumption should look like:

```rust
let aftermath = write_receipt
    .consume_projection_facts(
        ProjectMaterializedFacts::declare()
            .target_identity()
            .source_references()
            .effect_continuity_facts(),
    )?;

let target = aftermath.facts().target_identity()?;
let bridge_reference = aftermath.facts().source_references();
```

Grouped/topology-style consumption should look like:

```rust
let grouped = grouped_projection
    .consume_projection_facts(
        ProjectMaterializedFacts::declare()
            .memberships()
            .relation_endpoints()
            .view_local_identities(),
    )?;

for membership in grouped.facts().memberships() {
    // use admitted grouped truth, not row comparison
}
```

The common path must not require the caller to:

- ask for raw rows first and then reinterpret them locally
- name bridge or relational internals unless the source family itself is the
  deliberate subject of the call
- stitch together separate helper calls for support lookup, extraction, and
  receipt shaping just to do the ordinary admitted thing

### Advanced Path DX

The advanced path should expose the real phase progression without collapsing
it into one magic helper.

The public shape should support code that reads roughly like:

```rust
let declaration = ProjectMaterializedFacts::declare()
    .source(QueryReadReceiptSource::from(&read_receipt))
    .entity_identities()
    .labels()
    .derived_scalars([UserProfileView::display_name()]);

let eligibility = declaration.evaluate()?;

let contract = eligibility.into_admitted()?.bind_contract()?;
let facts = contract.extract()?;

let receipt = facts.issue_receipt()?;
let envelope = receipt.to_self_describing_envelope();
```

This path is where we make expensive or boundary-crossing decisions visible:

- support lookup is explicit
- source-family binding is explicit
- contract binding is explicit
- extraction is explicit
- envelope derivation is explicit

This path should exist for callers that need planning, inspection, or
certification-level control. It must not be the only ergonomic path.

### Support And Denial DX

Callers must be able to ask what is supported before trying to extract facts.
That should look like a typed inspection surface, not a free-form note in docs.

Representative shape:

```rust
let support = ProjectMaterializedFacts::declare()
    .source(QueryWriteReceiptSource::from(&write_receipt))
    .memberships()
    .support()?;

assert!(support.is_denied());
assert_eq!(
    support.denial_reason(),
    ProjectionConsumptionDenialReason::SourceFamilyDoesNotProveMembership,
);
```

When a request is admitted, admitted-with-warnings, denied, deferred, or
source-mismatched, the caller should see that state as a domain value with
inspectable cause:

```rust
match declaration.evaluate()? {
    ProjectionConsumptionEligibility::Admitted(admitted) => {
        let facts = admitted.bind_contract()?.extract()?;
        use_facts(facts);
    }
    ProjectionConsumptionEligibility::AdmittedWithWarnings(admitted, warnings) => {
        inspect_warnings(&warnings);
        let facts = admitted.bind_contract()?.extract()?;
        use_facts(facts);
    }
    ProjectionConsumptionEligibility::Denied(denied) => {
        inspect_denial(denied.reason(), denied.requested_fact_families());
    }
    ProjectionConsumptionEligibility::Deferred(deferred) => {
        plan_later_milestone_support(deferred.deferred_reason());
    }
    ProjectionConsumptionEligibility::SourceMismatch(mismatch) => {
        correct_source_family(mismatch.expected(), mismatch.actual());
    }
}
```

The API must not collapse these cases into:

- `Option<Facts>`
- `bool` support flags
- stringly `reason` bags without typed posture
- "empty fact set means unsupported"

### Fact Access Shape

The final `ConsumedProjectionFactSet` should read like a semantic object, not
like a payload container.

Required feel:

```rust
let facts = consumed.facts();

facts.entity_identities();
facts.target_identity();
facts.memberships();
facts.relation_endpoints();
facts.labels();
facts.derived_scalars();
facts.view_local_identities();
facts.source_references();
```

Forbidden feel:

```rust
consumed.rows();
consumed.values();
consumed.json();
consumed.fields()["display_name"];
consumed.into_iter().map(|row| row["id"].clone());
```

If the caller truly needs raw lower-runtime source artifacts, that must happen
through a different explicit boundary. The consumed-fact lifecycle is for
Query-owned fact contracts, not for handing raw materialization bags back to
the host.

### Receipt And Envelope Shape

Once facts exist, the operational story should stay compact and obvious.

The code should look like:

```rust
let receipt = consumed.receipt();

receipt.contract_digest();
receipt.source_family();
receipt.extracted_fact_count();
receipt.authority_reopen_count();

let envelope = receipt.to_self_describing_envelope();
let diagnostics = envelope.diagnostics();
```

This should not look like:

- the caller reconstructing envelope meaning from raw fact vectors
- separate ad hoc helper modules that re-summarize the same contract after the
  receipt already exists
- domain code calculating its own "support matrix" from examples

### Source-Family-Specific Entry Points

The code should make source-family ownership visible without forcing every
caller through lower-level crate APIs.

The preferred shape is:

- Query-owned extension methods or facade helpers on:
  - `WORTHQueryReadReceipt`
  - `WORTHQueryWriteReceipt`
  - `QueryContextExecutionArtifact`
- Query-owned adapter constructors for:
  - `RelationalAuthoritativeRowSetArtifact`
  - `RelationalGroupedProjectionArtifact`
  - `BridgeMaterializedRowSetArtifact`
  - `BridgeGroupedTruthViewArtifact`

That means callers should be able to stay in Query for the ordinary path:

```rust
let consumed = read_receipt.consume_projection_facts(
    ProjectMaterializedFacts::declare().entity_identities().labels(),
)?;
```

But the lower-runtime origin should still remain inspectable in advanced code:

```rust
let declaration = ProjectMaterializedFacts::declare()
    .source(RelationalRowSetSource::from(&row_set))
    .entity_identities()
    .labels();
```

### Compile-Time Expectations

The target code shape must make several bad patterns impossible or at least
painfully unnatural:

- a `QueryWriteReceiptSource` declaration should not autocomplete membership
  extraction APIs that only a grouped source family can admit
- a denied or deferred posture should not expose `.bind_contract()`
- a raw `RelationalAuthoritativeRowSetArtifact` should not expose Query fact
  accessors until it is wrapped in a Query declaration/eligibility/contract
  progression
- a caller should not be able to construct `ConsumedProjectionFactSet` or
  `ProjectionConsumptionReceipt` directly in tests
- source-family-specific fact accessors should only appear once the contract
  proves those families are admitted

In other words, autocomplete should teach the lifecycle:

- declaration methods while authoring intent
- eligibility inspection methods while checking support
- contract methods only after admission
- fact accessors only after extraction
- receipt/envelope methods only after facts exist

### Explicitly Forbidden Finished Shapes

Even if they are easy to implement, these outcomes are out of spec:

- `project_materialized_facts(materialized_rows, options)` where `options` is a
  bag of booleans or strings
- `consume_projection_facts()` returning `Vec<Value>`, `Vec<Row>`, or
  `HashMap<String, Value>`
- one generic `ProjectionFact` enum with every family stuffed into a single
  unindexed collection and no family-specific accessors
- helper modules that require the caller to manually pair a receipt, payload,
  and support matrix after extraction
- implicit fallback from denied Query source families to relational or bridge
  re-reads
- host-side recovery of identities or memberships by pattern-matching labels,
  indexes, or positional row fields

If the code still wants to be used that way, the milestone is not finished no
matter how complete the internal proof chain appears.

## Phases

These phases are mandatory sequence, not parallel workstreams and not a menu.
An engineer should be able to complete Phase 1, stop, and know exactly which
new artifact now exists and which later work is still forbidden.

Each phase therefore has four practical obligations:

- define the exact input artifacts it is allowed to consume
- define the exact output artifact it must leave behind
- define the concrete implementation work that happens in that phase
- define the gate that must pass before the next phase may begin

No later phase may be "started in spirit" by sneaking its logic into an
earlier helper. If a phase needs a capability from a previous phase, that
previous artifact must exist as a named public or crate-visible structure
first.

### Phase 1: Declare Fact Consumption Intent

Purpose:
Freeze caller intent before any support decision, contract binding, or fact
extraction exists.

Allowed inputs:

- canonical query/result-shape identity already produced by existing Query
  authoring/runtime surfaces
- authorized projection identity from Milestone 9
- admitted materialization basis posture from Milestone 9.3.2
- one concrete source family from the `Initial Source-Family Lock`

Required output:

- `ProjectionConsumptionDeclaration`

Engineer work in this phase:

- create the declaration module and public authoring surface
- make fact-family selection explicit and intention-revealing
- make source-family selection explicit where the source is not already implied
  by the entry point
- bind the declaration to canonical query identity, result-shape identity,
  authorized projection identity, and admitted basis/materialization posture
- assign the declaration its canonical digest
- wire facade entry points so ordinary callers can begin from Query-owned
  surfaces instead of inventing local adapters

The code produced in this phase should let an engineer write these declarations
and nothing more:

```rust
let declaration = read_receipt
    .declare_projection_fact_consumption(
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .labels(),
    )?;
```

```rust
let declaration = ProjectMaterializedFacts::declare()
    .source(RelationalGroupedProjectionSource::from(&grouped_projection))
    .memberships()
    .relation_endpoints()
    .view_local_identities();
```

This phase is not allowed to:

- inspect source payload rows
- decide support or denial
- bind a materialized contract
- extract facts
- shape receipts or envelopes

Completion gate for Phase 1:

- declarations exist as named artifacts with private construction internals
- source-family choice is represented structurally, not as strings
- the public facade can author declarations for the first-slice source families
- no declaration API accidentally exposes fact extraction or receipt methods

Required crate work:

- `worth-query`: add declaration authoring and source-family selection APIs in
  the new `projection_consumption/declaration` subdomain and export them
  through the facade
- `worth-runtime-bridge`: no new declaration surface; Query declarations that
  target bridge-backed sources must reference existing bridge source artifacts
  only
- `worth-relational`: no new declaration surface; relational meaning stays
  behind existing authoritative row-set/grouped-projection artifacts
- `worth-signal`: no changes

### Phase 2: Admit Or Deny The Requested Fact Families

Purpose:
Turn a declaration into an explicit support decision before any contract or
fact set can exist.

Allowed inputs:

- `ProjectionConsumptionDeclaration`
- existing authorized-projection visibility evidence
- existing basis/materialization capability evidence
- source-family capability/provenance exposed by the declared source artifact

Required output:

- `ProjectionConsumptionEligibility`
  or one of its typed postures:
  `AdmittedProjectionConsumption`,
  `AdmittedProjectionConsumption + ProjectionConsumptionWarnings`,
  `DeniedProjectionConsumption`,
  `DeferredProjectionConsumption`,
  `SourceMismatchedProjectionConsumption`

Engineer work in this phase:

- implement eligibility evaluation in one place rather than scattering checks
  across source adapters
- prove visibility against `AuthorizedProjectionArtifact` and
  `PolicyFieldInfluenceSet`
- prove basis validity against the 9.3.2 materialization lane
- prove source-family compatibility against the declaration
- distinguish "not proven by this source" from "masked or not visible" from
  "deferred by roadmap"
- model admitted, admitted-with-warnings, and non-admitted results as distinct
  typed outcomes

Hostile lanes that must be implemented and named in tests during this phase:

- masked field or hidden influence requested as a consumed fact
- membership/shape facts requested from a source family that does not prove
  them
- branch/preview/historical source mismatch
- effect receipt family that has not yet materialized the required projection
  evidence
- stale or policy-drifted materialization trying to satisfy a fresh
  declaration
- a caller attempting to consume grouped membership facts from an ordinary
  detail read source
- a caller attempting to treat row identity as entity identity without an
  admitted identity fact family

This phase is not allowed to:

- bind a materialized projection contract
- extract facts "because support was obvious anyway"
- create an empty fact set as a substitute for denial
- let source adapters re-decide policy visibility locally

Completion gate for Phase 2:

- every first-slice source/fact-family combination evaluates into a typed
  admitted, admitted-with-warnings, denied, deferred, or source-mismatch
  posture
- denial reasons are typed and inspectable
- there is still no public path from declaration directly to facts without
  passing through eligibility
- hostile fixtures cover the required denial and mismatch lanes

Required crate work:

- `worth-query`: implement eligibility, warning-bearing admission, denial,
  deferred, and
  source-mismatch proof families in `projection_consumption/eligibility`
- `worth-runtime-bridge`: no new eligibility logic; bridge support is consumed
  through existing source/writeback artifacts and their current facade reach
- `worth-relational`: no new eligibility logic; relational authority is
  consumed through existing row-set/grouped artifacts
- `worth-signal`: no changes

### Phase 3: Bind One Materialized Projection Contract

Purpose:
Freeze one admitted interpretation of one declaration over one concrete
materialization source so extraction later has no authority to improvise.

Allowed inputs:

- admitted eligibility output from Phase 2
- the exact source artifact named by the declaration and admitted by
  eligibility

Required output:

- `MaterializedProjectionContract`

Engineer work in this phase:

- create the contract type and binder
- normalize equivalent admitted declarations over equivalent materializations
  into the same contract meaning
- bind every digest and source identity needed for later extraction, receipt
  shaping, inspection, and certification
- define the explicit source posture family so Query-owned, relational, and
  bridge-backed sources cannot masquerade as each other

The contract must bind all of the following:

- canonical query digest
- canonical result-shape digest
- authorized projection identity and narrowed result-shape digest
- admitted basis/materialization receipt digests
- source materialization family and source receipt/envelope digest
- policy digest and tenant/schema digest where applicable
- fact-family inventory and support posture
- any required source-reference identities for later inspection

The contract must also classify one source artifact posture:

- `QueryOwnedReceiptSource`
- `RelationalAuthoritySource`
- `BridgeAuthoritySource`

This posture is not cosmetic. It must be used by compile-fail boundaries and
proof-shape audits so one source family cannot silently masquerade as another.

Equivalent declarations over equivalent materializations must normalize to the
same contract digest. Intentionally different fact families, source families,
policy basis, result-shape meaning, or view-shape meaning must change the
relevant contract fields and digest.

This phase is not allowed to:

- inspect raw payloads to derive consumed facts
- hide source-family posture inside an erased helper object
- let extraction begin from admitted eligibility alone

Completion gate for Phase 3:

- admitted eligibility is the only public path into contract binding
- contracts carry explicit source posture and all required digests
- contract equality and difference rules are covered by normalization tests
- lower-crate reachability gaps, if any, have been resolved by narrow facade
  additions rather than Query-side authority recreation

Required crate work:

- `worth-query`: implement contract binding in
  `projection_consumption/contracts`, including explicit source-family binding
  to Query, bridge, or relational evidence already exposed through current
  public surfaces
- `worth-runtime-bridge`: if Query cannot bind one bridge-backed source family
  without reopening bridge internals, expose one additional facade accessor for
  that already-authoritative source artifact and nothing broader
- `worth-relational`: if Query cannot bind the first grouped/topology hostile
  lane from `RelationalAuthoritativeRowSetArtifact` or
  `RelationalGroupedProjectionArtifact`, expose one additional facade accessor
  for that authoritative artifact and nothing broader
- `worth-signal`: no changes

### Phase 4: Extract Typed Consumed Fact Sets

Purpose:
Materialize the actual Query-owned consumed facts from one already-frozen
contract and nothing weaker.

Allowed inputs:

- `MaterializedProjectionContract`
- the exact source artifact family bound into that contract

Required output:

- `ConsumedProjectionFactSet`

Engineer work in this phase:

- expose a typed consumed-fact artifact rather than a raw row/value bag
- guarantee that every fact in the set traces back to one admitted fact family
  and one source proof
- preserve source distinction between read materialization, effect aftermath,
  query-context payload, and other admitted source classes
- make absent, denied, deferred, warning-bearing, and source-mismatch
  neighbors inspectable without
  pretending they are equivalent to admitted facts

The fact set must remain mechanically queryable without exposing raw payload
bags. At minimum it must support:

- `entity_identities()`
- `view_local_identities()`
- `memberships()`
- `labels_or_display_values()`
- `derived_scalar_facts()`
- `source_references()`

Those names may evolve, but the accessor split is load-bearing. A single
`facts()` iterator of untyped values is out of spec.

The implementation order inside this phase should be practical:

1. extract identity/display/scalar families for the simplest admitted sources
2. add write-receipt-backed aftermath/source-reference extraction
3. add grouped/topology extraction from admitted grouped sources
4. add query-context-backed narrow fact extraction only after the simpler
   source families are behaving honestly

This phase is not allowed to:

- call back into relational truth, bridge internals, signal internals, or
  domain caches to "fill in" missing facts
- widen collection/topology membership by host-side scans unrelated to the
  admitted materialization
- let source-family adapters silently reinterpret materialized rows under a
  different fact taxonomy

Exact counters must capture at least:

- declared fact family count
- admitted fact family count
- extracted fact count
- source row width consumed
- source evidence lookup width
- authority reopen count

The last counter is load-bearing: any runtime path that reopens source
authority after the admitted materialization should increment it, and the
certification suite should require it to remain zero for admitted 9.3.4 lanes.

Completion gate for Phase 4:

- fact extraction starts only from `MaterializedProjectionContract`
- each first-slice admitted fact family is reachable through typed accessors
- raw payload bags are no longer the practical public API for these lanes
- exact counters, including `authority_reopen_count`, are emitted for the new
  extraction path

Required crate work:

- `worth-query`: implement fact extraction adapters in
  `projection_consumption/sources` for:
  - Query read receipts
  - Query write receipts
  - query-context execution artifacts
  - admitted bridge-backed grouped/materialized sources that are already facade
    reachable
  - admitted relational row-set/grouped sources that are already facade
    reachable
- `worth-runtime-bridge`: no Query-shaped fact extraction types; only existing
  bridge source artifacts or one narrowly added accessor if Phase 3 proved a
  public reachability gap
- `worth-relational`: no Query-shaped fact extraction types; only existing
  authoritative row-set/grouped artifacts or one narrowly added accessor if
  Phase 3 proved a public reachability gap
- `worth-signal`: no changes

### Phase 5: Shape Receipt, Envelope, And DX Surfaces

Purpose:
Turn extracted facts into the operational surface downstream code will actually
hold, inspect, and pass around.

Allowed inputs:

- `ConsumedProjectionFactSet`

Required outputs:

- `ProjectionConsumptionReceipt`
- `SelfDescribingProjectionConsumptionEnvelope`
- common-path facade helpers that lower through the earlier phases rather than
  bypassing them

Engineer work in this phase:

- define `ProjectionConsumptionReceipt` as the canonical operational artifact
  for downstream consumed-fact use
- derive `SelfDescribingProjectionConsumptionEnvelope` from the receipt rather
  than constructing a separate fact source
- provide common-path APIs for ordinary callers and compile-checked golden DX
  transcripts for:
  - read-backed materialized fact consumption
  - effect-backed materialized fact consumption
  - support/discovery before consumption
  - typed denial/deferred handling
  - envelope/inspection after receipt
- preserve explicit expensive-work boundaries so receipt/envelope derivation or
  advanced inspection cannot masquerade as a cheap field access

Receipt transition rules should be explicit, not ambient. At minimum:

- receipt inspection is implemented
- self-describing envelope derivation is implemented
- support/discovery over the consumed-fact lane is implemented
- persisted receipt reload, store-backed reconstruction, and portable receipt
  export remain typed deferred until later milestones

The receipt must name at least these practical fields:

- contract digest
- source family
- source artifact identity/digest
- admitted fact-family count
- extracted fact count
- denied/deferred neighbor summary
- authority reopen count
- integrity digest

The implementation order inside this phase should be:

1. issue the canonical receipt from facts
2. derive the self-describing envelope from the receipt
3. add common-path helpers on read/write/context-facing Query surfaces
4. add support/discovery and inspection helpers
5. lock in compile-checked golden DX transcripts

This phase is not allowed to:

- create a second parallel receipt vocabulary
- let common-path helpers skip eligibility or contract binding internally
- make envelope derivation a hidden side effect of field access

Completion gate for Phase 5:

- downstream code can use a short common path for each first-slice admitted
  source family
- the advanced path still exposes explicit declaration, eligibility, contract,
  extraction, and receipt/envelope steps
- receipt and envelope fields cover the practical operational needs named in
  this spec
- golden transcript tests prove the intended caller ergonomics compile

Required crate work:

- `worth-query`: implement receipt/envelope shaping, DX helpers, target
  transcripts, and inspection support in `projection_consumption/receipts` and
  `projection_consumption/dx`
- `worth-runtime-bridge`: no new receipt surface; Query receipts may reference
  bridge source/writeback identities but do not replace bridge receipts
- `worth-relational`: no new Query receipt surface; relational stays
  authoritative for underlying truth artifacts only
- `worth-signal`: no changes

### Phase 6: Close Public Boundaries, Support, And Certification

Purpose:
Prove the finished lifecycle is structurally closed before later milestones are
allowed to build on it.

Allowed inputs:

- the complete declaration -> eligibility -> contract -> fact set ->
  receipt/envelope lifecycle from Phases 1 through 5

Required output:

- `ProjectionConsumptionCertificationBundle`
- public-boundary, proof-shape, and DX closure evidence sufficient to close the
  milestone

Engineer work in this phase:

- define support matrices and source/fact family inventories derived from
  executable admission facts
- audit public boundaries so external callers cannot mint contracts, fact sets,
  receipts, envelopes, or certification rows directly
- provide proof-shape and phase-progression audits proving weaker artifacts
  cannot cross into later phases
- certify at least one hostile topology/Worth-style lane without making the
  design topology-specific
- bind closeout DX, oracle comparisons, and exact counter snapshots into one
  certification bundle

The closeout bundle must reject:

- source-family overclaim
- hidden/masked fact resurrection
- generic row-summary substitution for required output digests
- performance claims without exact counter/slope evidence

The implementation order inside this phase should be:

1. lock support matrices and inventories to executable behavior
2. add compile-fail boundary fixtures
3. add proof-shape and phase-progression audits
4. certify the hostile grouped/topology/Worth-style lane
5. emit the closeout certification bundle with counter snapshots and digests

This phase is not allowed to:

- rely on narrative explanation instead of executable proof
- claim support for a source/fact lane that is not present in the support
  matrix and hostile tests
- treat a passing happy-path demo as certification

Completion gate for Phase 6:

- certification artifacts, compile-fail fixtures, support matrices, DX
  transcripts, and hostile oracle checks all agree
- no external caller can mint later-phase artifacts directly
- the milestone can be handed to 9.3.5 and later work as one closed public
  lifecycle rather than a partially social convention

Required crate work:

- `worth-query`: implement inventories, support matrices, proof-shape audits,
  public-boundary audits, certification bundles, compile-fail fixtures, and
  hostile topology/Worth certification
- `worth-runtime-bridge`: no certification ownership change; bridge remains a
  source/oracle input to Query certification where applicable
- `worth-relational`: no certification ownership change; relational remains an
  oracle/source input to Query certification where applicable
- `worth-signal`: no changes

## Required Topology

Milestone 9.3.4 should map into responsibility-specific subdomains. The exact
file names may follow local crate conventions, but the boundaries must remain
structurally honest.

Required subdomains:

- `projection_consumption/facts`
  - owns fact-family taxonomy, typed fact artifacts, and family inventories
  - does not inspect lower-runtime state directly
- `projection_consumption/declaration`
  - owns declaration authoring, canonical identity, and source/fact intent
  - does not extract facts
- `projection_consumption/eligibility`
  - owns admitted/denied/advisory/deferred/source-mismatch resolution
  - does not shape receipts or envelopes
- `projection_consumption/contracts`
  - owns materialized projection contract binding and identity
  - does not become a fact store
- `projection_consumption/sources`
  - owns adapters from read materialization, effect receipts, query-context
    payloads, and other admitted source families
  - does not reopen authority beyond the source artifacts explicitly admitted by
    the contract
- `projection_consumption/receipts`
  - owns canonical operational receipt construction and transition rules
  - does not perform source admission
- `projection_consumption/dx`
  - owns common-path helpers, fluent APIs, and golden transcript support
  - does not weaken proof boundaries for convenience
- `projection_consumption/support`
  - owns support matrices, inventories, and deferred-neighbor truth
  - does not infer support from successful examples alone
- `projection_consumption/certification`
  - owns audits, certification bundles, oracles, slope reports, and compile-fail
    fixtures
  - does not reuse free-form logs as proof

Forbidden topology:

- one broad `projection.rs` or `materialization.rs` bucket that mixes fact
  taxonomy, source adapters, contract binding, receipt shaping, DX, and
  certification
- source-specific public consumed-fact APIs that bypass the shared lifecycle
- host/test helper modules that become the de facto source of fact-family truth
- certification fixtures that hide which responsibility failed behind mutable
  global harness state
- burying source-family distinctions inside generic helper traits whose concrete
  implementations are not visible from the owning subdomain

## Must Ship

- declared projection-consumption artifacts for materialized read, effect
  aftermath, query-context, and inspection-adjacent source families where
  admitted
- typed consumed-fact families for identity, relation endpoint, membership,
  label/display, derived scalar, shape, and view-local identity lanes
- admitted, denied, advisory, deferred, and source-mismatch eligibility
  postures
- materialized projection contracts binding query, result shape, authorized
  projection, basis, policy, tenant/schema, source receipt, and fact-family
  inventory
- proof-bearing consumed fact sets plus canonical receipts and self-describing
  envelopes
- support matrices, source/fact family inventories, DX transcripts, proof-shape
  audits, public-boundary audits, and certification bundles
- topology/Worth-style hostile certification as the first nontrivial consumer
  lane, without baking topology-specific semantics into the shared lifecycle
- the explicit crate-boundary implementation split described in
  `Required Crate Changes`

## Must Preserve

- relational owns authoritative truth; materialized projections remain derived
- Query owns projection contracts, consumed-fact declarations, receipt shaping,
  and support/certification
- authorized projection from Milestone 9 remains the visibility authority for
  fact consumption under policy
- materialization basis from Milestone 9.3.2 remains the capability precondition
  for projection consumption
- effect receipts from Milestone 9.3.3 remain operational sources, not excuse
  to rediscover authority locally
- future workflow, geometry, table, design, and temporal projections reuse the
  same lifecycle instead of adding local lookup helpers
- `worth-runtime-bridge`, `worth-relational`, and `worth-signal` do not become
  the owner of Query consumed-fact declarations, contracts, or receipts

## Acceptance Evidence

This milestone is complete only when a hostile certification program can:

- declare the projection facts it intends to consume
- bind those facts to one admitted materialization contract
- obtain one typed fact set plus one receipt/envelope story
- prove equivalent declarations/materializations normalize to the same consumed
  fact meaning
- prove different fact/source/policy/basis choices change the relevant digests
- avoid direct source-authority reads for fact discovery
- reject unsupported, masked, stale, or source-mismatched fact requests before
  a fact set exists

## Required Verification Output

The 9.3.4 certification bundle must emit:

- `query_digest`
- `result_shape_digest`
- `authorized_projection_digest`
- `materialization_basis_digest`
- `projection_consumption_declaration_digest`
- `projection_consumption_eligibility_digest`
- `materialized_projection_contract_digest`
- `consumed_projection_fact_set_digest`
- `projection_consumption_receipt_digest`
- `projection_consumption_envelope_digest`
- `projection_source_digest`
- `projection_source_receipt_digest`
- `projection_fact_family_inventory_digest`
- `projection_support_matrix_digest`
- `projection_public_surface_digest`
- `projection_target_dx_digest`
- `projection_golden_transcript_digest`
- `projection_proof_shape_digest`
- `projection_phase_progression_digest`
- `projection_transition_rules_digest`
- `projection_oracle_digest`
- `seeded_sequence_digest`
- `seed_replay_digest`
- `compile_fail_boundary_digest`
- `failure_digest`
- `counter_snapshot`
- `authority_reopen_count`
- `fact_extraction_width`
- `projection_declaration_slope_digest`
- `projection_eligibility_slope_digest`
- `projection_contract_binding_slope_digest`
- `projection_fact_extraction_slope_digest`
- `projection_receipt_materialization_slope_digest`
- `projection_envelope_materialization_slope_digest`
- `projection_support_lookup_slope_digest`

## Architectural Notes

- A consumed projection fact is not authoritative truth. It is a Query-owned
  derived fact contract bound to one admitted materialization.
- Materialized rows are not the public lifecycle. They are source evidence that
  the consumed-fact lifecycle may adapt, deny, or defer.
- Authorized projection is the visibility authority for materialized fact
  consumption under policy. No consumed fact may bypass it by re-reading hidden
  fields indirectly.
- Basis capability is a precondition, not a helper detail. Projection
  consumption must consume the materialization lane from Milestone 9.3.2 rather
  than accepting raw branch/snapshot/preview/history identifiers.
- Effect receipts are operational sources, not substitute authority. 9.3.4 may
  consume them to prove what materialized aftermath exists, but it may not
  reinterpret execution authority or replay lower-runtime protocol locally.
- A consumed-fact contract must bind its source family. "Same shape, different
  source" is semantically meaningful because source families prove different
  fact sets and support/deferred neighbors.
- Denied, warning-bearing admission, deferred, and source-mismatch
  projection-consumption
  outcomes must be different proof families from admitted contracts and fact
  sets.
- Digests bind explanations and certification, but executable APIs must consume
  proof-bearing contract/fact/receipt types rather than raw digests.
- Support matrices must be derived from executable admission behavior. A
  hand-maintained spreadsheet of supported fact families is not the source of
  truth.
- Golden DX transcripts are part of the proof. If ordinary callers still need
  local cache lookups, host-side row parsing, or lower-runtime inspection to use
  consumed facts naturally, the milestone is unfinished.

## Deferred Scope

Runtime-backed projection consumption is not blocked on `worth-store`.

The following remain explicitly deferred:

- persisted projection-consumption receipts
- durable projection reload from stored consumed-fact receipts
- store-backed reconstruction parity for consumed fact sets
- restart-stable projection-consumption envelopes
- portable consumed-fact receipt import/export
- temporal fact consumption, async/resource fact lanes, and time-only fact
  surfaces owned by Milestones 9.4 through 9.7

Any 9.3.4 surface that encounters those families must report typed deferred or
unsupported posture.

## Sequencing Notes

Milestone 9.3.4 belongs after 9.3.3 because consumed projection facts must bind
to already-honest materialization sources, including effect receipts and
envelopes, rather than reopening execution authority to discover what changed.

It belongs after 9.3.2 because materialization basis admission is already the
typed permission boundary for consuming a projection from current, branch,
preview, or historical truth.

It belongs before 9.3.5 because the later decision lattice should consume one
honest projection-consumption lifecycle instead of unifying over ad hoc local
fact lookups.

It belongs before 9.3.6 because lower-runtime capability routing needs to know
which source family and receipt actually proved the consumed facts before it can
route or certify boundary contact honestly.

It belongs before the Runtime API Public Stabilization Gate because downstream
domain runtimes need one stable answer to "which projection facts may I consume
from this materialization?" before the public facade freezes.

## Closeout Standard

This milestone may close only when:

- the 9.3.4 phases have been implemented in order, or any production deviation
  was first approved by a spec amendment
- every admitted consumed-fact lane binds to an authorized projection identity,
  admitted materialization basis, and source materialization receipt/envelope
- equivalent declaration/materialization paths normalize to the same contract,
  fact set, and receipt meaning
- intentionally different fact families, source families, basis postures,
  policy postures, or result-shape meaning change the relevant digests
- unsupported, masked, stale, deferred, warning-bearing, and source-mismatch
  lanes
  fail typed before a consumed fact set exists
- compile-fail boundaries prove public callers cannot construct admitted
  contracts, fact sets, receipts, envelopes, inventories, or certification rows
  directly
- any lower-crate changes that were needed remained narrow facade additions for
  already-authoritative artifacts rather than ownership drift of Query
  declaration/receipt logic
- support metadata, executable behavior, DX transcripts, and certification
  coverage agree for admitted, admitted-with-warnings, denied, deferred, and
  source-mismatch
  families
- exact counters and slope digests prove projection consumption costs are
  bounded by declared fact width, source evidence width, and materialized row
  width rather than unrelated runtime graph breadth or host cache size
- roadmap and test-requirement references point at this spec and its named
  certification suite accurately

## Self-Check

- Does the milestone solve a real structural problem or just package work
  cosmetically? Yes: it closes the gap between receiving a materialized
  projection and legally consuming facts from it without authority leaks.
- Is the adversarial constraint precise and load-bearing? Yes: it targets the
  exact failure where consumers reopen truth, preview state, signal state, or
  host caches after materialization to rediscover facts.
- Does the milestone preserve crate authority boundaries? Yes: Query owns the
  consumed-fact lifecycle while relational, bridge, signal, and store retain
  their respective authority surfaces.
- Does the milestone define proof obligations, not just implementation tasks?
  Yes: declaration identity, typed eligibility, contract binding, source/fact
  parity, compile-fail boundaries, oracles, DX closure, and exact slope
  counters are all explicit requirements.
- Could a competent engineer map this spec into honest types, modules, and
  tests? Yes: the phase progression names the required artifacts, topologies,
  denials, receipts, envelopes, inventories, and certification outputs.
- Does the milestone belong in this roadmap sequence, or is it out of order?
  Yes: it consumes the basis and effect seams already closed by 9.3.2/9.3.3 and
  prepares a stable fact-consumption contract for 9.3.5, 9.3.6, and the public
  runtime API freeze.
