# WORTH Query And Bridge Authoritative Mutation Evidence Plan Appendix

This appendix carries the longer-form proof obligations and guardrails that
support the main plan in
[runtime-authoritative-mutation-evidence-plan.md](./runtime-authoritative-mutation-evidence-plan.md).

## Governing Summaries

- `MENTALITY.md`: protects solving the hard structural problem first. Here, the
  hard problem is not "support one more write helper"; it is making authority
  mutation evidence explicit before more domains depend on weaker semantics.
- `arch_laws.md`: protects dual write/read contracts, authority/derivation
  separation, batch-derived summaries, self-describing envelopes, and facade
  honesty. Mutation meaning must be preserved by the runtime contract, not
  reverse-engineered by domains from patches, logs, or lower-crate artifact
  spelunking.
- `perf_laws.md`: protects semantic-delta-bounded work and boundary-cardinality
  honesty. Bulk authoritative imports and edit batches must not be fragmented
  into scalar explanations that hide breadth or lose target identity.
- `domain_laws.md`: protects responsibility-shaped modules and honest naming.
  This work belongs in generic runtime mutation/inspection modules, not in a
  Worth-shaped adapter layer.
- `worth_query_vision.md`: protects Query as the daily-driver platform surface
  for asking for truth and orchestrating domain work over truth. If writes are
  part of the daily-driver story, their authority evidence, causality, and
  provenance have to be first class.
- `worth_query_roadmap.md`: protects the rule `declare query intent once,
  lower it once, execute it against canonical truth`. Mutation authoring and
  authoritative outcome evidence must obey the same rule.
- `test-requirements.md`: protects canonical, adversarial certification
  artifacts. This gate needs named suites for authoritative mutation evidence,
  not just happy-path CRUD tests.
- `worth_runtime_bridge_vision.md`: protects the bridge as a causal protocol
  boundary, not incidental glue. If Query promises lineage/provenance for free,
  the bridge must preserve that meaning through the runtime seam.
- `worth_runtime_bridge_roadmap.md`: protects bridge-owned causality transfer,
  writeback protocol meaning, and replay-safe cross-runtime artifacts. This
  gate must consume those strengths and harden any missing carry-forward seam.
- `worth-runtime-bridge/test-requirements.md`: protects canonical,
  machine-checkable bridge bundles for causality, replay, writeback, temporal,
  and async boundaries. This gate must not define a Query-side evidence story
  the bridge cannot certify.
- `worth-runtime-bridge/milestone-12.md`: protects bridge-mediated writeback,
  replay-safe provenance, idempotence, and loop prevention as first-class bridge
  protocol surfaces. This gate must reuse and extend that contract, not dilute
  it.
- `worth-runtime-bridge/milestone-13.md`: protects one canonical end-to-end
  causality bundle story across routing, evaluation, writeback, and replay.
  This gate must make Query consume that story rather than inventing a second
  public provenance model.
- `aspect-api-finalization-closeout.md`: protects aspect-native mutation as the
  ordinary public story. This plan deepens that story rather than replacing it.
- `runtime-api-public-stabilization-closeout.md`: protects the stable public
  facade and inspectable handles. This plan hardens the mutation side of that
  same facade so downstream write-heavy domains can rely on it honestly.
- `worth/worth-query-runtime-rewrite-plan.md`: protects the clean-break rule
  that Worth must harden Query instead of working around it. Worth Phase 6 is
  the forcing function, not the owner of the fix.

## Concrete Artifact Boundary

This gate must not stay abstract. It needs a closed family of proof-bearing
runtime artifacts that a downstream engineer could point to in code and tests.

At minimum, the final public surface should converge on materially equivalent
artifacts to:

- `WORTHQueryMutationTargetEvidence`
- `WORTHQueryMutationCausalityEvidence`
- `WORTHQueryMutationProvenanceEvidence`
- `WORTHQueryExistingTruthBinding`
- `WORTHQueryBatchMutationEvidence`
- `WORTHQueryNamingMutationEvidence`
- `WORTHQueryContinuityMutationEvidence`
- `WORTHQueryWriteReceiptInspection`
- `WORTHQueryBatchWriteComponentInspection`

At minimum, the bridge-side carry-forward surface should converge on materially
equivalent artifacts to:

- `BridgeMutationCausalityBundle`
- `BridgeMutationProvenanceBundle`
- `BridgeExistingTruthBindingArtifact`
- `BridgeNamingWritebackEvidence`
- `BridgeContinuityMutationEvidence`
- `BridgeWritebackOutcomeProvenance`

The exact Rust names may differ, but the artifact responsibilities may not blur.
If one type tries to carry target classification, causality, provenance,
naming, continuity, denial, and batch aggregation all at once, the spec has
collapsed too many ontologies into one bag.

## Compile-Time Enforcement Policy

This gate must make the wrong thing hard or impossible, not merely discouraged.

`Unrepresentable` in public types:

- a mutation receipt or inspection artifact that exposes touched aspects but not
  whether the target was declared, resolved, or denied
- a naming-aware or continuity-aware mutation family encoded as open-ended
  strings, bools, or generic metadata bags instead of closed evidence families
- a batch/session summary that can exist without a canonical relationship to
  component receipts
- a causality/provenance surface that can omit source declaration identity,
  authority-lane transition identity, or denial class while still claiming to
  be complete authority evidence

`Uncompilable` through privacy and compile-fail enforcement:

- external construction of proof-bearing mutation evidence artifacts without the
  runtime-owned lowering path
- public APIs that accept raw lower-runtime mutation provenance blobs as though
  they were admitted public Query evidence
- public mutation APIs that let callers override resolved target class,
  continuity class, or naming outcome after runtime lowering
- public shortcut paths that encode existing-truth identity binding as ad hoc
  `String` reuse with no admitted binding artifact

`Construction-time rejection`:

- unsupported existing-truth binding families
- unsupported symbolic same-batch reference classes
- unsupported naming-writeback families
- unsupported continuity-sensitive families
- invalid target-class pairings between authored mutation family and resolved
  lower-runtime target
- attempts to cross from ordinary mutation into naming/continuity semantics
  without the explicit admitted family

Compile-fail coverage must prove at minimum:

- no public forging of mutation evidence artifacts
- no bool-driven "is continuity-aware" or "is naming-aware" shortcuts
- no public consumption of raw lower-runtime provenance bags
- no public target-class override after lowering

## Representative Scenario Matrix

This gate must certify concrete authority lanes, not just abstract categories.

At minimum, the closeout matrix should include:

- `new-target-insert-with-causality`:
  one ordinary insert carries declared target class, resolved target class,
  source declaration identity, and provenance bundle
- `existing-target-update-with-binding`:
  one update lowers through admitted existing-truth binding and preserves why
  the authoritative target was selected
- `existing-target-delete-with-clear-fallout`:
  one delete preserves target evidence, touched-aspect fallout, and delete
  causality together
- `mixed-batch-symbolic-and-existing-targets`:
  one ordered batch mixes same-batch symbolic references and existing-truth
  bindings without collapsing their evidence into one scalar receipt
- `authoritative-import-session-aggregate`:
  one import/session proves aggregate authority evidence is not just the final
  component re-labeled
- `naming-attachment-rebind`:
  one admitted naming-aware mutation preserves attach-vs-rebind outcome and
  provenance
- `continuity-aware-rebind-or-denial`:
  one continuity-sensitive mutation either preserves a continuity evidence chain
  or yields one typed denial with explicit reason
- `unsupported-existing-target-family-denied`:
  one hostile lane requests an unsupported binding family and fails before
  execution
- `unsupported-naming-family-denied`:
  one hostile lane requests naming-aware behavior on an unadmitted family and
  fails typed and early
- `unsupported-continuity-family-denied`:
  one hostile lane requests continuity-sensitive mutation on an unadmitted
  family and fails typed and early

## Must Ship

- generic public target evidence vocabulary for direct writes and batches
- explicit declared-versus-resolved target evidence in receipts and inspection
- first-class causality and provenance bundles on receipts, inspection, and
  batch/session summaries
- aggregate batch/session evidence for authoritative mutation sessions
- admitted existing-truth identity-binding contract with typed denials
- naming-aware mutation evidence sufficient for projected naming writeback
- continuity-aware mutation evidence sufficient for future lineage-sensitive
  domains
- support-matrix rows and certification suites covering the new surfaces
- a downstream dependency contract closeout
- bridge-side carry-forward artifacts and replay-safe causality/provenance
  guarantees sufficient for Query to expose the same meaning publicly

## Must Preserve

- aspect-native CRUD remains the ordinary public mutation story
- touched-aspect fallout remains explicit and auditable
- causality and provenance remain runtime-carried evidence rather than optional
  domain-local adornment
- lower runtimes remain authoritative for truth, naming, writeback, and
  lineage semantics
- bridge-owned causality transfer remains the only admitted cross-runtime
  carry-forward path for this evidence family
- Query remains the facade and explanation carrier, not a second mutation
  engine
- unsupported families fail closed rather than degrading into best-effort
  target recovery

## Complexity / Proof Obligations

This gate must make authority evidence cost visible and testable.

Minimum named counters should include materially equivalent versions of:

- `mutation_target_evidence_count`
- `mutation_resolved_target_count`
- `mutation_existing_truth_binding_count`
- `mutation_symbolic_target_reference_count`
- `mutation_batch_component_count`
- `mutation_batch_aggregate_evidence_count`
- `mutation_causality_edge_count`
- `mutation_provenance_record_count`
- `mutation_naming_evidence_count`
- `mutation_continuity_evidence_count`
- `mutation_binding_denial_count`
- `mutation_naming_denial_count`
- `mutation_continuity_denial_count`
- `mutation_evidence_rediscovery_count`

Rules:

- counters belong on receipts, inspection bundles, certification bundles, or
  equivalent canonical artifacts, not only on debug logs
- `mutation_evidence_rediscovery_count` must be exactly zero on admitted paths
- every denied existing-truth binding must increment
  `mutation_binding_denial_count`
- every denied naming-aware family must increment
  `mutation_naming_denial_count`
- every denied continuity-aware family must increment
  `mutation_continuity_denial_count`
- aggregate batch/session counters must be derived once at the session boundary,
  not recomputed independently by each consumer

## Acceptance Evidence

This plan is complete only when `worth-query` can prove:

- direct insert/update/delete/batch receipts preserve target evidence and
  touched-aspect evidence together
- direct insert/update/delete/batch receipts preserve causality/provenance
  alongside target evidence rather than flattening them into incidental metadata
- authoritative and preview receipts expose the same target-evidence concepts
- batch/session inspection preserves aggregate authority evidence honestly
- existing-truth binding failures deny typed and early
- naming-aware and continuity-aware mutation families either preserve explicit
  outcome evidence or deny typed and early
- lineage/provenance and authority-causality breadcrumbs survive the public
  runtime boundary without downstream reconstruction
- bridge replay and diagnostics artifacts agree with the Query-facing receipt
  and inspection story rather than exposing a second incompatible provenance
  model
- support metadata, inspection bundles, and executable admission behavior stay
  in sync
- downstream domain tests can delete local target-recovery glue rather than
  merely wrapping it

Required verification commands at closeout:

- `cargo fmt -p worth-query`
- `cargo check -p worth-query --tests`
- `cargo test --manifest-path crates/worth-query/Cargo.toml --test phase_boundaries_compile_fail`
- `cargo test -p worth-query`
- targeted mutation evidence certification tests
- targeted runtime public support tests
- `cargo fmt -p worth-runtime-bridge`
- `cargo check -p worth-runtime-bridge --tests`
- `cargo test -p worth-runtime-bridge`
- targeted bridge causality/writeback certification tests
- `git diff --check`

## Roadmap Placement

This belongs immediately after the Runtime API Public Stabilization Gate on the
Query side, and as an explicit cross-reference hardening gate over the already
claimed bridge writeback/causality surfaces before downstream domains rely on
them as one end-to-end contract.

Why here:

- the public facade is already stabilized enough for downstream domains to use
  now
- the next serious blocker is not temporal/async semantics; it is authority-
  mutation honesty for write-heavy domains
- Worth Phase 6 is already surfacing this gap in a way that other domains will
  also hit around writeback, naming, and continuity
- the bridge already claims causality-transfer and writeback strength, so the
  honest next step is to bind that bridge promise to the Query public contract
  explicitly instead of leaving a soft seam between them
- later temporal/async milestones should extend a mutation contract that is
  already semantically honest, not one that still loses target identity

## Architectural Notes

- target evidence and touched-aspect evidence are separate and both required
- causality and provenance are first-class runtime evidence, not "nice to have"
  metadata layers domains can reconstruct later
- batch/session authority evidence is the mutation analogue of batch-derived
  summaries on the read side
- existing-truth binding is a public contract because serious domains cannot
  honestly avoid it
- naming-aware and continuity-aware mutation evidence are capability families;
  they must be admitted or denied explicitly
- this gate should reduce downstream code by moving authority explanation into
  Query rather than teaching each domain to rebuild it

## Explicit Failure Taxonomy

- unsupported existing-truth binding family
- unresolved existing-truth target binding
- mismatched target-class binding
- unsupported symbolic same-batch target reference
- unsupported naming-writeback family
- ambiguous naming attachment outcome
- unsupported continuity-sensitive family
- denied continuity classification
- raw lower-runtime provenance passthrough attempt
- host override of resolved target evidence
- host reconstruction of causality/provenance after the runtime boundary
- scalarized batch/session evidence that drops component authority meaning

## Naive Traps Explicitly Rejected

- treating `collection + entity_identity` as sufficient long-term authority
  evidence for every mutation family
- packing causality or provenance into an untyped metadata map and calling the
  job done
- letting domains attach their own naming/continuity side packets beside Query
  receipts because "the runtime doesn't know enough yet"
- using the final component receipt as the meaning of a bulk import or ordered
  mutation session
- degrading unsupported existing-truth, naming, or continuity families into
  ordinary update semantics
- assuming the bridge can silently drop causality/provenance and that domains
  can rehydrate it later from lower-crate internals
- exposing proof-bearing evidence types with public constructors because it is
  "just docs/spec work for now"

## Explicit Non-Goals

- inventing new persistent naming semantics
- inventing new lineage or continuity semantics
- making Query the execution owner of writeback protocols
- implementing temporal or async mutation families
- claiming store-backed restart or durable continuation behavior
- adding domain-specific target classes to the generic runtime facade

## Self-Check

- Does this solve a real structural problem or just package work cosmetically?
  It solves the missing authority-evidence seam that write-heavy downstream
  domains are already exposing.
- Is the adversarial constraint precise and load-bearing? Yes. It names the
  exact information domains must not be forced to recover locally.
- Does the plan preserve crate authority boundaries? Yes. Query owns the
  facade contract; lower runtimes keep mutation truth, naming truth, and
  lineage truth.
- Does the plan define proof obligations, not just implementation tasks? Yes.
  Batch/session evidence, typed denial, support synchronization, and downstream
  code-deletion proof are all explicit.
- Could a competent engineer map this into honest types, modules, and tests?
  Yes. The phases map directly to mutation vocabularies, receipts, inspection,
  support rows, and certification suites.
- Does this belong in the roadmap sequence? Yes. It is the immediate runtime
  hardening gate that downstream write-authority work needs before temporal and
  async expansion.
