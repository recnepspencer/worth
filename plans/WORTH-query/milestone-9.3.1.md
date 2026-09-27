# Milestone 9.3.1 Engineering Spec: Cross-Runtime Causal Diagnostics And Query Inspection

> **Status:** Closed on 2026-05-12 for runtime-backed cross-runtime causal
> diagnostics and Query inspection; durable causal archives, store-backed replay
> reconstruction, and restart-stable expanded explanation reload remain later
> milestone debt.
>
> **Closeout:** [milestone-9.3.1-closeout.md](./milestone-9.3.1-closeout.md)
>
> **Roadmap parent:** [worth_query_roadmap.md](./worth_query_roadmap.md)
>
> **Vision parent:** [worth_query_vision.md](./worth_query_vision.md)
>
> **Prior milestone:** [milestone-9.3.md](./milestone-9.3.md)
>
> **Next milestone:** [milestone-9.3.2](./milestone-9.3.2.md)
> continues the runtime API stabilization path by making Query basis a
> capability lifecycle. The Runtime API Public Stabilization Gate follows
> Milestones 9.3.2 through 9.3.6.
>
> **Test requirements:** [test-requirements.md](./test-requirements.md)
>
> **Primary architectural driver:** make Query inspection the ordinary public
> door for cross-runtime causal explanations while preserving runtime bridge,
> relational, and signal authority boundaries.
>
> **Companion docs:**
> - [MENTALITY.md](../../docs/coding-guidelines/MENTALITY.md)
> - [arch_laws.md](../../docs/coding-guidelines/arch_laws.md)
> - [composition_laws.md](../../docs/coding-guidelines/composition_laws.md)
> - [domain_structure_laws.md](../../docs/coding-guidelines/domain_structure_laws.md)
> - [perf_laws.md](../../docs/coding-guidelines/perf_laws.md)
> - [worth_query_vision.md](./worth_query_vision.md)
> - [worth_query_roadmap.md](./worth_query_roadmap.md)
> - [test-requirements.md](./test-requirements.md)
> - [milestone-9.3.md](./milestone-9.3.md)

## Goal

Make cross-runtime causal diagnostics a first-class Query inspection capability.
When a query-observed result changes, is suppressed, is denied, or is replayed,
Query must expose one typed inspection artifact that joins relational authority,
runtime-bridge routing/evaluation/source/structural/stream/preview/writeback
evidence, signal invalidation/evaluation/forensic availability, lineage,
provenance, and replay posture without requiring consumers to reach directly
into lower runtimes.

## Why This Milestone Exists

Milestone 9.3 makes subscription family selection bridge-honest. It does not
yet close the larger explanation boundary that downstream domains need: "why
did this query-observed thing happen?"

Without Milestone 9.3.1, a domain such as Worth can be tempted to assemble
explanations by importing runtime-bridge diagnostics, relational evidence, and
signal graphs directly. That produces powerful local narratives, but it also
means the public Query surface missed the boundary where those lower-runtime
facts should be anchored, admitted, typed, referenced, redacted, and replayable.

This milestone exists to make that boundary explicit. The runtime bridge may
need new implementation work, but the public contract belongs in Query because
Query is the ordinary read, live, and inspection surface for domain runtimes.

## Governing Summaries

- `MENTALITY.md`: solve the adversarial version first. The hard case is not
  "show a diagnostic string"; it is proving one canonical causal explanation
  across multiple lower-runtime authorities without moving their authority into
  Query.
- `arch_laws.md`: boundary envelopes must be self-describing, phase-typed, and
  proof-bearing. Cross-runtime diagnostics must therefore be an admitted
  artifact with decision logs and bounded evidence references, not ambient
  debug access or a second diagnostics store.
- `composition_laws.md`: responsibility names must stay honest. Query owns the
  public inspection request and materialized inspection artifact; the bridge
  owns bridge causal envelopes; signal owns signal evidence; relational owns
  truth authority.
- `domain_structure_laws.md`: source truth, projections, diagnostics, and
  boundary crossings must remain visible in the tree. This milestone may not
  bury bridge/signal/relational stitching inside a Worth helper or a generic
  `diagnostics.rs` bag.
- `perf_laws.md`: diagnostic richness belongs on the cold path. The causal
  envelope must carry retained references, policy, counters, and materialization
  posture without widening hot-path invalidation or query execution.
- lower-runtime awareness: Query already has inspection variants and mutation
  causality/provenance evidence; runtime bridge already retains route,
  historical, source, structural, stream, preview, writeback, replay, and
  diagnostics records; signal already exposes forensic availability, replay
  cursors, and lineage artifact identities. Milestone 9.3.1 must bind and
  expose those artifacts, not rediscover or duplicate their authority.

## Adversarial Constraint

A downstream domain must be able to ask Query inspection why a specific
query-observed artifact changed, did not change, failed, was denied, or replayed
differently, and receive one machine-checkable causal explanation anchored to
the same canonical observation receipt that produced the public Query result.
The explanation must prove the same relational authority, bridge route, bridge
evaluation, source materialization, structural transition, writeback/preview
posture, signal invalidation, signal evaluation, lineage, provenance, forensic
availability, and replay posture that lower-runtime diagnostics would expose
independently. The domain must not need direct access to runtime-bridge
diagnostics, relational internals, or signal graph internals to construct that
explanation.

## Product Decision Lock

The product surface is Query inspection. The implementation may extend the
runtime bridge, signal diagnostic retention, or relational evidence export, but
the user-facing capability is not "call the bridge diagnostics facade from a
domain." It is "inspect this query observation and receive the causal envelope
that explains it."

Crate ownership is load-bearing for this milestone:

- `worth-query` owns the public inspection request, observation receipt/anchor,
  evidence-reference request and resolution posture, admission/advisory/denial
  lattice, redaction/materialization policy, and final Query-facing inspection
  artifact.
- `worth-runtime-bridge` owns bridge causal envelope authority: bridge causal
  envelope identity, evidence binding, envelope receipt, envelope denial,
  bridge-side retained indexes, bridge-side causal mapping from existing route,
  evaluation, source, structural, stream, preview, writeback, continuity,
  merge, mapper, replay, historical, and diagnostics records, and the
  envelope assembly operation itself.
- `worth-relational` owns truth authority, commit/snapshot authority,
  relational decision evidence, relational causality/provenance, and any
  relational digest that appears in the causal story.
- `worth-signal` owns invalidation, evaluation, scheduling, signal lineage,
  signal provenance, forensic availability, replay cursor, and signal-side
  observation evidence.

Forbidden in `worth-query`:

- defining production `BridgeCausal*` envelope, binding, index, receipt,
  counter, or denial types
- assembling a bridge causal envelope from retained bridge diagnostics
- indexing retained runtime-bridge records as though Query owned the bridge
  diagnostics store
- minting synthetic bridge route/evaluation/source/structural/stream/preview/
  writeback/replay facts from Query observation receipts
- exporting bridge-owned constructors or bridge-owned proof artifacts through
  the Query facade

Query may be bridge-aware, but it must not become bridge-authoritative. Its
bridge-facing production code is limited to request/response adapters around
the runtime bridge facade and materialization from a sealed bridge-owned
envelope.

## Typed Phase Progression

Milestone 9.3.1 must introduce or certify the following phase progression:

- `QueryObservationReceipt` or existing operational receipt: the canonical
  Query artifact that originally observed, delivered, denied, replayed, or
  suppressed the result being inspected.
- `CausalObservationAnchor`: Query-owned proof that names exactly one
  observation receipt, basis posture, result-shape context, observation target,
  and lower-runtime evidence identities emitted by the operation that produced
  the observation.
- `CausalEvidenceReferenceSet`: bounded digest/reference set for relational,
  bridge, signal, lineage, provenance, replay, policy, forensic availability,
  and materialization evidence retained by lower-runtime authorities or Query
  operational receipts.
- `CausalInspectionRequest`: Query-owned request naming the causal observation
  anchor, inspection reason, requested evidence families, and permitted
  diagnostic richness.
- `CausalInspectionAdmission`: policy-checked success/advisory/violation record
  proving whether the request can ask for the selected explanation family,
  whether detail is narrowed, and what evidence families may be materialized.
- `BridgeCausalExplanationEnvelope`: bridge-owned envelope joining bridge
  route/evaluation/source/structural/stream/preview/writeback/replay facts with
  relational authority and signal evidence references.
- `QueryCausalInspectionArtifact`: Query-owned public artifact that presents the
  admitted explanation with redaction, capability posture, counters, and
  replay/materialization metadata.
- `CausalInspectionCertificationBundle`: hostile proof package covering
  changed, suppressed, denied, missing-evidence, branch/preview, and replay
  cases.

Rules:

- each phase consumes only the immediately prior proof type or an explicitly
  named lower-runtime evidence authority already represented in the causal
  observation anchor
- no public API may construct a later-phase artifact from raw strings,
  lower-runtime handles, ad hoc JSON, or optional-hole structs
- Query may not mint bridge-owned causal envelope facts; it may only request,
  admit, redact, materialize, and certify them
- production `BridgeCausalExplanationEnvelope` and sibling `BridgeCausal*`
  proof types must live in `worth-runtime-bridge`, not in `worth-query`
- Query may carry opaque bridge envelope identity, digest, denial, and receipt
  references returned by the bridge facade, but those references are not bridge
  authority and cannot be expanded without the bridge-owned API
- bridge-owned envelopes must name which lower runtime supplied each evidence
  family
- Query may retain observation anchors and evidence references, but it may not
  become the primary store for bridge diagnostics, signal forensic artifacts, or
  relational truth evidence
- diagnostic richness may widen materialized explanation detail, but it may not
  widen retained hot-path evidence, route selection, query meaning, or signal
  invalidation meaning
- denied, advisory, and admitted inspection artifacts are different proof
  families rather than the same structure with missing fields

## Phases

Phases are ordered gates. A later phase may be sketched with fixture types, but
it may not be implemented against production surfaces until every prior phase
has passed its exit criteria, performance counters, and compile-fail/proof
boundary checks. The point is to keep the proof chain honest: each phase must
consume the proof-bearing artifact emitted by the phase immediately before it.

### Phase 1: Existing Evidence Inventory And Observation Anchor Adapters

Inventory the existing Query, bridge, signal, and relational evidence surfaces
and add the smallest adapter layer needed to derive one causal observation
anchor from existing Query operational artifacts. This phase is deliberately
practical: no bridge envelope, no public materialized artifact, and no new
diagnostics store yet.

Must ship:

- `QueryObservationReceipt` coverage for existing Query observation families
  where a narrower receipt type already exists, such as write receipts, intent
  denials, live delivery/inspection receipts, preview receipts, branch intent
  receipts, replay receipts, and suppression/no-op receipts
- `CausalObservationAnchor`
- `CausalObservationAnchorDigest`
- `CausalEvidenceFamily`
- `CausalObservationOutcome`
- `CausalInspectionReason`
- inventory table mapping each evidence family to its authority surface
  (`worth-query`, `worth-runtime-bridge`, `worth-relational`, or
  `worth-signal`) and to the reference identity Query can carry

Must preserve:

- every causal explanation begins from one canonical observation anchor, not a
  post-hoc search across matching digests
- lower runtimes remain authorities for their evidence records
- inventory names authority surfaces and reference identities only; it must not
  introduce Query-owned bridge record classes or retained bridge indexes
- this phase adds anchor derivation only; it does not expose public causal
  explanation APIs

Exit criteria:

- tests prove a causal inspection cannot be requested without an observation
  anchor derived from a Query operational artifact
- changed, suppressed, denied, branch/preview, and replay observation anchors
  can be constructed from fixture receipts
- anchor construction emits exact counters for source receipt family,
  reference-family count, missing-reference posture, and anchor digest width
- anchor construction is O(reference families carried by the receipt), not
  O(runtime graph size) or O(total diagnostics retention width)

### Phase 2: Evidence Reference Resolution And Performance Indexes

Build the typed evidence-reference set and the indexed lower-runtime lookup
contracts that later phases consume. This phase proves that evidence references
are cheap, bounded, and authority-preserving before any admission or envelope
logic depends on them.

Must ship:

- `CausalEvidenceReferenceSet`
- `CausalEvidenceReferenceDigest`
- `CausalEvidenceReferenceReceipt`
- `CausalEvidenceReferenceResolution`
- `CausalEvidenceReferenceResolutionCounters`
- indexed reference adapters for:
  - relational authority and decision evidence
  - bridge route, evaluation, source materialization, source failure,
    continuity, merge, structural remap, structural branch comparison, stream,
    preview, writeback, mapper, and replay evidence
  - signal invalidation, evaluation, forensic availability, replay cursor, and
    lineage artifact evidence
  - bridge/query mutation causality and provenance evidence
  - replay and materialization posture
  - policy and redaction posture

Must preserve:

- evidence references are sufficient for cold-path materialization, but are not
  themselves an expanded narrative or a second authority store
- reference resolution may read lower-runtime diagnostic indexes, but it may not
  scan unrelated retained records looking for plausible matches
- bridge record lookup must happen through bridge-owned facade/index surfaces;
  Query may not mirror, rebuild, or retain a Query-local bridge diagnostics
  index
- missing evidence has typed posture before any public artifact is materialized

Exit criteria:

- evidence references can represent changed, suppressed, denied,
  branch/preview, and replayed observations
- tests prove evidence-reference width is independent of unrelated runtime graph
  size and lower-runtime retained record count
- certified ordinary lanes have zero bridge record scan fallbacks
- small/medium/large fixtures prove lookup cost follows reference width and
  requested evidence-family count

### Phase 3: Query Inspection Admission And Proof Progression

Define the Query-owned request and admission boundary for cross-runtime causal
inspection. This phase consumes a `CausalObservationAnchor` and
`CausalEvidenceReferenceSet`; it may not accept raw lower-runtime handles or raw
diagnostics records.

Must ship:

- `CausalInspectionRequest`
- `CausalInspectionTarget`
- `CausalInspectionAdmissionSubject`
- `CausalInspectionAdmissionDecision` with success, advisory, and violation
  families
- `AdvisoryCausalInspection`
- `AdmittedCausalInspection`
- `DeniedCausalInspection`
- `CausalInspectionAdmissionReceipt`
- `CausalDecisionTraceIndex`
- `CausalInspectionProofFlow` or equivalent `worth-proof`-backed progression
  wrapper for request -> admitted/advisory/denied inspection

Must preserve:

- Query owns public inspection admission and redaction posture
- policy, tenant, branch, preview, and result-shape context are resolved before
  bridge envelope materialization
- unsupported explanation families fail before bridge or signal internals are
  exposed
- advisory outcomes proceed only with structured narrowing, redaction, or
  missing-evidence posture; they are not silently promoted to full success
- the admission decision trace is span-aware, queryable, and has O(1) lookup for
  authority-path decisions used by the causal artifact
- admission does not resolve bridge envelopes or materialize narratives

Exit criteria:

- public callers cannot request cross-runtime explanations with raw bridge,
  relational, or signal handles
- success, advisory, and violation admission cases each produce typed artifacts
  with decision trace and counters
- admitted inspection carries exactly the permissions and evidence families the
  next phase may request
- advisory inspection proves which requested detail was redacted, narrowed,
  deferred, or unavailable before any bridge envelope is requested
- compile-fail tests prove a request cannot skip anchor/reference phases and
  cannot be treated as admitted/advisory/denied without the admission boundary

### Phase 4: Runtime-Bridge Causal Envelope Authority

Implement the bridge-owned envelope in `crates/worth-runtime-bridge` and expose
it only through a runtime-bridge facade surface that Query can call after Query
admission succeeds. This phase joins lower-runtime evidence without moving
bridge, relational, or signal authority into Query. It consumes only an
`AdmittedCausalInspection` or `AdvisoryCausalInspection` summary from Query plus
resolved evidence references; it returns either a sealed bridge-owned envelope
or a bridge-owned typed denial.

Must ship:

- `BridgeCausalExplanationEnvelope` in `worth-runtime-bridge`
- `BridgeCausalEnvelopeIdentity` in `worth-runtime-bridge`
- `BridgeCausalEvidenceBinding` in `worth-runtime-bridge`
- `BridgeCausalEnvelopeReceipt` in `worth-runtime-bridge`
- `BridgeCausalEnvelopeDenial` in `worth-runtime-bridge`
- mapping from existing bridge diagnostics, historical evaluation, writeback,
  preview, route, evaluation, source materialization, source failure,
  structural, stream, continuity, merge, mapper, and replay records into causal
  evidence bindings
- one bridge facade operation that accepts a Query-admitted causal inspection
  summary plus evidence references and returns the sealed bridge-owned causal
  envelope or denial
- bridge envelope performance counters for binding count, lower-runtime family
  count, index lookup count, materialized-detail count, and scan fallback count

Must preserve:

- this phase is production bridge work, not Query work; Query may add adapter
  types only after the bridge facade shape exists
- bridge route/evaluation semantics remain bridge-owned
- relational evidence remains relational-owned and referenced through digests or
  authority records
- signal evidence remains signal-owned and referenced through retained
  invalidation/evaluation, lineage, and provenance records
- the bridge envelope names evidence ownership explicitly rather than flattening
  all facts into one Query story
- bridge envelope assembly consumes the causal observation anchor and evidence
  references; it may not scan unrelated diagnostics state looking for plausible
  matches
- Query may not assemble a bridge envelope, fabricate bridge evidence bindings,
  or translate bridge retained records into causal facts outside the
  bridge-owned assembly path

Exit criteria:

- changed, suppressed, denied, branch/preview, and replay scenarios can all
  produce either an admitted bridge envelope or a typed bridge-envelope denial
- bridge envelope digests agree with the lower-runtime records they summarize
- no Query type defines, constructs, or exports production bridge causal
  envelope types or bridge causal evidence bindings
- compile-fail tests prove external callers cannot construct bridge-owned
  causal envelopes directly and cannot obtain them through Query-local
  constructors
- scale rows prove envelope assembly follows anchor/reference width rather than
  total bridge diagnostics retention width

### Phase 5: Query Materialization, Redaction, And Public Artifact

Materialize the sealed runtime-bridge-owned causal envelope into a Query-owned
public inspection artifact with policy-safe detail and stable consumer
vocabulary. Phase 5 must consume the Phase 4 bridge facade result; it must not
construct bridge facts or operate on bridge internals directly.

Must ship:

- `QueryCausalInspectionArtifact`
- `AdmittedQueryCausalInspectionArtifact`
- `AdvisoryQueryCausalInspectionArtifact`
- `DeniedQueryCausalInspectionArtifact`
- `CausalInspectionRedactionPolicy`
- `CausalInspectionMaterializationPolicy`
- `CausalMaterializationReceipt`
- `CausalInspectionPerformanceEnvelope`
- Query-side bridge envelope adapter/reference types only where needed to bind
  the sealed bridge facade result to Query materialization
- result-shape context and query observation digest binding
- boundary-envelope categories required by `arch_laws.md`: primary result,
  structured warnings, decision trace, structural deltas, integrity markers,
  and performance accounting

Must preserve:

- public artifacts preserve lower-runtime authority names and evidence digests
- public artifacts preserve the bridge envelope digest/receipt returned by
  `worth-runtime-bridge`; they do not reassemble or reinterpret bridge bindings
- redaction can hide detail without changing causal digests or outcome meaning
- expanded narrative/prose renderers are derived from machine-checkable
  artifacts
- domains consume Query inspection artifacts rather than bridge, relational, or
  signal internals
- materialized artifacts expose lower-runtime evidence references and
  availability posture without requiring consumers to query lower runtimes to
  interpret the artifact
- every public artifact includes performance accounting for anchor derivation,
  evidence-reference resolution, admission, bridge envelope assembly,
  redaction, and materialization

Exit criteria:

- policy-redacted and full-richness artifacts share the same causal identity
  where semantics are unchanged
- missing or redacted evidence produces typed denial or narrowed detail rather
  than best-effort prose
- Worth-style explanation consumers can use only Query inspection artifacts for
  the ordinary explanation path
- public artifacts remain interpretable offline from their self-describing
  envelope content and referenced evidence identities
- redaction/materialization tests prove richer detail changes only detail
  digests and materialization counters, not causal identity or query meaning

### Phase 6: Certification, Performance Closure, And Escape-Hatch Audit

Close the milestone with hostile certification, compile-fail boundaries, and a
named audit of remaining escape hatches.

Must ship:

- `CausalInspectionCertificationScope`
- `CausalInspectionCertificationBundle`
- `CausalInspectionScaleCounterSnapshot`
- `CausalInspectionPerformanceCertificationBundle`
- `CausalInspectionBoundaryAudit`
- compile-fail tests rejecting public construction of causal proof artifacts
- public-surface, dependency, or lint-backed tests rejecting ordinary domain
  causal explanation paths that bypass Query inspection when the crate boundary
  can enforce them
- hostile lanes for changed, suppressed, denied, missing-signal,
  missing-bridge, policy-redacted, branch/preview, replay, and Worth-style
  consumer cases
- scale lanes proving anchor derivation, evidence-reference resolution, bridge
  envelope assembly, and materialization each have named slope counters

Must preserve:

- certification closes runtime-backed causal inspection only
- durable causal archives, persisted expanded narratives, and store-backed
  replay reconstruction remain explicit later-milestone debt
- compatibility escape hatches are named with exit criteria rather than
  normalized as alternate public APIs
- existing lower-runtime diagnostic and forensic facades may remain available to
  their owning crates, harnesses, and compatibility debt paths; 9.3.1 closes the
  ordinary public Query explanation path, not every internal debugging API

Exit criteria:

- the named 9.3.1 certification suite passes with canonical machine-checkable
  artifacts
- all remaining direct lower-runtime explanation paths are either removed from
  ordinary domain usage or tracked as compatibility debt
- scale rows prove inspection materialization cost follows evidence-reference
  width and requested richness, not unrelated runtime graph size
- no phase may be marked complete until all earlier phase closeout artifacts,
  counters, and proof-boundary tests are passing

## Must Ship

- Query inspection APIs and artifacts for cross-runtime causal explanations.
- A `worth-runtime-bridge`-owned causal explanation envelope that can carry
  relational authority digests, bridge route/evaluation/source/structural/
  stream/preview/writeback/replay digests, signal invalidation and evaluation
  references, signal forensic availability, lineage/provenance references,
  replay posture, and materialization policy.
- A Query-facing bridge adapter that calls the runtime-bridge facade and stores
  only sealed envelope identity, digest, receipt, denial, availability, and
  materialization posture in Query-owned artifacts.
- A canonical causal observation anchor binding every explanation to the Query
  operational artifact that produced the observed result, denial, suppression,
  preview, branch result, or replay result.
- Evidence-reference contracts that consume existing lower-runtime retained
  records instead of making Query the owner of a second diagnostics store.
- Explicit denial artifacts for missing bridge route evidence, missing signal
  evidence, incompatible relational authority, policy-redacted diagnostics, and
  unsupported explanation families.
- Advisory/narrowed artifacts for policy redaction, partial forensic
  availability, deferred durable replay, or unavailable optional evidence where
  the primary explanation remains valid but not full-richness.
- Cold-path materialization controls that distinguish digest/reference-only
  evidence from expanded narrative/debug detail.
- Support metadata and certification rows declaring which causal explanation
  families are runtime-backed now and which durable/store-backed replay
  families remain later milestone debt.
- Compile-fail or public-surface tests proving downstream domains cannot rely on
  direct runtime-bridge, relational, or signal imports as the ordinary
  explanation path.

## Existing Lower-Runtime Evidence To Consume

Milestone 9.3.1 must begin from the evidence surfaces that already exist rather
than treating cross-runtime diagnostics as greenfield work.

Runtime bridge evidence families:

- `BridgeDiagnosticsFacade::route_records`,
  `historical_evaluation_records`, `source_materialization_records`,
  `source_failure_records`, `continuity_records`, `merge_records`,
  `structural_remap_records`, `structural_branch_comparison_records`,
  `stream_checkpoints`, `stream_replay_records`, `preview_execution_records`,
  `preview_discard_records`, `preview_promotion_records`,
  `writeback_admission_records`, `writeback_execution_records`,
  `writeback_mapper_envelopes`, `writeback_mapped_family_inputs`,
  `writeback_mapper_records`, and `writeback_replay_records`
- bridge explanation constructors such as route, historical evaluation, source
  materialization/failure, structural remap/branch comparison, stream,
  preview, writeback candidate/admission/execution/mapper/outcome/replay, and
  preview replay explanations
- `BridgeProducerMetadata` writeback feedback provenance and causality digests
  carried on bridge patch envelopes

Query evidence families:

- existing `WORTHQueryInspection` targets and variants for live views, derived
  views, effects, write receipts, batch write receipts, intent receipts,
  intent denials, preview bindings/outcomes/receipts, and branch intent receipts
- `WORTHQueryWriteReceiptInspection` causality and provenance evidence derived
  from bridge mutation authority bundles
- `WORTHQueryRuntimeInspectionEvidence`, existing inspected artifacts, and
  receipt digests already emitted by Query's runtime inspection surface

Signal evidence families:

- signal observer explanation and replay cursor evidence
- forensic `diagnostics_for_graph(...).forensic()` materialization posture for
  explanation and provenance artifacts
- `DiagnosticsAvailability`, `ReplayCursor`, `LineageArtifactId`, execution
  record identity, semantic segment identity, and signal lineage/provenance
  availability

Relational evidence families:

- commit, snapshot, branch head, lineage, publication, decision, and durability
  evidence remain relational-owned
- Query causal inspection may reference relational authority identities and
  decision evidence, but it must not reopen relational truth as a fishing
  expedition during explanation materialization

The milestone may add adapters, indexes, and typed reference wrappers around
these surfaces. It must not build a parallel authority model that competes with
them.

## WORTH Proof Usage

`worth-proof` should be used where this milestone needs reusable progression
law, sealed proof minting, fixed-shape proof sets, authority witnesses, or
trust-boundary readmission. It should not be used as a diagnostics schema,
artifact store, runtime workflow engine, lineage/provenance model, or
performance-reporting substrate.

Use `worth-proof` for:

- `CausalInspectionProofFlow`: encode the ordered progression from observation
  anchor -> evidence references -> inspection request -> admitted/advisory/
  denied inspection -> bridge envelope readiness -> materialized Query artifact
  so later phases cannot be called with earlier proof states
- authority and capability witnesses for the boundaries that are allowed to
  mint `CausalObservationAnchor`, `CausalEvidenceReferenceSet`,
  `AdmittedCausalInspection`, `AdvisoryCausalInspection`,
  `DeniedCausalInspection`, and materialized artifact proof forms
- proof-carrying collections for evidence references when the property matters:
  non-empty evidence-family sets, canonical ordering, uniqueness, fixed arity,
  and disjoint authority-family bindings
- checked transition outcomes for admission and envelope readiness, preserving
  success/advisory/violation or success/deferred/stale/rebind-required/failure
  posture without collapsing into `Result<bool, Error>`
- trust-boundary readmission when evidence crosses from Query into runtime
  bridge envelope assembly and back into Query materialization
- compile-fail proof-shape tests proving raw strings, raw vectors, raw lower
  runtime handles, and stale proof states cannot satisfy later-phase APIs

Do not use `worth-proof` for:

- bridge, relational, signal, lineage, provenance, or diagnostics record
  storage
- expanded narrative/prose rendering
- performance counters or scale reports, except as proof-bearing closeout
  artifacts that carry already-computed counter digests
- domain-specific causal semantics that belong to Query, runtime bridge,
  relational, or signal

Required `worth-proof` evidence:

- `causal_inspection_proof_shape_digest`
- `causal_inspection_phase_progression_digest`
- `causal_inspection_witness_authority_digest`
- `causal_evidence_reference_collection_proof_digest`
- `causal_bridge_readmission_proof_digest`

## Performance Contract

Performance is a first-class part of Milestone 9.3.1, not a closeout garnish.
Every phase must expose counters at the boundary it adds and must prove its
cost slope before the next phase can depend on it.

Required slope claims:

- observation anchor derivation is bounded by the source receipt and evidence
  identity count carried by that receipt
- evidence-reference resolution is bounded by requested evidence-family count
  and indexed reference width, with zero ordinary-lane bridge diagnostics scan
  fallback
- Query inspection admission is bounded by request shape, policy/tenant
  posture, evidence-family set width, and decision-trace index writes/lookups
- bridge causal envelope assembly is bounded by resolved evidence-reference
  width and requested bridge evidence families
- Query materialization is bounded by bridge envelope width, redaction policy,
  materialization richness, and serialized artifact width
- certification scale rows must report slopes separately for anchor derivation,
  evidence-reference resolution, admission, bridge envelope assembly,
  redaction, materialization, and public artifact serialization

Required performance gates:

- no phase may introduce an API whose cheap-looking name hides a broad graph
  walk, diagnostics scan, replay reconstruction, or expanded narrative
  materialization
- every phase must record exact counter assertions in hostile, parity, and
  scale lanes
- materialization richness may increase cold-path work, but it may not widen
  hot-path query execution, bridge route selection, signal invalidation, or
  relational authority work
- compatibility lanes that still need scan fallback must expose the fallback as
  named debt with an exit condition and must not count as certified ordinary
  behavior

## Must Preserve

- Relational remains the authority for truth, commits, snapshots, and
  relational decision evidence.
- Runtime bridge remains the authority for bridge protocol, route, evaluation,
  writeback, preview, historical materialization, and cross-runtime envelope
  assembly.
- Signal remains the authority for observation, invalidation, scheduling,
  signal lineage, and signal provenance evidence.
- Query remains the authority for canonical query intent, public inspection
  admission, redaction, result-shape context, and consumer-facing artifacts.
- Diagnostic richness may change materialized detail, but it may not change
  query meaning, bridge route meaning, signal invalidation meaning, or
  relational authority.
- Hot-path query execution and signal invalidation may emit bounded references
  and counters, but expanded explanation materialization stays on the cold path.
- Query may be the ordinary public door, but it is never the bridge causal
  authority. Public convenience cannot collapse runtime-bridge facade calls,
  bridge diagnostics indexing, and Query artifact materialization into one
  Query-owned implementation.

## Compile-Time Enforcement Policy

Milestone 9.3.1 must classify its boundaries by enforcement strength.

`Unrepresentable` in public types:

- a causal inspection request without a causal observation anchor, target, basis
  posture, inspection reason, requested evidence-family set, and richness policy
- a causal observation anchor that is not derived from exactly one Query
  operational artifact or receipt
- a causal evidence reference set that claims lower-runtime evidence ownership
  rather than reference ownership
- an admitted causal inspection without an admission receipt and evidence-family
  allowlist
- an advisory causal inspection without explicit narrowed/redacted/deferred
  posture and decision-trace evidence
- a bridge causal envelope without bridge route/evaluation evidence ownership,
  relational authority references, signal evidence references, existing bridge
  diagnostic family references, lineage, provenance, replay posture, forensic
  availability, and materialization posture
- a Query causal inspection artifact that does not distinguish admitted from
  advisory from denied inspection
- an expanded narrative artifact that is not derived from a machine-checkable
  causal inspection artifact
- a support claim that promotes durable/store-backed replay from deferred to
  runtime-backed support

`Uncompilable` through visibility and compile-fail enforcement:

- external construction of `CausalObservationAnchor`,
  `CausalEvidenceReferenceSet`, `AdmittedCausalInspection`,
  `AdvisoryCausalInspection`, bridge-owned
  `BridgeCausalExplanationEnvelope`,
  `AdmittedQueryCausalInspectionArtifact`,
  `AdvisoryQueryCausalInspectionArtifact`,
  `DeniedQueryCausalInspectionArtifact`, or
  `CausalInspectionCertificationBundle`
- any `worth-query` public export that makes bridge-owned `BridgeCausal*`
  envelope, binding, receipt, index, or constructor types look Query-owned
- ordinary public domain lanes constructing causal explanations from
  `BridgeDiagnosticsFacade`, `RelationalRuntime`, or `SignalGraph` instead of
  Query inspection artifacts, wherever crate visibility, dependency topology,
  facade exports, or lint rules can enforce that boundary
- public APIs that accept raw lower-runtime handles as causal-inspection inputs
- public APIs that expose mutable evidence references, mutable redaction
  policy, or mutable causal artifact internals
- public APIs that accept booleans such as `changed`, `supported`,
  `redacted`, `explained`, or `complete` instead of typed outcome/posture enums
- public APIs that turn missing bridge/signal/relational evidence into a
  successful artifact with optional empty fields

`Construction-time rejection`:

- inspection request whose query artifact and observation target do not share a
  basis or result-shape context
- inspection request whose observation anchor cannot prove one canonical Query
  operation/result/denial/suppression/replay artifact
- inspection request whose policy/tenant posture denies the requested richness
  or evidence family
- bridge causal envelope assembled from mismatched route, evaluation,
  relational authority, or signal evidence digests
- bridge causal envelope whose signal evidence does not bind to the query
  observation or invalidation family being explained
- Query materialization that would flatten lower-runtime authority names into
  Query-owned facts
- redaction request that would change causal identity rather than only detail
  richness
- certification scope that lacks changed, suppressed, denied, branch/preview,
  replay, missing-evidence, and public-boundary rows
- causal inspection assembly whose cost posture requires an unbounded scan of
  unrelated runtime graph state
- causal envelope assembly that matches lower-runtime records by heuristic
  digest search instead of consuming identities carried by the observation
  anchor and evidence reference set

## Complexity / Proof Obligations

- Prove that the same query observation can be explained through Query without
  lower-runtime spelunking by a domain consumer.
- Prove that bridge, relational, and signal evidence digests agree with the
  lower-runtime records they summarize.
- Prove that no-op and suppression cases carry causal explanation with the same
  rigor as changed-result cases.
- Prove that branch, preview, and historical basis changes remain visible in the
  explanation envelope.
- Prove that missing evidence fails as typed diagnostic denial and redacted or
  narrowed evidence becomes typed advisory detail rather than best-effort
  narrative output.
- Prove that diagnostic richness policy widens only cold-path materialization,
  not live invalidation, planning, or subscription semantics.
- Prove that Worth-style consumers can delete local stitching of bridge facade,
  signal graph, and relational runtime access once the public Query inspection
  artifact exists.
- Prove that evidence references are resolved through anchor-bound identities or
  indexed lower-runtime records, never by unbounded scans or best-effort digest
  matching.
- Prove that success, advisory, and violation admission outcomes preserve
  actionable context and never collapse into a boolean supported/unsupported
  wall.

Named contracts:

- `CausalObservationAnchorContract`
  - bounded by one Query operational artifact, one basis posture, one
    observation target, one result-shape context, one observation outcome, and
    one lower-runtime evidence reference digest
- `CausalInspectionAdmissionContract`
  - bounded by one causal observation anchor, one inspection target, one basis
    posture, one policy/tenant context, one richness policy, one
    evidence-family request, one decision trace index, and one admission receipt
- `CausalEvidenceReferenceContract`
  - bounded by the semantic delta that produced the observation plus the
    lower-runtime evidence families admitted by policy
- `BridgeCausalEnvelopeContract`
  - owned by `worth-runtime-bridge` and bounded by one admitted or advisory
    inspection summary, one bridge evidence-family set, one relational
    authority reference set, one signal evidence reference set, one forensic
    availability posture, and one replay/materialization posture
- `QueryCausalMaterializationContract`
  - bounded by one sealed runtime-bridge facade envelope result, one redaction
    policy, one materialization policy, one result-shape context, and one
    materialization receipt
- `CausalInspectionCertificationContract`
  - bounded by one certification scope, one admitted explanation-family matrix,
    one hostile coverage set, and one scale-counter snapshot

Required counters:

- `causal_observation_anchor_count`
- `causal_observation_anchor_denial_count`
- `causal_evidence_reference_count`
- `causal_evidence_reference_width`
- `causal_evidence_reference_resolution_count`
- `causal_inspection_proof_transition_count`
- `causal_inspection_proof_outcome_count`
- `causal_inspection_proof_readmission_count`
- `causal_inspection_request_count`
- `causal_inspection_admission_count`
- `causal_inspection_advisory_count`
- `causal_inspection_denial_count`
- `causal_decision_trace_lookup_count`
- `causal_decision_trace_index_hit_count`
- `bridge_causal_envelope_request_count`
- `bridge_causal_envelope_admitted_count`
- `bridge_causal_envelope_denial_count`
- `causal_bridge_record_index_lookup_count`
- `causal_bridge_record_scan_fallback_count`
- `relational_authority_evidence_reference_count`
- `bridge_route_evidence_reference_count`
- `bridge_evaluation_evidence_reference_count`
- `bridge_source_materialization_evidence_reference_count`
- `bridge_structural_evidence_reference_count`
- `bridge_stream_evidence_reference_count`
- `bridge_preview_evidence_reference_count`
- `bridge_writeback_evidence_reference_count`
- `signal_invalidation_evidence_reference_count`
- `signal_evaluation_evidence_reference_count`
- `signal_forensic_availability_reference_count`
- `lineage_evidence_reference_count`
- `provenance_evidence_reference_count`
- `causal_materialization_request_count`
- `causal_materialization_redaction_count`
- `causal_materialization_denial_count`
- `causal_materialization_expanded_detail_count`
- `causal_inspection_boundary_violation_denial_count`
- `causal_inspection_certification_row_count`
- `causal_inspection_hostile_row_count`
- `causal_inspection_scale_fixture_row_count`
- `causal_inspection_scale_slope_digest_part_count`
- `causal_anchor_derivation_slope_counter`
- `causal_reference_resolution_slope_counter`
- `causal_admission_slope_counter`
- `causal_bridge_envelope_slope_counter`
- `causal_materialization_slope_counter`
- `causal_artifact_serialization_slope_counter`

Counter rules:

- exact counter assertions are required for certification rows
- every phase must add or reuse a named counter boundary before the next phase
  may consume its artifact
- evidence reference width may grow with causal evidence families, not unrelated
  runtime graph size
- materialization cost may grow with requested richness and admitted evidence
  width, not with hidden lower-runtime rescans
- bridge record scan fallback count must be zero for certified ordinary
  explanation lanes; compatibility debt lanes must name why an index is absent
- redaction rows must assert stable causal identity and changed redaction
  detail digests
- missing-evidence rows must deny before public admitted artifact emission
- public-boundary rows must prove direct lower-runtime stitching is absent from
  ordinary domain explanation code

## Allowed Debt

- Durable causal explanation archives, restart-stable expanded narratives, and
  store-backed replay reconstruction may remain explicit debt until Milestones
  10 and 11 close.
- Domain-specific prose renderers may remain local to domains, provided they
  consume the Query causal inspection artifact rather than lower-runtime
  internals.

## Forbidden Debt

- A public Query inspection artifact that omits which lower runtime owns each
  evidence family.
- Production `BridgeCausal*`, bridge envelope index, bridge evidence binding,
  bridge envelope receipt, bridge envelope denial, or bridge causal performance
  counter definitions inside `worth-query`.
- Query-owned retained indexes over runtime-bridge route/evaluation/source/
  structural/stream/preview/writeback/replay/continuity/merge/mapper records.
- Query materialization that reconstructs bridge causal facts from lower-runtime
  records instead of consuming the sealed runtime-bridge envelope.
- Direct Worth or domain imports of runtime-bridge diagnostics, relational
  runtimes, or signal graphs as the ordinary explanation path.
- Diagnostic strings that cannot be traced back to machine-checkable causal
  observation anchors and evidence references.
- Hot-path expanded explanation materialization hidden inside query execution
  or signal invalidation.

## Proposed Module Topology

Milestone 9.3.1 should extend the existing Query inspection surface before it
creates any new top-level public module. `crates/worth-query/src/runtime/`
already owns runtime inspection entry points and `WORTHQueryInspection`
variants; causal inspection should either extend that surface directly or place
a narrow public facade above it. It must not create a second unrelated
inspection vocabulary.

Expected Query-side subdomains:

- `runtime/inspection/causal/request.rs`
  - public causal inspection request, target, reason, richness policy, and
    admission subject vocabulary
- `runtime/inspection/causal/anchor.rs`
  - causal observation anchors, observation receipt adapters, and evidence
    reference digests
- `runtime/inspection/causal/admission.rs`
  - success/advisory/violation admission decisions, admitted/advisory/denied
    inspection proof types, decision trace indexes, and admission receipts
- `runtime/inspection/causal/artifact.rs`
  - admitted, advisory, and denied Query causal inspection artifacts plus
    redaction and materialization receipts derived from sealed bridge envelope
    results
- `runtime/inspection/causal/certification.rs`
  - certification scope, hostile row coverage, and final certification bundle
- `runtime/causal_evidence.rs`
  - evidence references, evidence-family vocabulary, counters, and evidence
    reference receipts
- `runtime/inspection/causal/bridge_adapter.rs` or `runtime/causal_bridge.rs`
  - Query-facing runtime-bridge facade adapters that carry bridge envelope
    identity, digest, receipt, denial, and availability posture without owning
    bridge semantics or retained bridge indexes

Expected runtime-bridge-side subdomains:

- `diagnostics/causal_envelope.rs`
  - bridge-owned causal envelope, evidence bindings, and bridge envelope
    receipts, denials, counters, sealed constructors, and facade result types
- `diagnostics/causal_mapping.rs`
  - mapping from existing route, evaluation, historical, preview, writeback,
    source, structural, stream, mapper, replay, and continuity records into
    causal evidence bindings
- `diagnostics/causal_index.rs` or the existing bridge diagnostics index
  surface
  - bridge-owned indexed lookup over retained bridge records used by causal
    envelope assembly
- `facade` extension for causal envelope assembly
  - public bridge entrypoint Query calls after Query admission; not a Query
    module and not a domain escape hatch

Expected tests:

- `runtime/tests/inspection/causal_explanation.rs`
- `runtime/tests/inspection/causal_redaction.rs`
- `runtime/tests/inspection/causal_boundary.rs`
- `runtime/tests/inspection/causal_scale.rs`
- runtime-bridge tests for bridge-owned causal envelope assembly, mapping,
  denial, indexing, counters, and facade boundaries
- compile-fail tests proving external callers cannot construct causal proof
  artifacts or ordinary explanations from lower-runtime internals

Topology rules:

- causal inspection must reuse or deliberately extend `WORTHQueryInspection`
  rather than making a competing inspection enum
- admission logic must not live inside artifact formatting
- bridge envelope assembly must not live inside Query public artifact
  materialization
- bridge envelope assembly must live in `worth-runtime-bridge`; if the
  implementation creates a Query file that can assemble bridge evidence
  bindings, the implementation is out of spec
- Query bridge adapters must depend on the runtime-bridge facade, never on
  runtime-bridge internal diagnostics modules
- redaction and richness policy must not live inside lower-runtime evidence
  collection
- certification harness code must not become a second construction path for
  proof-bearing runtime artifacts
- compatibility code that still touches direct bridge, relational, or signal
  facades must live in an explicitly named compatibility/audit lane with exit
  criteria

## Representative Scenario Matrix

Minimum canonical rows:

- `changed-result-causal-envelope`
- `suppressed-result-causal-envelope`
- `query-denied-before-bridge-envelope`
- `advisory-redacted-causal-envelope`
- `branch-preview-causal-envelope`
- `historical-replay-causal-envelope`
- `policy-redacted-causal-materialization`
- `worth-style-query-only-consumer-explanation`
- `bridge-route-and-signal-evidence-bind-same-observation`
- `observation-anchor-binds-one-query-receipt`
- `bridge-source-structural-stream-preview-writeback-records-bind-through-existing-diagnostics`
- `signal-forensic-availability-and-replay-cursor-bind-through-query-artifact`
- `causal-richness-does-not-change-query-meaning`
- `causal-inspection-scale-honesty`

Minimum rejection rows:

- `missing-bridge-route-evidence-denied`
- `missing-signal-invalidation-evidence-denied`
- `missing-signal-evaluation-evidence-denied`
- `relational-authority-mismatch-denied`
- `redaction-policy-overclaim-denied`
- `unsupported-explanation-family-denied`
- `direct-bridge-diagnostics-domain-explanation-forbidden`
- `direct-relational-runtime-domain-explanation-forbidden`
- `direct-signal-graph-domain-explanation-forbidden`
- `durable-causal-archive-overclaim-forbidden`
- `store-backed-replay-reconstruction-overclaim-forbidden`

Every representative row must identify:

- query digest
- query observation receipt digest
- causal observation anchor digest
- inspection target digest
- inspection request digest
- admitted, advisory, or denied inspection digest
- causal envelope digest where admitted
- relational authority digest where present
- bridge route/evaluation digests where present
- bridge source/structural/stream/preview/writeback/replay digests where
  present
- signal invalidation/evaluation digests where present
- signal forensic availability digest where present
- lineage and provenance digests where present
- materialization policy digest
- redaction policy digest
- replay posture digest
- decision trace index digest
- failure digest where rejected
- counter snapshot

## Sequencing Notes

This belongs after Milestone 9.3 because subscription family diagnostics prove
Query can explain automatic bridge lowering for live query families. Milestone
9.3.1 then widens that proof from subscription selection to the general
cross-runtime causal explanation boundary that the public runtime API must
stabilize before downstream domains build against it.

This belongs before the Runtime API Public Stabilization Gate because the
inspection contract is part of the ordinary public API. Freezing the public API
before this boundary is named would normalize direct lower-runtime explanation
access as domain glue.

## Store Dependency

Runtime-backed causal diagnostics are not blocked on `worth-store` when they
operate over retained runtime, bridge, relational, and signal evidence already
available in memory or runtime-backed artifacts.

Store-backed reconstruction, durable replay across restart, archived expanded
inspection narratives, and persisted causal-envelope reload remain Milestone 10
and Milestone 11 scope.

## Explicit Assumptions And Deferred Decisions

- 9.3.1 assumes existing Query inspection, mutation evidence, subscription
  diagnostics, bridge diagnostics, and runtime-bridge route/evaluation/source/
  structural/stream/preview/writeback/replay records remain canonical enough to
  bind into a shared causal envelope.
- 9.3.1 assumes `worth-runtime-bridge` may need a new causal envelope surface,
  but Query owns the public inspection request and materialized public artifact.
- 9.3.1 assumes `worth-signal` already owns or can expose retained
  invalidation, evaluation, lineage, provenance, and replay references without
  Query becoming the signal diagnostic authority.
- 9.3.1 assumes `worth-relational` remains the source of truth for commits,
  snapshots, branch/head identity, and relational decision evidence.
- The first implementation may be digest/reference-heavy rather than expanding
  full lower-runtime narratives, provided the artifact is sufficient for
  machine-checkable certification and later cold-path expansion.
- Durable causal archives, persisted expanded narratives, store-backed replay
  reconstruction, cross-process causal-envelope reload, and restart-stable
  causal artifact materialization remain later milestone debt.
- Domain-specific prose renderers may remain domain-owned, but they must
  consume Query causal inspection artifacts rather than lower-runtime internals.

## Explicit Failure Taxonomy

Milestone 9.3.1 must preserve at least these failure classes:

- unsupported causal explanation family
- inspection target and query artifact basis mismatch
- inspection target and result-shape context mismatch
- policy or tenant richness denial
- bridge route evidence missing
- bridge evaluation evidence missing
- relational authority evidence missing
- relational authority digest mismatch
- signal invalidation evidence missing
- signal evaluation evidence missing
- signal lineage evidence missing
- signal provenance evidence missing
- replay posture unsupported
- redaction policy overclaim
- materialization policy overclaim
- lower-runtime authority flattening denied
- direct lower-runtime explanation path forbidden
- durable causal archive overclaim
- store-backed replay reconstruction overclaim
- causal inspection scale contract violation

## Parallelization Notes

Implementation phases must close in order. Parallel work is allowed only for
non-production scaffolding that cannot be consumed until its prerequisite phase
has passed.

- During Phase 1, bridge-side causal envelope design may sketch fixture-only
  evidence-family names in the runtime-bridge work area, but Query production
  code may not define bridge envelope or bridge binding types. Production bridge
  envelope code must wait for Phase 2 evidence-reference resolution and
  performance indexes.
- During Phase 2, admission fixtures may be drafted against fake anchors, but
  production Phase 3 admission must consume the real Phase 1 anchor and Phase 2
  reference proof types.
- During Phase 3, bridge envelope fixtures may be prepared, but production Phase
  4 assembly must be implemented in `worth-runtime-bridge` and must consume
  admitted/advisory proof summaries plus resolved evidence references.
- During Phase 4, Query artifact renderers may be prototyped against fixture
  envelopes, but production Phase 5 materialization must consume bridge-owned
  causal envelopes and performance counters returned through the runtime-bridge
  facade. Fixture envelopes are not allowed to become Query-local production
  bridge envelope types.
- During Phase 5, certification rows may be scaffolded, but Phase 6 cannot close
  until every prior phase has production artifacts, exact counters,
  compile-fail/proof-shape tests, and scale rows.

## Acceptance Evidence

This milestone is complete only when `worth-query` can prove:

- the `Cross-Runtime Causal Explanation Envelope Test` in
  [test-requirements.md](./test-requirements.md) passes with canonical
  machine-checkable artifacts
- the runtime-bridge causal envelope authority is implemented in
  `worth-runtime-bridge`, exposed through the runtime-bridge facade, and
  consumed by Query only as a sealed facade result
- Query inspection can explain changed, suppressed, denied, branch/preview, and
  replayed observations through one typed public artifact
- the bridge-owned causal envelope carries evidence digests for relational
  authority, bridge route/evaluation/source/structural/stream/preview/writeback/
  replay, signal invalidation/evaluation, signal forensic availability,
  lineage, provenance, replay posture, and materialization policy
- public Query artifacts preserve lower-runtime authority names instead of
  flattening them into Query-owned narrative facts
- downstream domains can consume causal explanations without direct imports from
  runtime bridge diagnostics, relational runtime internals, or signal graph
  internals
- direct lower-runtime explanation paths remaining in domains or certification
  programs are either removed from ordinary usage or named compatibility debt
  with exit criteria
- every causal explanation binds to one Query observation anchor derived from an
  existing operational receipt or artifact
- Query causal evidence references resolve through existing lower-runtime
  retained records, Query inspection evidence, signal forensic availability, or
  relational authority identities rather than through new Query-owned
  diagnostics authority
- compile-fail or public-surface tests prove `worth-query` does not export
  production `BridgeCausal*` envelope constructors, bridge evidence binding
  constructors, bridge envelope indexes, or bridge causal receipt constructors
- runtime-bridge boundary tests prove bridge causal envelope constructors are
  sealed to the bridge-owned assembly path
- Query materialization tests prove public causal artifacts consume a sealed
  runtime-bridge facade envelope rather than Query-local bridge envelope types

Required verification output must include:

- `query_digest`
- `query_observation_receipt_digest`
- `causal_observation_anchor_digest`
- `inspection_target_digest`
- `inspection_request_digest`
- `admitted_inspection_digest`
- `advisory_inspection_digest`
- `denied_inspection_digest`
- `causal_evidence_reference_digest`
- `causal_envelope_digest`
- `relational_authority_digest`
- `bridge_route_digest`
- `bridge_evaluation_digest`
- `bridge_source_materialization_digest`
- `bridge_structural_digest`
- `bridge_stream_digest`
- `bridge_preview_digest`
- `bridge_writeback_digest`
- `signal_invalidation_digest`
- `signal_evaluation_digest`
- `signal_forensic_availability_digest`
- `signal_lineage_digest`
- `signal_provenance_digest`
- `decision_trace_index_digest`
- `query_observation_digest`
- `materialization_policy_digest`
- `redaction_policy_digest`
- `replay_posture_digest`
- `admitted_causal_inspection_artifact_digest`
- `advisory_causal_inspection_artifact_digest`
- `denied_causal_inspection_artifact_digest`
- `causal_materialization_receipt_digest`
- `causal_inspection_boundary_audit_digest`
- `causal_inspection_proof_shape_digest`
- `causal_inspection_phase_progression_digest`
- `causal_inspection_witness_authority_digest`
- `causal_evidence_reference_collection_proof_digest`
- `causal_bridge_readmission_proof_digest`
- `failure_digest`
- `counter_snapshot`
- `causal_inspection_scale_slope_digest`
- `causal_anchor_derivation_slope_digest`
- `causal_reference_resolution_slope_digest`
- `causal_admission_slope_digest`
- `causal_bridge_envelope_slope_digest`
- `causal_materialization_slope_digest`
- `causal_artifact_serialization_slope_digest`
- `compile_fail_boundary_digest`

## Self-Check

This milestone solves a real structural problem rather than packaging work
cosmetically because it closes the public explanation boundary between Query,
runtime bridge, relational, and signal. Without it, domains can still build
powerful explanations by stitching lower-runtime internals directly.

The adversarial constraint is load-bearing because it forbids the easy version:
debug strings, direct bridge diagnostics, raw relational snapshots, or signal
graph spelunking that produce explanations outside Query's public inspection
contract.

The milestone preserves crate authority boundaries because Query owns request,
admission, redaction, materialization, and public artifacts; runtime bridge owns
the cross-runtime causal envelope; relational owns truth authority; signal owns
observation, invalidation, lineage, and provenance evidence.

The milestone defines proof obligations rather than implementation chores
because it requires phase types, sealed construction, hostile certification
rows, exact counters, typed denial, `worth-proof`-style progression, and
compile-fail boundaries.

A competent engineer should be able to map this spec into honest modules and
tests because each phase names its input, output, ownership boundary,
performance counters, forbidden shortcuts, and acceptance evidence, and because
the phases are ordered by production dependency rather than abstract taxonomy.

The milestone belongs after 9.3 because bridge-honest subscription diagnostics
provide the narrow live-query explanation lane first. It belongs before 9.3.2
through 9.3.6 and before the Runtime API Public Stabilization Gate because
Query's public inspection contract must be named before the facade is frozen.

## Closeout Standard

Milestone 9.3.1 is closed only when:

- the named 9.3.1 certification suite exists and passes
- Query exposes public causal inspection artifacts for admitted, advisory, and
  denied explanations
- bridge-owned causal envelopes are implemented in `worth-runtime-bridge`,
  returned through the runtime-bridge facade, and bind route/evaluation,
  relational authority, source/structural/stream/preview/writeback/replay
  records, signal invalidation/evaluation, forensic availability, lineage,
  provenance, replay, and materialization evidence
- Query materializes only sealed bridge facade envelope results and does not
  define or export production `BridgeCausal*` envelope authority
- redaction and richness policy are cold-path materialization concerns only
- Worth-style consumers can explain ordinary query observations without direct
  lower-runtime stitching
- compile-fail boundaries prevent external construction of proof-bearing causal
  inspection artifacts
- proof-shape certification proves phase skipping, raw collection substitution,
  stale proof reuse, and WORTHd authority witnesses are impossible or rejected
- performance certification proves anchor derivation, reference resolution,
  admission, bridge envelope assembly, materialization, and artifact
  serialization each obey their named slope counters
- durable/store-backed causal replay claims remain explicit later-milestone
  debt
