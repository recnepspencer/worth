# Milestone 7 Engineering Spec: Reactive Source Protocol And Clean Host Surfaces

> **Status:** Completed. Closeout: [milestone-7-closeout.md](./milestone-7-closeout.md).
>
> **Roadmap parent:** [worth_runtime_bridge_roadmap.md](./WORTH_runtime_bridge_roadmap.md)
>
> **Vision parent:** [worth_runtime_bridge_vision.md](./WORTH_runtime_bridge_vision.md)
>
> **Prior milestone:** [milestone-6.md](./milestone-6.md)
>
> **Bridge certification companion:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make truth-backed reads a first-class bridge protocol and construction surface so hosts and compute consumers stop learning relational storage details, capability folklore, and builder-order quirks directly
>
> **Companion docs:**
> - [worth_relational_roadmap.md](../WORTH-relational/WORTH_relational_roadmap.md)
> - [worth_signal_vision.md](../WORTH_signal/WORTH_signal_vision.md)
> - [worth_signals2.md](../WORTH_signal/WORTH_signals2.md)
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [architectural_guidelines.md](../../docs/coding-guidelines/arch_laws.md)
> - [domain_standards.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [performance_guidelines.md](../../docs/coding-guidelines/perf_laws.md)

## Summary

Milestones 1 through 6 established that the bridge already has strong
write-side and change-consumption authority:

- canonical committed truth enters through one bridge-owned envelope path
- routing, continuity, historical selection, and bulk planning lower through
  explicit proof chains
- stream consumption is now a protocol surface rather than a host-shaped feed
- replay, checkpoint, and diagnostics already have canonical bridge artifacts

That is enough to make the bridge authoritative about how truth changes are
consumed.

It is not enough to make the bridge authoritative about how truth-backed reads
are declared, admitted, and materialized.

Without Milestone 7, the bridge still risks a weaker and more dangerous read
story:

- signal-facing code learns relational storage capabilities directly
- host adapters expose whatever read surfaces they happen to support rather
  than one bridge-owned contract
- builder setup becomes a bag of source knobs whose meaning depends on order or
  host convention
- unsupported historical, branch, or facet reads fail late during
  materialization instead of early during admission
- source diagnostics describe host accidents rather than canonical bridge truth

Milestone 7 exists because Milestone 6 completed only half of the public
protocol boundary. The bridge now has a protocol-grade change surface. It must
gain a protocol-grade read surface that is equally explicit, replay-safe, and
host-agnostic.

The bridge must be able to say:

`this exact source declaration, under this exact capability set and truth-view basis, admitted this exact packetized read contract, materialized through this exact host adapter seam, and produced this exact canonical truth-view result`

not:

`the bridge reached into a relational adapter, asked for whatever read mode happened to exist, and callers learned the rest by folklore`

## Goal

Make truth-backed reads a deterministic, capability-explicit, replay-safe
bridge protocol with clean builder/configuration surfaces and narrow host
adapter seams.

## Why This Milestone Exists

Milestone 7 belongs immediately after Milestone 6 because change-stream
contracts and read-source contracts are the two halves of the same bridge
boundary.

Milestone 6 established:

- canonical stream-member identity
- explicit checkpoint, replay, resume, and coalescing semantics
- a bridge-owned protocol vocabulary for truth change consumption

Milestone 7 now needs to establish the matching read-side truths:

- canonical source declaration identity
- explicit truth-view capability admission
- explicit packetized source materialization semantics
- a bridge-owned protocol vocabulary for truth-backed reads

If Milestone 7 shipped before Milestone 6, the bridge would have a polished
read surface attached to a host-shaped write/change surface. That would be an
asymmetric protocol boundary and therefore a weak one.

Milestone 7 also belongs before Milestone 8 because structural-identity-aware
remapping will rely on a trustworthy source contract for branch and historical
comparison. Structural reuse cannot be honest if the underlying read surface is
still adapter-shaped and capability-ambient.

Milestone 7 therefore earns its place in the roadmap by solving the next real
structural problem after protocol-grade change consumption: protocol-grade
truth-backed reads and clean public construction surfaces.

## Adversarial Constraint

Milestone 7 must survive the following hostile condition:

> A long-lived system with multiple host adapter shapes, snapshot reads,
> historical reads, branch-local reads, admitted field or facet reads,
> diagnostics tiers that vary by environment, replay after restart, builder
> setup performed in different explicit orders, and future remapping and policy
> work layered on top must admit the same declared bridge source contract into
> the same truth-view plan and packetized materialization every time, while
> keeping truth storage internals out of signal-facing code, while rejecting
> unsupported source capabilities before read execution begins, and while
> preventing host adapter quirks or flat builder flags from redefining bridge
> meaning.

Concretely, the design must remain correct when all of the following are true:

- one host exposes snapshot and historical reads through one combined source
  object while another host exposes narrower adapter seams
- one request needs only current snapshot reads while another needs branch and
  historical truth-view selection
- admitted field or facet reads exist for some source registrations but not
  others
- builder setup order differs across equivalent hosts
- diagnostics richness changes between environments
- replay occurs from canonical bridge source and truth-view artifacts after
  restart
- later milestones will add structural remapping, merge-bearing histories, and
  policy propagation, but Milestone 7 must not fake those semantics early

If any supported path:

- lets signal-facing code depend on relational storage internals directly
- lets host adapters widen or narrow capabilities without bridge admission
- performs scalar ad hoc reads where a canonical packetized source plan already
  exists
- treats builder order as semantic input
- discovers unsupported source modes during materialization instead of during
  contract admission
- cannot explain which source declaration, truth-view basis, packet identity,
  or adapter capability basis produced a read result

then Milestone 7 has failed.

## Product Decision Lock

The following decisions are explicit and not open questions for this milestone:

- reactive source contracts are a first-class bridge protocol, not convenience
  wrappers over `SnapshotReadSource` or host-local relational facades
- source declaration identity, capability identity, truth-view selection
  identity, and materialization identity are distinct concepts and must remain
  distinct types
- every admitted source request must bind to one explicit truth-view authority
  basis; ambient "latest enough" reads are not admitted in Milestone 7
- unsupported source capabilities fail during admission, not late during read
  execution
- packetized read planning is the authority for broad source materialization;
  scalar convenience reads remain layered on top
- builder/configuration surfaces must mirror subsystem boundaries rather than
  becoming a flat bag of read flags
- duplicate or conflicting source/builder declarations must fail through one
  canonical bridge rule set rather than inheriting host-specific precedence
- host integration is through bridge-owned narrow adapter traits, not broad
  reach-through into parent runtime internals
- diagnostics richness may change retained explanation, but not admitted source
  truth or read results
- Milestone 7 productizes read/source contracts and setup surfaces only; it
  does not productize merge-aware source semantics, policy propagation, or
  writeback

Normative consequence:

- public bridge APIs that expose raw relational storage handles to signal code
  are out of spec
- builder methods whose meaning depends on call ordering are out of spec
- host-specific capability fallback outside the bridge source contract is out
  of spec
- materialization paths that discover unsupported read modes after packet
  planning are out of spec
- host adapters that reshape, reorder, widen, or coalesce bridge-planned source
  packets are out of spec
- diagnostics-only descriptions of source mismatch without typed bridge
  failures are out of spec

## Guideline Influence

This section is mandatory. It states exactly how each coding-guideline document
changes the shape of this milestone rather than merely being listed as context.

### 1. `MENTALITY.md`

This document drives the design stance of the entire milestone:

- adversarial constraint first:
  the spec starts from hostile multi-host, multi-mode truth-backed reads rather
  than from a pleasant "let signals read relational data" feature description
- solve the hard problem first:
  Milestone 7 productizes capability admission, truth-view selection, and
  clean construction surfaces before any later read-side convenience or
  remapping work
- enforce mechanically, not by convention:
  source capability mismatch, truth-view admission, and builder completeness
  must be represented by proof-bearing types and typed failures rather than by
  comments or host discipline
- spec is architecture is code:
  the spec names concrete subdomains, proof phases, and API surfaces that must
  map honestly into crate modules and types
- authority first, derivation second:
  truth runtime remains the authority for truth views; bridge source artifacts,
  read packets, and diagnostics are derived protocol artifacts
- separate what/how/whether:
  the truth view is the `what`, source materialization is the `how`, and rich
  diagnostics are the `whether`; diagnostics may never redefine read meaning

### 2. `architectural_guidelines.md`

This document determines the structural boundaries of the milestone:

- Law 1:
  the bridge source subsystem must be an autonomous facade-owned subsystem, not
  a few extra methods hung directly off host adapters
- Laws 7, 20, and 32:
  every source boundary crossing must produce self-describing packets and
  counters so callers can see what truth-view work happened and what it cost
- Laws 16 and 34:
  source registration and lifecycle must begin from one declaration surface
  that the framework owns, rather than scattered host setup calls
- Laws 18, 30, and 41:
  source observation must be phase-typed, and phase outputs must be proof
  bearing: declared, validated, admitted, planned, materialized
- Laws 21 and 33:
  read truth and diagnostics artifacts have separate lifecycles, and derived
  source diagnostics must remain destroyable and rebuildable from authoritative
  read inputs and bridge records
- Law 29:
  abstraction must stop before it hides cost or correctness boundaries, so the
  public source protocol cannot pretend snapshot, historical, branch, and facet
  reads all have identical cost or support guarantees

### 3. `domain_standards.md`

This document determines how the milestone must decompose inside the crate:

- the source system must be organized by subdomain responsibility such as
  `source/declaration`, `source/contracts`, `source/planning`, and
  `source/materialization`, not by generic `helpers` or `utils`
- the bridge must preserve one public `facade` boundary while keeping source
  internals private
- builder cleanup is not allowed to become one large `builder.rs` catch-all
  responsibility; source registration, diagnostics wiring, policy wiring, and
  adapter wiring must remain structurally distinct responsibilities
- tests must mirror source responsibilities rather than collapsing all read
  scenarios into one generic integration file

### 4. `performance_guidelines.md`

This document determines the cost model of the milestone:

- packetized source reads must be authoritative so broad truth-view requests do
  not degrade into scalar N+1 read orchestration
- capability rejection must precede expensive materialization, which is why
  unsupported historical, branch, or facet reads fail during admission
- locality and boundary honesty matter, so the source API must reveal when a
  request implies packetization, historical access, or branch-local work
- counters must explain breadth and mode:
  source request count, packet count, capability rejection count, fallback
  count, and materialization breadth are part of the contract
- builder/config cleanup is also a performance concern because flat ambient
  knobs create path conflation and hidden richness or breadth costs

## Scope

### In Scope

- one bridge-owned source declaration surface
- canonical source declaration identity, capability identity, truth-view
  selection identity, and materialization identity
- bridge-owned capability admission for snapshot, historical, branch, and
  admitted field or facet reads
- packetized source planning for broad truth-view materialization
- clean builder and configuration entrypoints for source registration, adapter
  wiring, policy wiring, and diagnostics selection
- explicit host adaptation seams that keep platform-specific code outside the
  bridge core
- typed diagnostics and counters for source mismatch, unsupported capability,
  packet breadth, materialization mode, and builder configuration mismatch
- harness certification for multi-host source parity, capability rejection, and
  builder-swap parity

### Explicitly Out Of Scope

- merge-aware source semantics over multi-parent histories
- structural-identity-aware remapping logic
- speculative preview lifecycle
- policy provenance across runtimes beyond the minimal source registration and
  diagnostics wiring needed here
- bridge-mediated writeback or source-driven commit strategies
- scheduler-owned downstream execution semantics inside `worth-signal`

Milestone 7 must stay focused on read/source contracts and clean host surfaces,
not absorb later merge, policy, or writeback milestones.

## Governing Design Rules

### 1. Truth Owns Truth Views, Bridge Owns Source Contracts

The truth runtime defines:

- what snapshots, histories, branches, fields, and facets actually mean
- what truth views are retained or reconstructable
- what read capabilities a host adapter can honestly provide

The bridge defines:

- the public source declaration surface
- canonical source declaration and capability identities
- truth-view admission and packet planning vocabulary
- source diagnostics and replay-safe source records

Signal and other downstream consumers define:

- what they do with admitted read results after materialization

The bridge must not redefine truth semantics or storage authority.

### 1.1 One Declaration Surface Must Begin Every Read Story

Milestone 7 must not allow hosts or consumers to assemble truth-backed read
behavior from scattered methods and ambient flags.

There must be one bridge-owned declaration surface that states:

- what kind of source is being declared
- what truth-view modes are requested
- what field or facet shapes are requested where admitted
- what diagnostics and policy surfaces are attached

This declaration surface is the only public starting point for source
admission.

### 1.2 Truth-View Authority Basis Must Be Explicit

Milestone 7 must not admit source requests that implicitly mean "whatever truth
view is current when the host gets around to reading it."

Every admitted source contract must bind to one explicit authority basis.

For Milestone 7 the closed authority-basis vocabulary must include:

- `ExplicitSnapshot`
- `ExplicitHistoricalVersion`
- `ExplicitBranchHead`

Rules:

- the authority basis is part of source-contract identity
- packet planning and replay records must preserve the authority basis
- later freshness or stream-coupled read modes may extend this vocabulary, but
  they are not ambiently admitted in Milestone 7
- a host adapter may not substitute a different authority basis during
  materialization

### 2. Capability Admission Must Precede Materialization

Milestone 7 must reject unsupported source requests before any read execution,
packet materialization, or host-specific reconstruction begins.

The bridge must know, before materialization:

- whether snapshot reads are admitted
- whether historical reads are admitted
- whether branch-local reads are admitted
- whether field or facet reads are admitted
- whether replay compatibility holds for the requested truth-view mode

This is a direct application of `MENTALITY.md` and Performance Law
"rejection must precede expensive construction."

### 2.1 Source Capability Vocabulary Must Be Closed Per Milestone

Milestone 7 must not leave source capability meaning implementation-defined.

For this milestone the bridge must ship a closed capability vocabulary
including at least:

- `SnapshotRead`
- `HistoricalRead`
- `BranchRead`
- `FacetRead`
- `ReplayCompatibleRead`

If a host can do something beyond that, it remains host-local until the bridge
admits it in a later milestone.

`FacetRead` is intentionally narrow in Milestone 7.

It means:

- a bridge-declared selector admitted at registration time
- with canonical selector identity
- with canonical ordering and digest participation

It does not mean:

- arbitrary host-defined projections
- ad hoc relational query surfaces
- callback-shaped extraction logic supplied at materialization time

### 3. Source Declaration, Truth-View Selection, And Materialization Are Distinct Proof Chains

Milestone 7 must not collapse:

- `SourceDeclaration`
- `ValidatedSourceDeclaration`
- `AdmittedSourceContract`
- `PlannedSourceReadPacketSet`
- `MaterializedTruthViewPacketSet`

Rules:

- a declaration is not itself an admitted contract
- an admitted contract is not itself permission to materialize any host read
  shape
- a packet plan is not itself a materialized truth view
- replay records must carry declaration truth, capability truth, and
  materialization truth separately

The authority-basis proof is part of this chain:

- declaration chooses an authority basis
- admission proves the basis is legal for the declared capability set
- packet planning lowers exactly that basis
- materialization consumes exactly that basis without substitution

### 4. Packetized Read Planning Must Be Authoritative For Broad Source Work

Milestone 7 must not let broad truth-view work devolve into scalar read loops
at the bridge boundary.

If a request implies multiple truth surfaces, branches, versions, or facets,
the bridge must lower it into canonical packetized source work first.

Scalar convenience reads may exist only as layers on top of the packetized
surface.

### 4.1 Packet Identity Contracts Must Be Explicit

For every packetized source artifact, the spec must define:

- the exact identity-bearing fields
- the canonical ordering basis
- the deduplication rule
- the digest basis
- what explanatory fields are excluded from identity

Canonicality is mandatory for at least:

- source declaration ordering
- source registration ordering
- truth-view packet ordering
- materialized truth-view packet ordering
- source record digest computation

Host adapters must preserve these packet identities exactly.

They may:

- materialize the declared packet set
- return packet results in the canonical packet order
- annotate failures or transport-local diagnostics in derived fields

They may not:

- merge packets
- split packets into new canonical members
- reorder packet identities
- invent new packet identity-bearing fields

### 5. Builder And Configuration Surfaces Must Mirror Subsystem Boundaries

Milestone 7 must not solve host-surface cleanup by piling more methods into a
flat builder bag.

The public setup surface must remain explicit about separate responsibilities:

- source registration
- host adapter registration
- diagnostics selection
- policy wiring
- mapping registration that already exists

Adding a new source subsystem later must create obvious construction work at
compile time rather than silently inheriting ambient defaults.

### 5.1 Builder Conflict Semantics Must Be Canonical

Milestone 7 must not leave duplicate registration behavior or configuration
conflict behavior to builder call order.

The builder/configuration surface must define one canonical rule set for:

- duplicate source declarations
- overlapping source declaration identities
- duplicate adapter registration for the same source identity
- conflicting diagnostics or source-policy declarations

Rules:

- if override is not explicitly admitted, duplicates fail
- if override is explicitly admitted later, precedence must be structural and
  digest-bearing, not order-dependent folklore
- builder configuration identity must include all identity-bearing source and
  adapter declarations
- equivalent setups built in different explicit orders must canonicalize to the
  same builder configuration digest

### 6. Host Adaptation Must Be Narrow And Bridge-Owned

The bridge must depend on narrow bridge-owned source traits implemented by the
host side, not on broad relational or kernel facades.

Required consequence:

- a host adapter can change its internal storage model without changing the
  public bridge source contract
- signal-facing code cannot accidentally reach through to relational storage
  internals
- host-shape variation becomes a certification concern rather than a source of
  public API drift

### 6.1 Adapters Must Materialize, Not Re-Plan

Milestone 7 must not let host adapters become a second planner.

Given an admitted and planned source packet set, the adapter may only:

- resolve the requested authoritative truth view
- materialize the planned packet members
- return canonical packet-aligned results or typed failures

The adapter may not:

- choose a different truth-view authority basis
- widen or narrow requested capability
- regroup packet boundaries
- coalesce or split canonical packet members
- reinterpret a rejected capability as an admitted fallback

### 7. Diagnostics Are Derived From Canonical Source Truth

Operational truth for Milestone 7 is:

- source declaration identity
- capability identity
- truth-view selector identity
- packet identity
- materialization identity
- counters

Rich explanation remains derived under diagnostics policy. Changing diagnostics
richness must not change admitted source meaning or materialized read results.

### 7.1 Source Caches And Materialized Views Are Derived Only

Milestone 7 must future-proof the bridge against read-side convenience layers
becoming accidental authority.

Any cache, packet reuse layer, materialized source view, or adapter-local
projection introduced under this milestone or later milestones is derived
state.

Rules:

- derived source state must be destroyable and rebuildable from authoritative
  truth-view inputs plus canonical bridge records alone
- replay parity must not depend on preserved adapter-local caches
- cache reuse is only legal where an explicit equivalence contract exists
- no source cache may silently redefine truth-view authority

### 8. Observation Must Remain Phase-Typed

Milestone 7 must satisfy Architectural Laws 18, 30, and 41.

Rules:

- a source materialization packet is read-only observation, not mutation
  authority
- materialization may not gain branch retargeting, capability widening, or
  storage-specific powers that planning did not admit
- phase transitions must make skipped admission or skipped packet planning
  uncompilable wherever the type system can enforce it
- runtime checks for source facts already carried by proof-bearing types are a
  design failure

### 9. Cost Must Be Visible At The Source Boundary

Milestone 7 must expose source counters and decision records explaining:

- how many declarations were admitted
- how many packets were planned
- what truth-view modes were used
- what capability mismatches were rejected
- what fallback classes, if any, were exercised

The bridge must not hide broad truth-view work behind cheap-looking read APIs.

## Phases

### Phase 1: Canonical Source Declaration And Capability Vocabulary

Phase 1 exists to make truth-backed reads structurally representable before
materialization, host cleanup, or parity claims are made.

Milestone 7 must first define:

- one canonical source declaration surface
- canonical source declaration identity
- canonical source capability identity
- canonical truth-view authority-basis identity
- the exact ontology for snapshot, historical, branch, replay-compatible, and
  admitted facet reads
- the distinction between declared capability, admitted capability, and
  materialized truth-view result
- the typed failure classes for unsupported or mismatched source capability

This phase leaves the system in a coherent state where:

- hosts and consumers speak one bridge-owned source vocabulary
- truth-view authority is explicit rather than ambient
- unsupported source modes are explicit before materialization exists
- later packet planning and builder cleanup can consume explicit source truth
  rather than adapter folklore

### Phase 2: Truth-View Admission And Packetized Materialization Planning

Phase 2 exists to turn source vocabulary into deterministic admitted truth-view
plans.

Milestone 7 must then implement:

- validation of source declarations
- admission of capability sets
- admission of truth-view authority basis
- typed truth-view selector admission
- deterministic packet planning for admitted source reads
- exact canonical identity and digest bases for source packets and source
  records

This phase leaves the system in a coherent state where:

- identical source declarations and capability inputs lower to identical packet
  plans
- unsupported source modes fail before host reads occur
- broad truth-view work stays packetized and bounded
- packet truth exists before any host adapter materialization begins

### Phase 3: Clean Host Surfaces, Materialization Discipline, And Certification

Phase 3 exists to make the public source boundary library-grade instead of
host-shaped, and to certify that later milestones cannot punch through it by
convenience.

Milestone 7 must finally ship:

- explicit narrow host adapter seams for source materialization
- adapter obedience to bridge-planned packets and authority basis
- clean builder and configuration entrypoints aligned to source responsibilities
- canonical duplicate and conflict handling for builder/source setup
- typed source materialization artifacts and counters
- harness suites proving multi-host source parity
- harness suites proving capability rejection happens before materialization
- harness suites proving builder-order and adapter-swap parity
- diagnostics reconstruction over canonical source truth
- hostile coverage for adapter capability drift and derived-state rebuildability

This phase leaves the system in a coherent state where:

- source setup remains explicit and comprehensible at construction sites
- host variation does not leak into public bridge semantics
- builder order cannot redefine source meaning
- source materialization remains subordinate to bridge planning
- later remapping, policy, and certification milestones inherit one stable
  source boundary instead of reopening it

This phase has the strictest closeout bar in the milestone.

It is not complete because:

- a direct fixture passed
- a bundle was emitted
- a digest was non-empty
- one adapter happened to work
- one hostile case failed somehow

It is complete only when the Milestone 7 certification suites satisfy the
global certification rules from
[test-requirements.md](./test-requirements.md)
mechanically.

For Milestone 7 that means every named certification suite must define, unless
the suite explicitly has a narrower reason:

- `control_lane`
- `hostile_lane`
- `replay_lane`

And must include every applicable assertion class:

- equality assertions for semantically equivalent lanes
- inequality assertions for intentionally different semantic lanes
- typed-failure assertions for rejected lanes
- zero-or-absence assertions for forbidden fallback, forbidden diagnostics
  influence, and forbidden residue

And must emit stable canonical bundles sufficient for offline audit rather than
host log inspection.

At minimum, the Milestone 7 certification surface must emit milestone-appropriate
combinations of:

- `truth_view_digest`
- `source_contract_digest`
- `routing_digest`
- `diagnostics_digest`
- `failure_digest`
- `counter_snapshot`
- `adapter_parity_matrix`
- `capability_matrix`
- `setup_parity_matrix`

And must prove all of the following, not merely imply them:

- semantically equivalent control, hostile, and replay lanes compare equal on
  canonical truth where the test declares they should
- intentionally different semantic lanes compare unequal on at least one
  declared digest or typed report
- rejected capability or configuration lanes fail at the declared admission or
  planning boundary rather than later during materialization drift
- diagnostics-richness perturbations change retained detail only and do not
  change canonical source truth
- forbidden adapter fallback leaves zero false-success residue
- forbidden builder-order influence leaves zero semantic drift
- replay from canonical source artifacts reproduces the same source truth
  without ambient host state
- exact counters match the claimed packet breadth, rejection count, fallback
  count, and zero-forbidden counters for the representative scenarios

If any certification lane depends on:

- ambient host state
- runtime log reading
- debugger-only context
- self-comparison within one run
- success-only assertions with no hostile or replay basis

then the certification is insufficient and Phase 3 is not complete.

## Must Ship

- canonical source declaration, capability, truth-view selector, and
  materialization artifacts
- typed source capability vocabulary for snapshot, historical, branch, replay,
  and admitted facet reads
- validation and admission of source declarations before materialization
- packetized source planning and typed materialization records
- clean builder/configuration entrypoints for source registration, host adapter
  wiring, diagnostics wiring, and policy wiring
- typed failures for unsupported source capability, source contract mismatch,
  truth-view mismatch, builder configuration conflict, and adapter capability
  drift
- counters and decision-log records for source declaration count, packet count,
  capability rejection count, materialization breadth, and fallback classes
- harness certification lanes for multi-host parity, capability rejection, and
  builder-swap parity

## Must Preserve

- truth runtime remains the authority for truth-view semantics and storage
- signal runtime remains the authority for downstream execution and scheduling
- no weakening of Milestone 1 through Milestone 6 canonical routing, snapshot,
  historical, bulk-planning, or stream-protocol truth
- no relational storage internals leaking into signal-facing code
- no host-specific source glue becoming the public bridge contract
- no ambient source capability or builder-order semantics
- canonical ordering and replay-safe source identities
- diagnostics richness changes explanation only, not source truth

## Acceptance Evidence

Milestone 7 is complete only when the bridge harness can prove:

- identical source declarations and admitted capability inputs lower to
  identical packetized truth-view plans
- multiple host-shaped source implementations satisfy the same canonical bridge
  contract
- source-backed evaluation remains parity-safe across snapshot, historical,
  branch, and admitted facet reads
- unsupported source modes fail during admission rather than during
  materialization
- builder setup order and adapter swapping do not change canonical source
  meaning
- diagnostics richness changes explanation only, not source truth or routing
  meaning
- source mismatch and capability mismatch failures are explicit, typed, and
  diagnosable
- replay from canonical source artifacts reproduces the same source truth
  without ambient host interpretation
- forbidden adapter fallback leaves zero false-success residue
- forbidden builder-order influence leaves zero semantic drift
- counter assertions prove the claimed packet breadth, rejection boundaries, and
  zero-forbidden counters exactly for representative certification scenarios

## Architectural Notes

### Expected Internal Subdomains

Milestone 7 should extend the bridge crate with subdomains such as:

- `source/declaration/`
- `source/contracts/`
- `source/capabilities/`
- `source/planning/`
- `source/materialization/`
- `source/adapters/`
- `builder/source/`
- `diagnostics/source/`
- `harness/fixtures/source_parity.rs`
- `harness/fixtures/source_capability_rejection.rs`
- `harness/fixtures/source_builder_parity.rs`

This follows workspace domain standards:

- declaration validation is not the same responsibility as capability
  admission
- truth-view planning is not the same responsibility as packet materialization
- adapter capability reporting is not the same responsibility as builder
  configuration
- delivery/routing behavior remains separate from source materialization

### Minimum Counter Floor

Milestone 7 must add counters such as:

- `source_declaration_count`
- `source_contract_count`
- `source_packet_count`
- `source_packet_member_count`
- `source_materialization_count`
- `source_snapshot_read_count`
- `source_historical_read_count`
- `source_branch_read_count`
- `source_facet_read_count`
- `source_capability_rejection_count`
- `source_contract_mismatch_count`
- `source_adapter_fallback_count`
- `source_builder_configuration_conflict_count`
- `source_replay_request_count`

Exact names may refine during implementation, but the structural floor is not
optional.

### Explicit Source Failure Policy

Milestone 7 must carry source failures structurally rather than narratively.

Required failure classes:

- `UnsupportedSourceCapability`
- `SourceContractMismatch`
- `SourceContractVersionMismatch`
- `TruthViewSelectionMismatch`
- `HistoricalReadUnavailable`
- `BranchReadUnavailable`
- `FacetReadUnavailable`
- `ReplayIncompatibleSourceRequest`
- `SourceMaterializationRejected`
- `AdapterCapabilityDrift`
- `BuilderConfigurationConflict`

Rules:

- every admitted source request receives exactly one materialization outcome
- failure remains visible in canonical source truth
- failure must identify the source boundary that failed
- capability or contract rejection must not degrade into silent fallback

## Test And Harness Model

Milestone 7 must follow the same structural testing discipline as earlier
bridge milestones and must satisfy the Milestone 7 certification suites in
[test-requirements.md](./test-requirements.md).

Expected first-class test surfaces:

- multi-host source parity scenarios
- source capability compatibility and incompatibility scenarios
- source packet planning determinism scenarios
- builder-order and adapter-swap parity scenarios
- diagnostics-tier invariance scenarios
- counter certification scenarios

Milestone 7 is not complete with only direct fixture tests. It must establish a
real source-contract certification surface on top of `worth-harness`.

Expected harness surfaces:

- `ScenarioPlan` and `ScenarioFixture` for truth-view matrices
- `ExecutionRequest` for declaration validation, capability admission, packet
  planning, source materialization, and diagnostics capture
- `ExecutionProfile` for deterministic, replay, and adapter-variation sweeps
- `ParitySuite` for adapter-to-adapter and setup-to-setup parity
- `CertificationMatrix` for hostile capability rejection and builder-order
  coverage

Every named Milestone 7 certification suite must follow the certification
discipline from
[test-requirements.md](./test-requirements.md)
explicitly.

Required lane structure unless the suite explicitly documents a narrower reason:

- `control_lane`
- `hostile_lane`
- `replay_lane`

Required assertion classes wherever applicable:

- equality assertions for semantically equivalent lanes
- inequality assertions for intentionally divergent semantic lanes
- typed-failure assertions for rejected lanes
- zero-or-absence assertions for forbidden fallback, forbidden diagnostics
  influence, and forbidden residue

Required Milestone 7 canonical bundle fields across the certification surface:

- `truth_view_digest`
- `source_contract_digest`
- `routing_digest`
- `diagnostics_digest`
- `failure_digest`
- `counter_snapshot`

Required suite-specific bundle fields:

- `adapter_parity_matrix` for multi-host parity
- `capability_matrix` for capability rejection
- `setup_parity_matrix` for builder/config swap parity

The following do not count as certification for Milestone 7:

- asserting that one run completed
- asserting that a digest is present or non-empty
- comparing a result only to itself from the same run
- checking only a happy path
- checking only a rejection path without a control or replay basis
- requiring host logs to understand why the suite passed

Phase 3 is not complete until the certification results are auditable from the
canonical bundle alone.

Minimum representative test names:

- `tests::source::identical_source_inputs_lower_to_identical_packet_plans`
- `tests::source::multi_host_adapters_preserve_canonical_truth_view_results`
- `tests::source::unsupported_source_capability_fails_before_materialization`
- `tests::builder::source_builder_order_does_not_change_contract_meaning`
- `tests::diagnostics::source_diagnostics_richness_preserves_source_truth`

## Target API And Module Plan

### Public Surface Growth

Milestone 7 should extend the facade with bridge-owned source types such as:

```rust
pub struct SourceDeclaration { ... }
pub struct ValidatedSourceDeclaration { ... }
pub struct AdmittedSourceContract { ... }
pub struct SourceCapabilitySet { ... }
pub struct PlannedSourceReadPacketSet { ... }
pub struct MaterializedTruthViewPacketSet { ... }
pub struct SourceMaterializationRecord { ... }

impl RuntimeBridge {
    pub fn admit_source(
        &self,
        declaration: SourceDeclaration,
    ) -> Result<AdmittedSourceContract, BridgeSourceError>;

    pub fn materialize_source(
        &self,
        contract: AdmittedSourceContract,
    ) -> Result<MaterializedTruthViewPacketSet, BridgeSourceError>;
}
```

Design rules:

- the facade exposes bridge source concepts only
- it does not re-export relational storage handles as the main contract
- admission and materialization are separate public boundary crossings
- callers must not be able to trigger hidden broad truth-view work through a
  getter-shaped API

### Host Adapter Growth

Milestone 7 should introduce or refine bridge-owned adapter seams such as:

```rust
pub trait BridgeSourceAdapter {
    type Error;

    fn declared_capabilities(&self) -> SourceCapabilitySet;

    fn materialize_packets(
        &self,
        packets: &PlannedSourceReadPacketSet,
    ) -> Result<MaterializedTruthViewPacketSet, Self::Error>;
}
```

Rules:

- the trait is owned by `worth-runtime-bridge`
- host implementations may internally use relational storage, database APIs, or
  kernel-specific views, but that detail must stay outside the public bridge
  contract
- capability reporting must be canonical and replay-safe

### Builder Growth

Milestone 7 should refine `RuntimeBridgeBuilder` so source setup is explicit
and subsystem-shaped.

Representative direction:

```rust
impl RuntimeBridgeBuilder {
    pub fn register_source(self, source: SourceDeclaration) -> Self;
    pub fn with_source_adapter<A>(self, adapter: A) -> Self
    where
        A: BridgeSourceAdapter;

    pub fn with_diagnostics_policy(self, policy: BridgeDiagnosticsPolicy) -> Self;
}
```

Rules:

- source registration and source adapter wiring are distinct builder concerns
- diagnostics and policy wiring remain explicit rather than ambient
- equivalent source setups must remain parity-safe under different explicit
  builder call orders

### New Files Expected

- `crates/worth-runtime-bridge/src/source/mod.rs`
- `crates/worth-runtime-bridge/src/source/declaration.rs`
- `crates/worth-runtime-bridge/src/source/contracts.rs`
- `crates/worth-runtime-bridge/src/source/capabilities.rs`
- `crates/worth-runtime-bridge/src/source/planning.rs`
- `crates/worth-runtime-bridge/src/source/materialization.rs`
- `crates/worth-runtime-bridge/src/source/adapters.rs`
- `crates/worth-runtime-bridge/src/diagnostics/source.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/source_parity.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/source_capability_rejection.rs`
- `crates/worth-runtime-bridge/src/harness/fixtures/source_builder_parity.rs`
- `crates/worth-runtime-bridge/src/tests/source/contracts.rs`
- `crates/worth-runtime-bridge/src/tests/source/materialization.rs`
- `crates/worth-runtime-bridge/src/tests/source/capabilities.rs`
- `crates/worth-runtime-bridge/src/tests/source/builder.rs`

### Existing Files Expected To Change

- [adapter.rs](../../crates/worth-runtime-bridge/src/adapter/mod.rs)
- [builder.rs](../../crates/worth-runtime-bridge/src/builder/mod.rs)
- [facade.rs](../../crates/worth-runtime-bridge/src/facade.rs)
- [lib.rs](../../crates/worth-runtime-bridge/src/lib.rs)
- [mod.rs](../../crates/worth-runtime-bridge/src/diagnostics/mod.rs)
- [mod.rs](../../crates/worth-runtime-bridge/src/harness/mod.rs)
- [mod.rs](../../crates/worth-runtime-bridge/src/snapshot/mod.rs)
- [materialization.rs](../../crates/worth-runtime-bridge/src/snapshot/materialization.rs)
- [declaration.rs](../../crates/worth-runtime-bridge/src/snapshot/declaration.rs)

## Implementation Phases

Milestone 7 must execute in strict order. Later phases may reopen earlier ones,
but no phase may bypass unfinished source foundations with host-local adapter
glue or flat builder flags.

### Phase M7.0 - Source Taxonomy And Capability Lock

Purpose:

- define the one source declaration surface
- define canonical source and capability vocabulary
- lock what truth exposes versus what the bridge admits

Required work:

- define `SourceDeclaration`
- define canonical source declaration identity and capability identity
- define the Milestone 7 closed capability vocabulary
- define explicit unsupported source classes
- define the distinction between declaration, admission, and materialization

Exit criteria:

- the source declaration surface is singular and explicit
- unsupported source modes are named rather than deferred
- host adapters are no longer the public vocabulary for read capability

### Phase M7.1 - Truth-View Admission And Packet Planning

Purpose:

- resolve source applicability before materialization begins

Required work:

- define `ValidatedSourceDeclaration`
- define `AdmittedSourceContract`
- define `PlannedSourceReadPacketSet`
- define exact canonical ordering and digest bases for source packet planning
- define typed capability rejection outcomes
- add exact counters and decision-log records

Exit criteria:

- source applicability is fully resolved before host reads occur
- identical source declarations and capability inputs lower to identical packet
  plans
- unsupported combinations become typed outcomes rather than materialization-time drift

### Phase M7.2 - Source Materialization And Builder Surface Cleanup

Purpose:

- make public setup and host seams library-grade

Required work:

- add `BridgeSourceAdapter`
- add `MaterializedTruthViewPacketSet`
- refine builder surfaces so source registration, adapter wiring, diagnostics
  wiring, and policy wiring remain explicit
- define identity relationships among declaration, contract, packet, and
  materialization records

Exit criteria:

- host materialization uses bridge-owned narrow seams
- builder setup is subsystem-shaped rather than flat
- source materialization remains separate from routing, stream, and diagnostics
  ownership

### Phase M7.3 - Multi-Host Certification And Hostile Coverage

Purpose:

- make source-contract claims certifiable rather than plausible

Required work:

- add `worth-harness` fixtures, parity suites, and certification matrices for
  multi-host parity, source capability rejection, and builder-order parity
- add hostile adapter capability-drift lanes
- add exact counter assertions for source packet breadth and rejection classes
- add replay lanes that reproduce source truth from canonical source artifacts
  alone
- add zero-or-absence assertions for forbidden fallback, forbidden diagnostics
  influence, and forbidden residue

Exit criteria:

- all roadmap acceptance evidence is covered by bridge-native harness scenarios
- host variation is mechanically certified not to alter canonical source truth
- builder richness changes observability and ergonomics only, not source meaning
- every named Milestone 7 certification suite has explicit control, hostile,
  and replay bases unless a narrower shape is explicitly justified
- pass/fail analysis is possible from canonical bundles alone without host logs

## Explicit Failure Taxonomy For Milestone 7

Milestone 7 must ship typed bridge failures for at least:

- unsupported source capability
- source contract mismatch
- source contract version mismatch
- truth-view selection mismatch
- historical read unavailable
- branch read unavailable
- facet read unavailable
- replay-incompatible source request
- source materialization rejection
- adapter capability drift
- builder configuration conflict

These are bridge failures, not raw parent-runtime strings.

## Anti-Patterns Explicitly Rejected

- exposing relational storage handles as the public bridge read contract
- discovering unsupported source modes only after host materialization begins
- treating scalar reads as the primary bridge truth for broad truth-view work
- making builder call order semantically meaningful
- letting host adapters silently widen, narrow, or reinterpret declared source
  capability
- mixing source materialization responsibilities into stream, routing, or
  diagnostics catch-all files
- hiding source cost behind elapsed-time metrics with no explanatory counters

## Sequencing Notes

Milestone 7 must land before:

- Milestone 8 structural-identity-aware remapping, because remapping needs one
  stable host-agnostic source contract for branch and historical comparison
- Milestone 11 policy propagation, because builder and configuration surfaces
  must already be structurally clean before policy provenance is layered onto
  them
- Milestone 13 bridge certification, because the bridge is not certifiable
  while its read boundary still depends on host-shaped adapter glue

Milestone 7 must not attempt to pre-solve:

- merge-aware source semantics
- structural-identity remapping policy
- speculative preview lifecycle
- cross-runtime policy provenance
- writeback or effect production

Those become stronger because Milestone 7 exists; they do not need to be
smuggled into it.

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because the bridge cannot honestly be a library-grade dual-runtime
boundary while reads still depend on relational internals, ambient capability
knowledge, and builder-order folklore.

The adversarial constraint is load-bearing because it forbids the easy failure
mode of treating read capability as whatever a host adapter happens to support,
hiding unsupported source modes until late materialization, and collapsing
clean setup into a flat bag of flags.

The milestone preserves authority boundaries because truth still owns truth-view
semantics and storage, signal still owns downstream computation, and the bridge
owns only the source protocol, admission, packet planning, and public setup
surface between them.

The milestone defines proof obligations rather than implementation chores
because canonical source identity, typed capability admission, packetized read
planning, clean builder surfaces, and multi-host parity certification are all
required for closeout.

A competent engineer should be able to map this spec into honest source
artifacts, adapter traits, builder surfaces, counters, and harness suites
without inventing the architecture during implementation.

## Closeout Standard

Milestone 7 is complete only when all of the following are true:

- truth-backed reads lower through one canonical bridge source contract surface
- source declaration, capability, truth-view admission, and materialization
  truth remain structurally distinct
- unsupported source modes fail before materialization begins
- broad truth-view work lowers through canonical packetized source planning
- builder and configuration surfaces remain explicit, subsystem-shaped, and
  parity-safe under equivalent construction orders
- multiple host adapters preserve identical canonical bridge-visible source
  meaning
- source truth is replay-safe and diagnostics-tier-invariant
- harness certification proves adapter parity, capability rejection, and
  explicit failure behavior under hostile host variation

If code lands but signal-facing code still depends on relational internals,
source capability remains ambient, builder order changes meaning, unsupported
reads fail late inside host materialization, or source diagnostics are the only
place canonical read truth can be understood, Milestone 7 is not complete.
