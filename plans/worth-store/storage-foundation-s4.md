# Storage Foundation S.4 Engineering Spec: WAL, Checkpoint, LSN, And Recovery Physics

> **Status:** Historical; superseded for current durability and recovery
> authority by
> [C.7 durable publication](physical-reconstruction-c7-durable-publication-join.md)
> and
> [C.8 fresh-process recovery](physical-reconstruction-c8-fresh-process-recovery-and-reopen.md).
> Retain this document only as design history; implementation and review must
> not use its wide recovery-physics facade, same-crate verifier placement,
> sharp-checkpoint assumptions, or pre-C.7 WAL ownership as current authority.
>
> **Roadmap parent:** [physical-database-roadmap.md](physical-database-roadmap.md)
>
> **Primary prerequisite:** `S.3 Physical Integrity, Scrub, Quarantine, And Corruption Localization`
>
> **Follow-on storage-foundation sequence:** `S.4.5 Physical Database Simulation Harness`
>
> **Primary architectural driver:** make acknowledged physical durability
> recover through WAL, pageLSN, checkpoint, flush ordering, and deterministic
> replay rules rather than backend residue, live heap state, or optimistic
> filesystem folklore.

## Goal

Make Worth Store recover acknowledged durable physical effects exactly once in
the recovered state through Store-owned physical recovery law.

S.4 turns WAL segments, LSNs, pageLSNs, checkpoint manifests, WAL-before-data
ordering, durable publication barriers, source precedence, replay cursors, and
bounded crash recovery into explicit database physics. It is complete when a
crash at any declared byte-publication seam recovers deterministically from
integrity-vetted persisted records, never trusts backend residue as authority,
never materializes closed physical effects twice, never loses acknowledged truth, and never
requires full-store scanning when the checkpoint interval and WAL tail bound
the recovery envelope.

## Why This Sequence Exists

S.1 made physical bytes addressable. S.2 made physical byte access bounded. S.3
made physical bytes integrity-vetted before logical decode. S.4 is the next
load-bearing foundation: once bytes are shaped, bounded, and vetted, the store
must decide which persisted physical records survive a crash and in what order
they may rebuild current physical state.

This is not the original Roadmap 1 WAL milestone with more fields. That earlier
program established semantic durable-mode crash recovery. S.4 is lower-level
database recovery physics for the Roadmap 2 physical substrate. Runtime
semantics remain above it. WAL is recovery machinery, not an alternate semantic
truth source.

## Governing Summaries

- `MENTALITY.md`
  protects adversarial-constraint-first design. S.4 starts with power loss,
  torn publication, reordered durability, and repeated restart, not with a
  friendly append log.
- `arch_laws.md`
  protects proof-bearing phase progression, source precedence, and one
  canonical artifact per authority boundary. S.4 must consume integrity-vetted
  inputs, produce recovery-admitted forms, and keep WAL mechanics subordinate
  to canonical authority.
- `composition_laws.md`
  protects named semantic steps. S.4 must not collapse WAL append, sync
  barriers, pageLSN comparison, checkpoint cutover, source precedence,
  idempotent redo, and evidence materialization into one recovery manager.
- `domain_structure_laws.md`
  protects responsibility topology. WAL, checkpoint, flush ordering, replay,
  source precedence, crash harness, offline verification, and evidence fail
  differently and need separate Store-owned homes.
- `perf_laws.md`
  protects visible cost. S.4 must expose exact WAL-tail, checkpoint-validation,
  replayed-frame, skipped-frame, page-redo, recovery-scan, and recovery-budget
  counters at the boundaries that claim bounded recovery.
- `physical-database-roadmap.md`
  places S.4 after physical integrity and before physical isolation. Recovery
  must consume S.3-vetted records and produce byte-stability assumptions that
  S.5 can later protect under maintenance interleavings.
- `runtime-integration-roadmap.md`
  keeps semantic durable truth above physical recovery mechanics. S.4 may
  recover physical pages and roots, but it does not redefine transaction,
  branch, snapshot, or canonical commit meaning.
- `storage-foundation-s2.md`
  gives S.4 recovery memory envelopes, dirty publication evidence, and bounded
  access constraints. S.4 consumes those envelopes rather than reopening memory
  residency law.
- `storage-foundation-s3.md`
  gives S.4 integrity-vetted WAL, checkpoint, page, manifest, and damage-map
  inputs. S.4 must not consume raw recovery bytes where S.3 can supply typed
  vetted records or recovery-blocking damage evidence.
- `worth_foundational_roadmap.md`
  protects shared boundary vocabulary. S.4 uses Foundational for exported
  reports, receipts, profiles, counters, and support evidence while Store keeps
  recovery physics authority.
- `worth_proof_roadmap.md`
  protects proof-bearing progression law. S.4 uses Proof patterns for
  integrity-vetted, recovery-admitted, replay-planned, replay-executed,
  checkpoint-validated, and closeout-ready states, but Proof does not own WAL
  semantics.
- `test-requirements.md`
  requires the `WAL/checkpoint/LSN recovery-physics test`, including crash
  points around WAL append, data-page flush, checkpoint write/cutover,
  compaction cutover, acknowledgment, and directory/rename durability.
- `test-requirements-2.md`
  requires the fault scheduler, storage-boundary interposer, crash harness,
  adversarial storage backend, offline verifier, recovery determinism harness,
  and mutation-style harness validation for S.4 closeout.

## Adversarial Constraint

S.4 must survive this hostile condition:

> A store receives foreground writes, dirty page publication, checkpoint
> attempts, compaction cutovers, directory or rename publication, flush delays,
> torn WAL frames, reordered persistence, lost flushes, repeated power-loss
> crashes, corrupted but integrity-classified records, and stale backend
> residue. After every crash point, recovery must derive one deterministic
> conclusion from integrity-vetted persisted records, WAL/checkpoint/pageLSN
> ordering law, and declared backend durability barriers, while bounded by
> checkpoint interval plus WAL tail and without reusing live heap state,
> trusting residue, losing acknowledged truth, or replaying closed work twice.

If recovery can read raw WAL bytes after S.3 produced a typed damage map, if an
acknowledgment can happen before its WAL durability barrier, if a page without a
sufficient pageLSN can outrank redo, if checkpoint cutover can publish without a
durable manifest/root basis, if recovery depends on directory listing residue
as authority, or if replay scans the full store rather than the checkpoint/WAL
tail envelope, S.4 is not closed.

## Product Decision Lock

- S.4 owns WAL, checkpoint, LSN, pageLSN, redo, source precedence, and crash
  recovery physics for Store physical bytes.
- S.4 consumes `S4RecoveryPhysicsIntegrityReadiness`; it must not accept raw
  WAL, page, manifest, or checkpoint bytes where S.3 handoff evidence exists.
- WAL is recovery machinery, not a second source of semantic truth.
- PageLSN is physical replay currency, not semantic commit authority.
- Checkpoints bound recovery work; they do not become unrebuildable authority.
- Acknowledgment is illegal until the declared WAL durability preconditions are
  met for the backend capability profile.
- Recovery conclusions are deterministic machine-checkable artifacts, not
  operator log interpretations.
- Foundational/Proof evidence describes exported boundary posture and
  progression. Store-owned recovery witness types remain the authority for
  replay, checkpoint, and recovered physical state.

## Foundational And Proof Adoption Contract

S.4 must use the Foundational and Proof features that already exist where they
fit the recovery boundary. This adoption is mandatory at Store boundary
evidence, certification, and proof-progression seams, and forbidden as a
replacement for Store-owned WAL, checkpoint, LSN, pageLSN, replay, or recovery
authority.

Foundational features S.4 must use:

- aspec-native boundary value, identity, and locator APIs wherever S.4 exports
  recovery evidence payloads, diagnostic subjects, mismatch locations,
  canonical basis entries, or S.5 handoff fields:
  `AspectValue`, `CanonicalString`, `CanonicalTimestamp`,
  `CanonicalTimestampTz`, `CanonicalBigInt`, `ContentRefId`, `EntityId`,
  `Generation`, `PartitionId`, `AspectKey`, `AspectMask`, `ProjectionMask`,
  `DiagnosticMask`, `CanonicalAspectStateMap`,
  `AuthoritativeRecordAspectState`, `AuthoritativeRecordAspectPatch`,
  `ContractValidatedAspectValue`, `AspectLocator`, `AspectValueLocator`,
  `BoundaryArtifactLocator`, `BoundarySourceLocator`,
  `BoundaryMismatchLocator`, `BoundaryArtifactId`, `BoundaryHandle`,
  `BoundaryEpoch`, `CanonicalDigestId`, and `EquivalenceBasisId`
- boundary artifact categories and roles for exported recovery surfaces:
  `FoundationalBoundaryArtifactCategory`,
  `FoundationalBoundaryArtifactRole`, `ReportCategory`, `ReceiptCategory`,
  `DerivedProjectionRole`, `SupportOnlyRole`, `PlannedWorkRole`, and
  `ReceiptEvidenceRole`
- boundary role claim APIs for exported recovery reports, support surfaces,
  planned recovery work, and completed recovery receipts:
  `claim_derived_projection_boundary_surface`,
  `claim_support_only_boundary_surface`,
  `claim_planned_work_boundary_surface`, and
  `claim_receipt_evidence_boundary_surface`
- boundary authority/current-basis APIs only for current-basis exported
  artifacts that have Store-owned recovery authority:
  `admit_current_basis_boundary_artifact`,
  `CurrentBasisBoundaryArtifact`, `BoundaryBridgedCurrentBasisBoundaryArtifact`,
  `readmit_current_basis_boundary_artifact_after_boundary`,
  and `FoundationalBoundaryCurrentBasisProofLane`
- boundary materialization APIs for recovery reports, receipts, and bundles:
  `plan_descriptive_boundary_materialization`,
  `plan_authoritative_boundary_materialization`,
  `materialize_descriptive_boundary_surface`,
  `materialize_authoritative_boundary_surface`,
  `plan_artifact_boundary_bundle`,
  `FoundationalBoundaryMaterializationPlan`,
  `FoundationalBoundaryMaterializationDecisionRow`,
  `FoundationalBoundaryMaterializationCost`, and
  `FoundationalBoundaryMaterializationBundle`
- canonical basis APIs for reproducible recovery evidence:
  `prepare_materialized_boundary_artifact_for_canonical_basis`,
  `prepare_materialized_boundary_bundle_for_canonical_basis`,
  `prepare_canonical_basis_sequence`,
  `prepare_canonical_basis_bundle`, `prepare_canonical_export_bundle`,
  `compare_canonical_basis`, `compare_canonical_exports`,
  `derive_canonical_digest`, `BoundaryBridgedCanonicalExportArtifact`, and
  `readmit_canonical_export_after_boundary`
- profile APIs for evidence richness and certification posture:
  `FoundationalProfileSet`, `DiagnosticRichnessProfile`,
  `SupportPostureProfile`, `RetentionDeliveryProfile`,
  `CertificationPostureProfile`, `plan_foundational_profile_materialization`,
  `plan_foundational_profile_materialization_with_elision`, and
  `attach_boundary_profiled_artifact`,
  `attach_support_profiled_artifact`, `attach_proof_bearing_profiled_artifact`,
  `certify_evidence_backed_proof_bearing_artifact`,
  `certify_production_certified_proof_bearing_artifact`,
  `readmit_evidence_backed_proof_bearing_artifact_after_boundary`, and
  `readmit_production_certified_proof_bearing_artifact_after_boundary`
- diagnostics APIs for recovery decision rows, failure rows, comparison rows,
  support rows, certified bundles, named gaps, absence causes, materialization
  plans, diagnostic subjects, and locators:
  `FoundationalDiagnosticOutcomeKind`, `FoundationalDiagnosticAbsenceCause`,
  `FoundationalDiagnosticDenialClass`, `FoundationalDiagnosticBreachClass`,
  `FoundationalDiagnosticEvidencePosture`,
  `FoundationalDiagnosticDeliveryClass`,
  `FoundationalDiagnosticSurfaceAvailability`,
  `FoundationalDiagnosticNamedGap`,
  `FoundationalDiagnosticCertifiedCoverageClass`,
  `FoundationalDiagnosticDecisionRow`, `FoundationalDiagnosticFailureRow`,
  `FoundationalDiagnosticComparisonRow`, `FoundationalDiagnosticSupportRow`,
  `FoundationalDiagnosticProvenanceReadyRow`,
  `FoundationalDiagnosticMaterializationPlan`,
  `FoundationalDiagnosticSupportReport`,
  `FoundationalDiagnosticExplanationBundle`,
  `FoundationalDiagnosticComparisonBundle`,
  `FoundationalCertifiedDiagnosticBundle`,
  `BoundaryBridgedCertifiedDiagnosticBundle`,
  `FoundationalDiagnosticSubject`, `FoundationalDiagnosticLocator`,
  `plan_diagnostic_support_report`,
  `materialize_diagnostic_support_report`,
  `plan_diagnostic_explanation_bundle`,
  `materialize_diagnostic_explanation_bundle`,
  `prepare_diagnostic_support_report_for_canonical_basis`,
  `prepare_diagnostic_explanation_bundle_for_canonical_basis`,
  `certify_current_basis_diagnostic_bundle`,
  `certify_diagnostic_bundle_with_source_basis`,
  `bridge_certified_diagnostic_bundle_trust_boundary`,
  `readmit_certified_diagnostic_bundle_after_boundary`, and
  `require_foundational_diagnostic_milestone6_production_test_readiness`
- boundary evidence APIs for provenance, lineage, reconstructed equivalence,
  receipt, support truth, freshness, locality, execution posture,
  runtime-assumption disclosure, and current-basis/readmitted evidence
  attachments:
  `FoundationalBoundaryEvidenceCategory`,
  `FoundationalBoundaryEvidenceAuthorityPath`,
  `FoundationalBoundaryEvidenceCanonicalDigestBasis`,
  `FoundationalBoundaryEvidenceLineageFrontDoor`,
  `FoundationalBoundaryEvidenceLineageOutcomeKind`,
  `FoundationalBoundaryEvidenceLineageSubject`,
  `FoundationalBoundaryEvidenceLocality`,
  `FoundationalBoundaryEvidenceProvenanceArtifact`,
  `FoundationalBoundaryEvidenceAttestedLineageArtifact`,
  `FoundationalBoundaryEvidencePartialLineageArtifact`,
  `FoundationalBoundaryEvidencePromotedLineageArtifact`,
  `FoundationalBoundaryEvidenceReplayDerivedLineageArtifact`,
  `FoundationalBoundaryEvidenceRestoredLineageArtifact`,
  `FoundationalBoundaryEvidenceReconstructedEquivalenceArtifact`,
  `FoundationalBoundaryEvidenceCompletedReceiptArtifact`,
  `FoundationalBoundaryEvidenceExecutedReceiptArtifact`,
  `FoundationalBoundaryEvidenceSupportRecoveryPosture`,
  `FoundationalBoundaryEvidenceSupportTruthKind`,
  `FoundationalBoundaryEvidenceFreshnessPosture`,
  `FoundationalBoundaryEvidenceRuntimeAssumption`,
  `FoundationalBoundaryEvidenceRuntimeNonAssumption`,
  `FoundationalBoundaryEvidenceSourceBasis`,
  `FoundationalBoundaryEvidenceStrategyBasis`,
  `FoundationalBoundaryEvidenceSupportAttachment`,
  `FoundationalBoundaryEvidenceSupportBasisDisclosure`,
  `CurrentBasisBoundaryEvidenceAttachmentBundle`, and
  `readmit_current_basis_boundary_evidence_attachment_bundle_after_boundary`
- performance APIs for counter-backed recovery-budget and replay-cost evidence:
  `FoundationalPerformanceLayoutIntent`,
  `FoundationalLayoutIntentClaim`,
  `FoundationalPerformanceAccessPatternDefinition`,
  `FoundationalPerformanceAccessPatternPosture`,
  `FoundationalPerformanceAllocationDefinition`,
  `FoundationalPerformanceAllocationPosture`,
  `FoundationalPerformanceBreadthLocalityDefinition`,
  `FoundationalPerformanceBreadthLocalityPosture`,
  `FoundationalPerformanceExecutionTemperature`,
  `FoundationalPerformanceExecutionTemperatureDefinition`,
  `FoundationalPerformanceFreshnessRetentionDefinition`,
  `FoundationalPerformanceFreshnessRetentionPosture`,
  `FoundationalPerformanceFallbackDebtDefinition`,
  `FoundationalPerformanceFallbackDebtPosture`,
  `FoundationalPerformanceBoundary`,
  `FoundationalPerformanceCounterName`,
  `FoundationalPerformanceCounterRow`,
  `FoundationalCounterBackedPerformanceReceipt`,
  `counter_backed_performance_receipt`,
  `attach_counter_backed_performance_receipt`,
  `FoundationalPolicyAdmissionPerformanceClaim`,
  `FoundationalPolicyAdmissionReceipt`,
  `attach_policy_admission_receipt`,
  `plan_performance_report`,
  `attach_performance_bundle`,
  `compare_performance_bundles`,
  `FoundationalPerformanceBudgetKind`,
  `FoundationalPerformanceEvidenceStrength`, and
  `FoundationalCertifiedPerformanceBundle`,
  `bridge_certified_performance_bundle_trust_boundary`, and
  `readmit_certified_performance_bundle_after_boundary`
- production-readiness APIs for the Foundational surface families S.4 consumes:
  `require_milestone1_production_test_readiness`,
  `require_foundational_boundary_artifact_milestone4_production_test_readiness`,
  `require_foundational_profile_milestone3_production_test_readiness`,
  `require_canonical_production_test_readiness`,
  `require_foundational_transition_milestone5_production_test_readiness`,
  `require_foundational_diagnostic_milestone6_production_test_readiness`,
  `require_foundational_boundary_evidence_milestone7_production_test_readiness`,
  and `require_foundational_performance_milestone8_production_test_readiness`

Proof features S.4 must use:

- proof-bearing recovery artifacts through `Artifact`, `ArtifactParts`,
  `ArtifactView`, `PhaseMarker`, `ProofMarker`, `Proof`, `ProofSet`, and
  `ProofSetCons`
- authority/capability witnesses through `AuthorityWitness`,
  `CapabilityWitness`, `AuthorityMarker`, and `CapabilityMarker` for recovery
  entry, durability-barrier, acknowledgment, replay, and closeout admission
- staged recovery planning through `Recipe<Unresolved>`, `Recipe<Resolved>`,
  `Recipe<Lowered>`, `Recipe<Admitted>`, `ExecutionReadyRecipe`, and
  `ExecutedRecipe`
- checked progression through `TransitionReadiness`, `PreConstructionGate`,
  `TransitionOutcome`, `SuccessfulTransitionOutcome`,
  `DenialTransitionOutcome`, `DeferredTransitionOutcome`,
  `FreshnessTransitionOutcome`,
  `resolve_checked_lower_and_admit_recipe`,
  `checked_admit_ready_and_execute_recipe`, and
  `checked_readmit_ready_and_execute_recipe`
- assumption and trust-boundary law through `AssumptionBasis`,
  `FreshnessScopedBasis`, `CurrentValidity`, `StaleReadable`,
  `RebindRequired`, `AuthorityRevalidationRequired`, `BoundaryBridged`, and
  the boundary-bridged stale/rebind/revalidation basis aliases
- structural proof collections through `CanonicalVec`, `UniqueVec`,
  `NonEmpty`, `Pair`, `DisjointPair`, `ExactlyOne`, `CanonicalOrder`,
  `Uniqueness`, and `Disjointness` where WAL frame ordering, source candidate
  uniqueness, non-empty redo tails, or disjoint recovery sources matter
- join/fork helpers only where S.4 has fixed-shape proof composition, such as
  joining checkpoint evidence with WAL-tail evidence:
  `JoinInputs2`, `ForkOutputs2`, `join_artifact_pair`,
  `compose_join_transition_outcome`, and `compose_join_success_transition`

Adoption denials:

- a Foundational report, receipt, profile, diagnostic row, performance receipt,
  boundary role claim, or boundary evidence attachment may never satisfy a
  Store API requiring `RecoveryRedoPlan`, `DurableAckReceipt`,
  `RecoveredPhysicalState`, `CheckpointCutoverReceipt`, or
  `S5PhysicalIsolationRecoveryReadiness`
- a Proof recipe, artifact, proof set, or witness may encode recovery
  progression but may not define what an LSN, pageLSN, checkpoint, WAL frame,
  source precedence edge, or redo operation means
- current-basis Foundational exports require Store-owned recovery authority
  plus Foundational current-basis admission/readmission; raw digests, raw basis
  rows, and copied receipt fields are not enough
- reduced-richness Foundational profiles may elide optional forensic material,
  but must not alter recovered physical state, recovery source decision,
  durability acknowledgment truth, replay counters, or S.5 handoff payload
- Foundational branch, merge, commit, and scoped-merge/cherry-pick surfaces are
  not recovery physics APIs. S.4 may use current-basis/readmission or receipt
  vocabulary where a completed recovery closeout crosses a trust boundary, but
  it must not model WAL replay as a branch merge, commit, merge scope, selected
  node, selected aspect, skipped-out-of-scope record, or cherry-pick admission.

## Recovery Physics Laws

- WAL Segment Authority Law: every replayable WAL record belongs to a declared
  segment, LSN interval, frame integrity scope, backend durability profile, and
  recovery generation. Records outside that basis are not replay candidates.
- LSN Monotonicity Law: admitted WAL frames advance LSN order monotonically
  inside their segment family. Gaps, duplicates, stale segments, and overlapping
  ranges are typed recovery conditions, not best-effort scans.
- Valid WAL Prefix Law: recovery admits only the maximal contiguous
  integrity-vetted WAL prefix from the selected replay basis. A torn or invalid
  tail suffix is a different condition from middle corruption, a stale segment,
  or a missing acknowledged range, and each must classify differently.
- Redo Record Grammar Law: every redo record must declare target physical page
  or extent generation, redo LSN, operation form, integrity binding,
  idempotence basis, and pageLSN comparison basis before it can enter a redo
  plan.
- WAL-Before-Data Law: acknowledged dirty data may not be published as durable
  unless the WAL records required to redo it crossed the declared durability
  barrier first.
- No-Undo Publication Law: S.4 is a redo-only recovery milestone. Durable
  data-page publication may include only physical mutations whose recovery
  outcome is already admitted as durable/recoverable under S.4 law. A page
  containing unadmitted physical changes is ineligible for durable publication
  unless protected by a rollback image or a later undo-capable milestone.
- PageLSN Replay Law: a page is up to date for a WAL record only when its
  admitted pageLSN is greater than or equal to that record's redo LSN under the
  declared comparison basis.
- Backend Profile Certification Law: every `DurableAckReceipt`, durability
  barrier verdict, recovery verdict, and crash-lane assertion is scoped to a
  named backend durability profile. A receipt certified under one profile may
  not satisfy another profile.
- Checkpoint Capture Mode Law: S.4 certifies checkpoint recovery under
  `SharpCheckpointCertificationMode` unless a later phase explicitly introduces
  `FuzzyCheckpointCertificationMode` with begin/end checkpoint records, redo
  frontier law, dirty-page table evidence, and interleaving constraints. S.5
  generalizes stable physical interleavings later.
- Checkpoint Cutover Law: a checkpoint is valid only if its manifest, root
  posture, covered LSN range, pageLSN frontier, and directory/rename durability
  evidence are all admitted together.
- Checkpoint Locator Law: directory listing may discover checkpoint
  candidates, but only a durable root selector, superblock ring, manifest
  pointer, or equivalent Store-owned locator can admit checkpoint freshness,
  generation, and source-precedence basis.
- WAL Retention Law: a WAL segment may not be deleted, recycled, or excluded
  from recovery until a checkpoint covering its LSN range has been durably
  admitted, its locator is recoverable, its manifest/root/pageLSN frontier have
  been admitted, and the next WAL tail begins contiguously from the checkpoint
  redo boundary.
- Source Precedence Law: recovery must choose among checkpoint, WAL tail,
  pages, manifests, compaction products, snapshots, and derived families by a
  typed precedence graph. Backend residue cannot outrank a valid recovery
  source.
- Source Role Law: checkpoint locators select base candidates, checkpoint
  manifests admit a base, WAL supplies redo records, pageLSN decides page-level
  skip/apply, compaction products are generation/cutover candidates, and
  backend residue only discovers candidates. These roles must not collapse into
  one generic source envelope.
- Compaction Visibility Law: S.4 does not own compaction strategy, but any
  compaction artifact visible to recovery must belong to a generation, carry an
  admitted cutover record, preserve old-generation recoverability until cutover
  durability is admitted, and be rejected as residue otherwise.
- Idempotent Redo Law: replaying the same admitted WAL tail multiple times over
  the same checkpoint basis must converge to the same recovered physical state.
- Acknowledgment Honesty Law: every acknowledged operation must either recover
  its physical effects exactly once in recovered state or fail the
  implementation. Redo records may be scanned, planned, skipped, or reapplied
  repeatedly according to pageLSN/idempotence rules. Every unacknowledged
  partial publication must be rejected, completed through typed replay, or
  classified as unrecoverable through evidence.
- Recovery Budget Law: ordinary recovery work must be bounded by checkpoint
  interval plus WAL tail, not total store size.
- Crash Isolation Law: recovery tests must discard live heap state, caches,
  buffer-pool frames, mmap views, singletons, and in-memory indexes before
  recovery begins.
- Offline Verifier Independence Law: the S.4 verifier may share stable physical
  format definitions, but it must not share the live recovery authority path in
  a way that makes identical replay mistakes invisible.

## Planned Directory Skeleton

`workspaces/worth-store/crates/worth-store-recovery-physics/src/`

- `lib.rs`
  exposes the S.4 facade and re-exports only proof-bearing recovery surfaces.
- `readiness.rs`
  consumes `S4RecoveryPhysicsIntegrityReadiness` and S.2 recovery envelopes.
- `lsn.rs`
  owns LSN, segment LSN interval, replay cursor, and monotonicity law.
- `wal_segment.rs`
  owns WAL segment identity, segment lifecycle, frame ordering, and segment
  closeout.
- `wal_frame.rs`
  owns replayable WAL-frame admission after S.3 integrity handoff.
- `wal_prefix.rs`
  owns valid-prefix admission, torn-tail classification, middle-corruption
  classification, stale-generation WAL rejection, and missing acknowledged
  range denial.
- `wal_retention.rs`
  owns segment truncation, recycling eligibility, checkpoint coverage, and
  contiguous-tail proof.
- `redo_record_grammar.rs`
  owns redo record target generation, operation form, integrity binding,
  idempotence basis, and pageLSN comparison basis.
- `durability_barrier.rs`
  owns WAL append, flush, fsync/fdatasync, directory sync, rename, and backend
  capability preconditions.
- `backend_durability_profile.rs`
  owns named S.4 durability profiles, profile-specific barrier requirements,
  and cross-profile receipt denial.
- `acknowledgment.rs`
  owns acknowledgment preconditions and illegal-acknowledgment denials.
- `page_lsn.rs`
  owns pageLSN comparison, stale-page classification, and redo eligibility.
- `dirty_publication.rs`
  consumes S.2 dirty publication evidence and checks WAL-before-data ordering.
- `no_undo_publication.rs`
  owns redo-only publication eligibility, unadmitted dirty-page denial,
  rollback-image posture, and undo-capable-milestone deferral.
- `checkpoint_manifest.rs`
  owns checkpoint manifest shape, covered ranges, root posture, and pageLSN
  frontier.
- `checkpoint_locator.rs`
  owns durable root selector, superblock ring, manifest pointer, generation
  selection, candidate discovery, and directory-residue denial.
- `checkpoint_capture_mode.rs`
  owns `SharpCheckpointCertificationMode`, fuzzy-checkpoint non-admission, and
  the exact capture assumptions S.4 certifies before S.5.
- `checkpoint_publication.rs`
  owns manifest write, manifest validation, durable cutover, and rollback
  posture.
- `source_precedence.rs`
  owns deterministic recovery source graph and residue rejection.
- `source_roles.rs`
  owns candidate discovery, source admission, and application-role separation
  for checkpoints, WAL tails, pageLSN/page images, compaction products, and
  backend residue.
- `compaction_visibility.rs`
  owns generation-bound visible compaction artifacts, cutover admission,
  old-generation recoverability, and residue rejection for S.4 crash seams.
- `redo_plan.rs`
  owns checkpoint-plus-tail planning and replay admission.
- `redo_execution.rs`
  owns idempotent page redo application over admitted pages.
- `partial_publication.rs`
  owns torn, unacknowledged, ambiguous, and incomplete publication outcomes.
- `recovery_budget.rs`
  owns checkpoint interval, WAL tail, scan breadth, replay breadth, and memory
  envelope counters.
- `recovered_state.rs`
  owns recovered physical root, recovered page frontier, and replay receipt.
- `offline_verifier.rs`
  owns independent read-only recovery verification.
- `aspec_boundary_payloads.rs`
  lowers recovery evidence, diagnostic subjects, locators, mismatch loci,
  counter payloads, and S.5 handoff fields into Foundational aspec-native
  values, identities, handles, epochs, digest ids, and boundary-safe locators.
- `foundational_evidence.rs`
  maps executed recovery findings into Foundational boundary artifacts,
  boundary evidence, diagnostics, profiles, canonical basis, and performance
  reports/receipts without moving Store recovery authority.
- `foundational_diagnostics.rs`
  maps recovery decisions, failures, mismatches, named gaps, absence causes,
  materialization availability, certified coverage, and readmitted diagnostic
  bundles onto Foundational diagnostics without changing recovery outcomes.
- `foundational_performance.rs`
  maps recovery-only, cold-path replay, verifier, materialization, support, and
  maintenance cost claims into Foundational performance boundaries,
  evidence-strength, layout intent, freshness/retention, fallback/debt,
  policy-admission, and counter-backed receipt surfaces.
- `proof_progression.rs`
  adapts Store recovery entry, planning, replay, readmission, and closeout
  states to shared Proof artifacts, recipes, witnesses, outcomes, assumptions,
  and proof collections.
- `foundational_adoption.rs`
  owns S.4's concrete Foundational API adoption inventory, readiness
  preconditions, and projection-authority denials.
- `proof_adoption.rs`
  owns S.4's concrete Proof API adoption inventory, phase mapping, and
  proof-owned-versus-Store-owned boundary denials.
- `counters.rs`
  owns exact recovery physics counters.
- `diagnostics.rs`
  owns typed recovery denials and recovery classification reports.

Minimum witness field contracts:

- `DurableAckReceipt`
  carries Store id, recovery generation, backend durability profile, WAL
  segment id, LSN range, frame digest, required barrier set, completed barrier
  evidence, directory/rename durability posture where applicable, and
  non-authoritative timestamp metadata.
- `CheckpointManifest`
  carries checkpoint id, checkpoint generation, capture mode, base root
  posture, covered LSN range, redo boundary LSN, pageLSN frontier, manifest
  digest, S.3 integrity basis, durable locator id, and publication generation.
- `RecoverySourceDecisionTrace`
  carries discovered candidates, admitted source roles, rejected residue,
  selected checkpoint locator, selected checkpoint manifest, selected WAL tail,
  compaction visibility decision, source-precedence graph digest, and denial
  reasons.
- `RecoveryRedoPlan`
  carries replay basis, valid WAL prefix, admitted redo records, target
  page/generation set, pageLSN comparison basis, idempotence basis, skipped
  frame posture, expected counters, and recovery budget envelope.
- `RecoveredPhysicalState`
  carries recovered root, recovered generation, pageLSN frontier, applied
  physical-effect digest, skipped-redo digest, source decision digest, recovery
  counter snapshot, and S.5 stability assumptions.
- `RecoveryBudget`
  carries checkpoint interval, WAL tail frame count, segment scan count, page
  redo count, skipped frame count, verifier read count, resident-byte ceiling,
  allocation envelope, and exact zero counters for forbidden full-store scans.
- `OfflineRecoveryVerificationReport`
  carries persisted artifact digest, selected locator, selected checkpoint,
  WAL prefix classification, source decision digest, recovered-state digest,
  verifier/runtime comparison, disagreement posture, and forbidden live-runtime
  dependency counters.
- `S5PhysicalIsolationRecoveryReadiness`
  carries recovered root, pageLSN frontier, replay receipt, source-precedence
  trace, WAL retention posture, checkpoint capture mode, backend profile, and
  explicit physical-stability assumptions for S.5.

`workspaces/worth-store/crates/worth-store-certification/src/`

- `recovery_physics_scenario_definitions.rs`
  defines S.4 crash/recovery scenario grammar over the Roadmap 2 harness.
- `recovery_physics_scenario_plans.rs`
  lowers scenarios into capabilities, crash seams, drivers, observers, oracles,
  counters, and transcript identity before execution.
- `recovery_physics_fault_drivers.rs`
  owns WAL append, flush, page flush, checkpoint, cutover, ack, and rename
  fault delivery through the storage-boundary interposer.
- `recovery_physics_observers.rs`
  observes WAL durability, pageLSN, checkpoint, replay, source-precedence,
  residue-rejection, budget, and offline-verifier facts.
- `recovery_physics_oracles.rs`
  judges acknowledged-write recovery, unacknowledged publication handling,
  idempotent redo, bounded recovery, and deterministic restart.
- `recovery_physics_mutation_validation.rs`
  declares controlled defective backends and weakened recovery variants that
  the S.4 suites must fail in the intended lane.
- `recovery_physics_transcripts.rs`
  emits replay-comparable crash, recovery, counter, source, verifier, and
  shortcut-rejection transcripts.
- `recovery_foundational_adoption_matrix.rs`
  maps each Foundational milestone surface consumed by S.4 to producer-diversity
  lanes, compile-fail substitutions, canonical/golden evidence, profile
  variants, readmission proofs, and explicit non-applicable surfaces.

`workspaces/worth-store/crates/worth-store-test-support/src/`

- `recovery_fault_profiles.rs`
  declares deterministic crash/fault profiles without owning proof meaning.
- `wal_checkpoint_fixtures.rs`
  creates WAL/checkpoint/pageLSN fixture stores through production boundaries.
- `crash_isolation.rs`
  supplies process-death or fresh-runtime restart mechanics.
- `adversarial_recovery_backend.rs`
  wraps the production storage boundary for reordered, delayed, torn, lost, and
  stale persistence behavior.
- `offline_verifier_fixtures.rs`
  supplies persisted-byte inputs to verifier lanes without live runtime state.

## Roadmap 2 Harness Test Plan

S.4 must extend the inherited Roadmap 2 physical scenario harness. It must not
create a separate recovery harness, put oracle meaning in test support, or prove
recovery by calling a restart method on live objects from the crashed instance.

The inherited proof pipeline remains:

`PhysicalScenarioDefinition` -> `PhysicalScenarioPlan` ->
`PhysicalScenarioExecution` -> `ObservedPhysicalTrace` ->
`PhysicalProofOracleVerdict` -> `PhysicalStoryTranscript`

S.4 scenario plans must declare before execution:

- the recovery law being proved or attacked
- required S.1 physical-format capabilities
- required S.2 recovery memory envelope and dirty publication evidence
- required S.3 integrity handoff records and recovery-blocking damage maps
- backend durability capability profile
- WAL segment and LSN topology
- pageLSN and dirty publication basis
- checkpoint interval and checkpoint manifest basis
- exact crash or fault seam
- expected acknowledgment posture
- expected recovery source precedence
- expected redo plan and replay cursor bounds
- expected offline verifier comparison
- expected forbidden shortcuts
- exact counter expectations
- artifact policy and transcript identity basis

Required definition vocabulary includes:

- `given_s4_integrity_readiness`
- `given_recovery_memory_envelope`
- `given_backend_durability_profile`
- `given_wal_segment_with_lsn_range`
- `given_checkpoint_manifest_covering_lsn`
- `when_crash_before_wal_append_durability`
- `when_crash_after_wal_durability_before_ack`
- `when_crash_after_ack_before_page_flush`
- `when_page_flush_reaches_media_before_checkpoint`
- `when_checkpoint_manifest_is_torn`
- `when_checkpoint_cutover_loses_directory_entry`
- `when_compaction_cutover_interleaves_with_checkpoint`
- `when_backend_returns_stale_residue`
- `then_recover_acknowledged_write_once`
- `then_reject_unacknowledged_partial_publication`
- `then_choose_checkpoint_plus_wal_tail`
- `then_redo_is_idempotent`
- `then_recovery_budget_matches_checkpoint_tail`
- `then_offline_verifier_agrees_or_reports_disagreement`

Required lane families:

- `recovery_entry_lane`
  proves S.4 consumes S.3 integrity readiness and S.2 recovery envelopes.
- `wal_lsn_topology_lane`
  proves WAL segment, LSN interval, gap, duplicate, and stale-segment law.
- `durability_barrier_lane`
  proves WAL append, flush, directory sync, rename, and ack preconditions.
- `page_lsn_ordering_lane`
  proves pageLSN redo eligibility and stale page rejection.
- `checkpoint_publication_lane`
  proves checkpoint manifest write, validation, cutover, and rollback posture.
- `source_precedence_lane`
  proves checkpoint/WAL/page/manifest/compaction/residue precedence.
- `idempotent_redo_lane`
  proves repeated replay converges and skipped-frame counters are exact.
- `partial_publication_lane`
  proves unacknowledged, torn, incomplete, and ambiguous publications are typed.
- `bounded_recovery_lane`
  proves recovery work is bounded by checkpoint interval and WAL tail.
- `crash_matrix_lane`
  proves declared crash seams with the fault scheduler and crash harness.
- `offline_verifier_lane`
  proves independent persisted-byte verification.
- `recovery_determinism_lane`
  proves identical bytes recover to identical classifications.
- `foundational_proof_evidence_lane`
  proves executed Store recovery evidence materializes into shared vocabulary
  without replacing Store-owned recovery witnesses.
- `synthetic_recovery_shortcut_rejection_lane`
  proves live-state reuse, backend residue guessing, logs, and same-run
  self-comparison cannot close S.4.

Required drivers:

- inherited `PlatformBackendDriver`, `PersistedFileDeviceDriver`,
  `AdversarialByteDeviceDriver`, `CrashInterposerDriver`,
  `MemoryPressureDriver`, `AllocationSentinelDriver`, and
  `BackgroundMaintenanceDriver`
- S.4 `FaultSchedulerDriver`, `StorageBoundaryInterposerDriver`,
  `WalAppendFaultDriver`, `FlushDurabilityFaultDriver`,
  `CheckpointPublicationFaultDriver`, `RenameDirectoryFaultDriver`,
  `AckBoundaryCrashDriver`, `StaleResidueDriver`, and
  `FreshRuntimeRecoveryDriver`

Required observers:

- `WalDurabilityObserver`
- `LsnTopologyObserver`
- `PageLsnObserver`
- `CheckpointManifestObserver`
- `RecoverySourcePrecedenceObserver`
- `RedoReplayObserver`
- `RecoveryBudgetObserver`
- `CrashIsolationObserver`
- `OfflineVerifierObserver`
- `ForbiddenRecoveryShortcutObserver`

Required proof oracles:

- `AcknowledgedWritesRecoverExactlyOnceOracle`
- `UnacknowledgedPartialPublicationRejectedOrCompletedOracle`
- `WalBeforeDataOrderingOracle`
- `PageLsnRedoEligibilityOracle`
- `CheckpointCutoverValidityOracle`
- `BackendResidueNeverOutranksRecoverySourceOracle`
- `IdempotentRedoConvergenceOracle`
- `RecoveryBoundedByCheckpointTailOracle`
- `FreshRuntimeCrashIsolationOracle`
- `OfflineVerifierIndependenceOracle`
- `RecoveryDeterminismOracle`
- `TestSupportCannotOwnRecoveryMeaningOracle`

Required machine-checkable outputs:

- `recovery_physics_story_transcript`
- `recovery_physics_scenario_plan`
- `wal_lsn_topology_trace`
- `wal_valid_prefix_trace`
- `wal_prefix_classification_report`
- `durability_barrier_trace`
- `backend_profile_certification_trace`
- `acknowledgment_boundary_trace`
- `page_lsn_redo_trace`
- `no_undo_publication_trace`
- `checkpoint_publication_trace`
- `checkpoint_locator_trace`
- `checkpoint_capture_mode_trace`
- `wal_retention_trace`
- `source_precedence_trace`
- `source_role_trace`
- `compaction_visibility_trace`
- `redo_replay_trace`
- `redo_record_grammar_trace`
- `partial_publication_trace`
- `crash_isolation_trace`
- `offline_verifier_report`
- `recovery_determinism_report`
- `recovery_budget_counter_trace`
- `backend_residue_rejection_report`
- `foundational_recovery_receipt_trace`
- `proof_progression_recovery_trace`
- `foundational_recovery_adoption_trace`
- `proof_recovery_adoption_trace`
- `foundational_projection_authority_denial_trace`
- `aspec_boundary_payload_trace`
- `foundational_diagnostic_certified_bundle_trace`
- `foundational_performance_claim_trace`
- `foundational_non_applicable_surface_denial_trace`
- `synthetic_recovery_shortcut_rejection_report`

## S.4 Certification Matrix

S.4 must include an explicit certification matrix like S.2 and S.3. The matrix
is not a reporting nicety; it is the plan-accountability proof that every
recovery claim is attacked by a real fault lane and judged by an oracle that
does not live in test support.

Every matrix row must name:

- suite id
- lane family
- workload profile
- backend durability profile
- fixture or generated trace
- fault seam and fault operator
- control lane
- hostile lane
- reopen lane
- offline verifier lane
- semantic parity lane where semantic authority is observable
- forbidden-shortcut lane
- required drivers
- required observers
- required proof oracles
- exact counter expectations
- evidence outputs
- expected mutant failures

Minimum matrix rows:

| Recovery pressure | Required suite | Lane family | Required profile or fixture | Required evidence |
| --- | --- | --- | --- | --- |
| Entry and handoff | `recovery_entry_authority_suite` | `recovery_entry_lane` | `s4_integrity_handoff_fixture`, `synthetic_recovery_shortcut_fixture` | `recovery_physics_story_transcript`, `recovery_entry_denial_trace`, `physical_proof_oracle_verdict` |
| WAL ordering | `wal_lsn_topology_suite` | `wal_lsn_topology_lane` | `wal_gap_fixture`, `wal_duplicate_lsn_fixture`, `wal_stale_segment_fixture` | `wal_lsn_topology_trace`, `scenario_denial_trace`, `recovery_physics_scenario_plan` |
| Valid WAL prefix | `valid_wal_prefix_classification_suite` | `wal_lsn_topology_lane` | `torn_tail_fixture`, `middle_corruption_fixture`, `missing_acknowledged_range_fixture`, `stale_generation_fixture` | `wal_valid_prefix_trace`, `wal_prefix_classification_report`, `failure_digest` |
| Durability and ack | `durability_barrier_and_ack_suite` | `durability_barrier_lane` | `lost_flush_fixture`, `short_write_fixture`, `directory_sync_failure_fixture`, `unsupported_backend_durability_fixture` | `durability_barrier_trace`, `acknowledgment_boundary_trace`, `hardware_assumption_report` |
| Backend profile certification | `backend_profile_certification_suite` | `durability_barrier_lane` | strict simulated, POSIX fsync+dir fsync, Windows flush, mmap-not-certified, lost-flush, and reordered-flush profiles | `backend_profile_certification_trace`, `durable_ack_receipt_trace`, `compile_fail_boundary_report` |
| WAL-before-data | `wal_before_data_page_lsn_suite` | `page_lsn_ordering_lane` | `stale_page_lsn_fixture`, `page_flush_before_wal_fixture`, `missing_page_lsn_fixture` | `page_lsn_redo_trace`, `redo_replay_trace`, `counter_snapshot` |
| No-undo publication | `no_undo_publication_suite` | `page_lsn_ordering_lane` | `unadmitted_dirty_page_flush_fixture`, `rollback_image_fixture`, `redo_only_publication_fixture` | `no_undo_publication_trace`, `partial_publication_trace`, `failure_digest` |
| Checkpoint publication | `checkpoint_manifest_publication_suite` | `checkpoint_publication_lane` | `torn_checkpoint_manifest_fixture`, `lost_checkpoint_cutover_fixture`, `stale_checkpoint_frontier_fixture` | `checkpoint_publication_trace`, `source_precedence_trace`, `failure_digest` |
| Checkpoint locator and capture mode | `checkpoint_locator_capture_mode_suite` | `checkpoint_publication_lane` | `directory_candidate_fixture`, `root_selector_fixture`, `superblock_ring_fixture`, `fuzzy_checkpoint_without_frontier_fixture` | `checkpoint_locator_trace`, `checkpoint_capture_mode_trace`, `failure_digest` |
| WAL retention | `wal_retention_truncation_suite` | `checkpoint_publication_lane` | `covered_segment_fixture`, `uncovered_segment_fixture`, `non_contiguous_tail_fixture`, `lost_locator_fixture` | `wal_retention_trace`, `checkpoint_publication_trace`, `failure_digest` |
| Source precedence | `recovery_source_precedence_suite` | `source_precedence_lane` | `backend_residue_fixture`, `orphaned_checkpoint_fixture`, `invalid_compaction_product_fixture` | `source_precedence_trace`, `backend_residue_rejection_report`, `recovery_physics_story_transcript` |
| Source roles and compaction visibility | `source_role_compaction_visibility_suite` | `source_precedence_lane` | `valid_checkpoint_plus_tail_fixture`, `stale_page_fixture`, `generation_bound_compaction_fixture`, `uncutover_compaction_residue_fixture` | `source_role_trace`, `compaction_visibility_trace`, `backend_residue_rejection_report` |
| Redo idempotence | `idempotent_redo_replay_suite` | `idempotent_redo_lane` | `checkpoint_plus_tail_fixture`, `duplicate_replay_fixture`, `already_redone_page_fixture` | `redo_replay_trace`, `recovery_determinism_report`, `counter_snapshot` |
| Redo record grammar | `redo_record_grammar_suite` | `idempotent_redo_lane` | missing target generation, missing operation form, missing integrity binding, missing idempotence basis, and wrong pageLSN basis fixtures | `redo_record_grammar_trace`, `redo_replay_trace`, `compile_fail_boundary_report` |
| Partial publication | `partial_publication_classification_suite` | `partial_publication_lane` | `crash_before_wal_fixture`, `crash_after_wal_before_ack_fixture`, `crash_after_ack_before_page_flush_fixture` | `partial_publication_trace`, `acknowledgment_boundary_trace`, `failure_digest` |
| Bounded recovery | `bounded_recovery_budget_suite` | `bounded_recovery_lane` | `checkpoint_heavy_profile`, `long_wal_tail_profile`, `store_larger_than_memory` | `recovery_budget_counter_trace`, `resource_envelope_report`, `counter_snapshot` |
| Crash matrix | `crash_matrix_fault_scheduler_suite` | `crash_matrix_lane` | generated `checkpoint_heavy` and `compaction_heavy` traces | `fault_delivery_log`, `crash_isolation_trace`, `recovery_physics_story_transcript` |
| Fresh restart | `fresh_runtime_crash_isolation_suite` | `crash_matrix_lane` | `fresh_runtime_reopen_fixture`, `live_state_reuse_mutant_fixture` | `crash_isolation_trace`, `synthetic_recovery_shortcut_rejection_report`, `physical_proof_oracle_verdict` |
| Offline verification | `offline_verifier_independence_suite` | `offline_verifier_lane` | `persisted_bytes_verifier_fixture`, `runtime_verifier_disagreement_fixture` | `offline_verifier_report`, `runtime_recovery_comparison_report`, `disagreement_report` |
| Determinism | `recovery_determinism_suite` | `recovery_determinism_lane` | repeated identical persisted-byte digests under the same profile | `recovery_determinism_report`, `persisted_artifact_digest`, `verifier_digest` |
| Evidence export | `foundational_recovery_evidence_suite` | `foundational_proof_evidence_lane` | executed recovery traces plus reduced-richness profile variants | `foundational_recovery_receipt_trace`, `proof_progression_recovery_trace`, `profile_elision_report` |
| Aspec boundary payloads | `foundational_aspec_boundary_payload_suite` | `foundational_proof_evidence_lane` | recovery evidence payloads, diagnostic locators, mismatch loci, counter payloads, and S.5 handoff payload fixtures | `aspec_boundary_payload_trace`, `canonical_basis_trace`, `compile_fail_boundary_report` |
| Foundational diagnostics | `foundational_diagnostic_recovery_bundle_suite` | `foundational_proof_evidence_lane` | decision, failure, mismatch, named-gap, missing-evidence, partial-coverage, and certified/readmitted diagnostic fixtures | `foundational_diagnostic_certified_bundle_trace`, `canonical_basis_trace`, `compile_fail_boundary_report` |
| Foundational performance | `foundational_recovery_performance_claim_suite` | `foundational_proof_evidence_lane` | recovery-only, cold replay, verifier, materialization, support-only, policy-admission, and counter-backed fixtures | `foundational_performance_claim_trace`, `recovery_budget_counter_trace`, `compile_fail_boundary_report` |
| Foundational adoption | `foundational_recovery_adoption_suite` | `foundational_proof_evidence_lane` | executed recovery, planned recovery, support-only recovery, and current-basis export fixtures | `foundational_recovery_adoption_trace`, `foundational_projection_authority_denial_trace`, `canonical_basis_trace` |
| Foundational non-applicable surfaces | `foundational_non_applicable_surface_denial_suite` | `foundational_proof_evidence_lane` | branch, merge, commit, scoped-merge, selected-node, selected-aspect, skipped-scope, and cherry-pick fixtures | `foundational_non_applicable_surface_denial_trace`, `compile_fail_boundary_report`, `test_support_authority_denial_report` |
| Proof adoption | `proof_recovery_adoption_suite` | `foundational_proof_evidence_lane` | recovery entry, redo planning, replay execution, stale basis, and readmission fixtures | `proof_recovery_adoption_trace`, `proof_progression_recovery_trace`, `compile_fail_boundary_report` |
| Synthetic rejection | `synthetic_recovery_test_rejection_suite` | every S.4 lane family | `synthetic_recovery_shortcut_fixture`, `log_only_fixture`, `same_run_self_comparison_fixture` | `synthetic_recovery_shortcut_rejection_report`, `test_support_authority_denial_report`, `physical_proof_oracle_verdict` |
| S.5 handoff | `s5_recovery_readiness_handoff_suite` | `bounded_recovery_lane` plus `source_precedence_lane` | recovered root and pageLSN frontier fixtures | `s5_recovery_readiness_trace`, `source_precedence_trace`, `replay_receipt_trace` |

Every required suite must have at least:

- one clean control lane
- one hostile fault lane
- one reopen-from-persisted-bytes lane
- one forbidden-shortcut lane
- exact positive counters and exact zero counters for forbidden work
- a transcript that can be replayed without the original runtime heap
- an evidence bundle sufficient for offline pass/fail judgment

Rows that exercise acknowledged durability must include both semantic parity
and physical recovery evidence. Rows that exercise unacknowledged or damaged
physical publication must include typed failure localization and must not claim
semantic truth recovery unless the canonical authority layer actually admits
that conclusion.

## Mutation-Style Harness Validation

S.4 must prove the harness catches known recovery defects. Controlled mutants
may be implemented as mutant backends, mutant harness profiles, or feature-gated
defective runtime variants, but each mutant must run through the same scenario
plan, driver, observer, oracle, and transcript machinery as ordinary suites.

Required mutant classes:

- `page_lsn_ignored_mutant`
- `wal_before_data_barrier_removed_mutant`
- `ack_before_wal_flush_mutant`
- `directory_sync_assumed_success_mutant`
- `checkpoint_cutover_without_manifest_validation_mutant`
- `backend_residue_preferred_over_wal_tail_mutant`
- `duplicate_lsn_accepted_mutant`
- `wal_gap_silently_skipped_mutant`
- `torn_tail_treated_as_middle_corruption_mutant`
- `middle_corruption_treated_as_torn_tail_mutant`
- `missing_acknowledged_wal_range_ignored_mutant`
- `redo_record_without_idempotence_basis_mutant`
- `redo_executor_redecides_source_precedence_mutant`
- `replay_not_idempotent_mutant`
- `unacknowledged_residue_promoted_mutant`
- `unadmitted_dirty_page_flush_allowed_mutant`
- `no_undo_policy_omitted_mutant`
- `durable_ack_receipt_profile_erased_mutant`
- `backend_profile_cross_certified_mutant`
- `directory_listing_selects_checkpoint_mutant`
- `fuzzy_checkpoint_admitted_without_frontier_mutant`
- `wal_segment_recycled_before_checkpoint_admission_mutant`
- `non_contiguous_wal_tail_after_checkpoint_accepted_mutant`
- `source_role_generic_envelope_mutant`
- `uncutover_compaction_artifact_admitted_mutant`
- `old_generation_reclaim_before_compaction_cutover_mutant`
- `live_state_reused_after_crash_mutant`
- `offline_verifier_shares_live_recovery_path_mutant`
- `recovery_budget_full_store_scan_mutant`
- `foundational_receipt_accepted_as_recovery_witness_mutant`
- `aspec_payload_uses_json_or_debug_string_mutant`
- `diagnostic_missing_evidence_collapses_into_denial_mutant`
- `diagnostic_partial_coverage_marked_certified_mutant`
- `support_truth_accepted_as_authority_truth_mutant`
- `replay_derived_lineage_marked_direct_restored_mutant`
- `reconstructed_equivalence_marked_direct_continuity_mutant`
- `performance_policy_receipt_accepted_as_counter_truth_mutant`
- `support_replay_performance_marked_current_basis_mutant`
- `foundational_merge_scope_used_as_recovery_source_mutant`
- `scoped_merge_cherry_pick_used_for_wal_replay_mutant`

Each mutant row must declare:

- expected failing suite
- expected failing lane
- expected oracle that catches the mutant
- expected counter or transcript difference
- whether the failure is compile-fail, admission denial, runtime oracle
  failure, or evidence-bundle mismatch

Mutation validation is not optional polish. S.4 cannot close until CI
Certification mode fails every required mutant in the intended lane.

## Test Topology Requirements

S.4 test code must obey the same composition laws as production code.

Required test topology:

`workspaces/worth-store/crates/worth-store-certification/src/`

- `recovery_physics_scenario_definitions.rs`
- `recovery_physics_scenario_plans.rs`
- `recovery_physics_fault_drivers.rs`
- `recovery_physics_observers.rs`
- `recovery_physics_oracles.rs`
- `recovery_physics_mutation_validation.rs`
- `recovery_physics_transcripts.rs`
- `recovery_physics_closeout.rs`

`workspaces/worth-store/crates/worth-store-test-support/src/`

- `recovery_fault_profiles.rs`
- `wal_checkpoint_fixtures.rs`
- `crash_isolation.rs`
- `adversarial_recovery_backend.rs`
- `offline_verifier_fixtures.rs`
- `recovery_mutant_backends.rs`

Forbidden topology:

- no `s4_tests.rs` file that owns several unrelated responsibilities
- no test-support oracle verdicts
- no fixture labels that imply pass/fail meaning
- no direct private-struct mutation to simulate storage failure
- no helper that hides crash/reopen, backend fault, or verifier independence
- no file over the workspace line cap unless explicitly exempted in the
  implementation plan

The next correct edit must be obvious from the file path: crash seam tests go
to crash/fault lanes, pageLSN tests go to pageLSN lanes, verifier independence
tests go to verifier lanes, and shortcut rejection tests go to shortcut
rejection lanes.

## Phases

### Phase 1: Consume Integrity Handoff And Recovery Envelopes

Phase 1 closes the S.3-to-S.4 entry boundary. S.4 may consume only integrity-
vetted recovery records, typed recovery-blocking damage, S.2 recovery memory
envelopes, and S.1 physical authority recaps.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-physical-integrity`
- `worth-store-buffer-pool`
- `worth-store-readiness`
- `worth-store-certification`
- `worth-proof`

**Relevant APIs**

- `S4RecoveryPhysicsIntegrityReadiness`
- `S4IntegrityHandoffPayload`
- `RecoveryMemoryEnvelope`
- `IntegrityVettedWalFrame`
- `IntegrityVettedCheckpointRecord`
- `IntegrityVettedRootManifestRecord`
- `IntegrityVettedPageFrameRecord`
- `IntegrityDamageMap`
- `RecoveryEntryAdmission`
- `RecoveryBlockedByIntegrityDamage`

**Warnings**

- Do not accept raw WAL/page/checkpoint bytes when S.3 can supply vetted
  records or typed damage maps.
- Do not reinterpret S.3 quarantine as recoverability.
- Do not let recovery planning reopen S.2 memory-budget or S.3 integrity law.

**Test requirements**

- Adversarial equivalence: independent S.3 handoff runs over the same intact
  physical records produce the same S.4 entry identity and recovery basis.
- Adversarial denial: raw bytes, copied integrity reports, expired protected
  views, and unbounded recovery envelopes cannot enter S.4 recovery admission.
- Boundary proof: recovery-blocking S.3 damage prevents replay planning before
  any WAL source precedence is chosen.

**Engineering decisions**

- Recovery starts from typed physical evidence, not from files.
- S.4 may classify integrity damage as recovery-blocking, but not repair it.
- Proof progression is used for recovery entry state, not for media semantics.

**Open questions**

- None.

### Phase 2: Define WAL Segment, LSN, And Replay Cursor Topology

Phase 2 defines the physical ordering vocabulary recovery will consume before
any append, ack, checkpoint, or redo path exists.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-physical-format`
- `worth-store-certification`

**Relevant APIs**

- `WalSegmentId`
- `WalSegmentGeneration`
- `LogSequenceNumber`
- `WalLsnRange`
- `ReplayCursor`
- `WalFrameOrderingProof`
- `WalTopologyDenial`

**Warnings**

- Do not derive LSN order from file listing order or backend residue.
- Do not let semantic commit ids substitute for physical LSNs.
- Do not permit gap, duplicate, or overlapping LSN ranges to become ordinary
  replay candidates.

**Test requirements**

- Adversarial equivalence: the same WAL topology discovered through independent
  segment scans produces the same ordered replay cursor.
- Adversarial denial: gaps, duplicates, stale segments, overlapping ranges, and
  wrong-generation segments deny or enter explicit recovery classification.
- Ordering proof: insertion order, directory listing order, and map iteration
  cannot affect replay cursor order.

**Engineering decisions**

- LSN is the physical recovery order currency.
- WAL segment lifecycle is distinct from semantic commit lifecycle.
- Replay cursor construction precedes replay execution.

**Open questions**

- None.

### Phase 3: Enforce WAL Durability Barriers And Acknowledgment Preconditions

Phase 3 makes acknowledgment illegal until WAL append and durability barriers
have completed under the declared backend profile.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-physical-backend`
- `worth-store-certification`

**Relevant APIs**

- `WalAppendPlan`
- `WalAppendReceipt`
- `WalDurabilityBarrier`
- `BackendDurabilityProfile`
- `SimulatedStrictDurableProfile`
- `PosixFileFsyncDirFsyncProfile`
- `WindowsFlushFileBuffersProfile`
- `MmapFlushNotDurabilityCertifiedProfile`
- `AdversarialLostFlushProfile`
- `AdversarialReorderedFlushProfile`
- `AcknowledgmentPrecondition`
- `DurableAckReceipt`
- `IllegalAcknowledgmentDenial`

**Warnings**

- Do not acknowledge on buffered write success where the backend profile
  requires flush or directory sync.
- Do not let one backend capability profile certify all profiles.
- Do not hide durability barriers behind a generic append helper.
- Do not leave the backend durability matrix as an implementation detail; every
  ack receipt and crash verdict must name the profile it was certified under.

**Test requirements**

- Adversarial equivalence: equivalent durable append paths under the same
  backend profile produce the same acknowledgment eligibility.
- Adversarial denial: lost flush, short write, delayed flush, directory sync
  failure, and unsupported durability capability block acknowledgment.
- Profile proof: `SimulatedStrictDurableProfile`,
  `PosixFileFsyncDirFsyncProfile`, `WindowsFlushFileBuffersProfile`,
  `MmapFlushNotDurabilityCertifiedProfile`, `AdversarialLostFlushProfile`, and
  `AdversarialReorderedFlushProfile` either certify their exact required
  barriers or deny explicitly; receipts cannot cross profile boundaries.
- Crash proof: crash after WAL durability but before acknowledgment recovers
  through typed unacknowledged-completed or replayable posture without losing
  persisted data.

**Engineering decisions**

- Acknowledgment is a derived permission from durable WAL evidence.
- Backend profiles scope durability claims.
- S.6 will later expand I/O QoS, but S.4 owns correctness barriers required for
  recovery.

**Open questions**

- None. S.4 must ship named backend durability profiles and may mark hardware
  qualification posture as later S.6/S.12 evidence, not as an unnamed S.4
  profile.

### Phase 4: Define PageLSN And WAL-Before-Data Publication Law

Phase 4 connects dirty page publication from S.2 to WAL durability through
pageLSN ordering.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-buffer-pool`
- `worth-store-physical-format`
- `worth-store-certification`

**Relevant APIs**

- `PageLsn`
- `PageRedoEligibility`
- `DirtyPublicationEvidence`
- `WalBeforeDataOrderingProof`
- `NoUndoPublicationEligibility`
- `UnadmittedDirtyPagePublicationDenial`
- `RollbackImagePublicationPosture`
- `StalePageRecoveryClassification`
- `PageFlushRecoveryReceipt`

**Warnings**

- Do not treat a clean page as current without comparing pageLSN to the replay
  frontier.
- Do not publish dirty pages before required WAL durability evidence exists.
- Do not let S.2 dirty publication evidence become a recovery claim by itself.
- Do not flush a data page containing unadmitted physical mutations under S.4's
  redo-only recovery law unless a rollback image or explicit undo-capable
  future posture protects it.

**Test requirements**

- Adversarial equivalence: the same dirty publication sequence produces the
  same pageLSN frontier and redo eligibility after restart.
- Adversarial denial: page flush before WAL durability, stale pageLSN, missing
  pageLSN, and mismatched page generation deny or require redo.
- No-undo proof: unacknowledged or unadmitted dirty page bytes cannot reach
  durable media under redo-only S.4 unless the page is protected by a declared
  rollback image posture; otherwise publication denies before flush.
- Idempotence proof: applying redo to a stale page and reapplying the same redo
  after restart converges to the same page state.

**Engineering decisions**

- PageLSN is physical replay metadata, not semantic truth.
- WAL-before-data is enforced before page publication becomes durable.
- S.4 is redo-only; it prevents unrecoverable unadmitted data-page publication
  instead of promising undo.
- Dirty publication and recovery replay share counters but not authority.

**Open questions**

- None.

### Phase 5: Define Checkpoint Manifest And Durable Cutover Law

Phase 5 makes checkpoints real bounded-recovery anchors rather than snapshots
of whatever files happen to exist.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-physical-format`
- `worth-store-certification`

**Relevant APIs**

- `CheckpointManifest`
- `CheckpointId`
- `CheckpointLocator`
- `DurableRootSelector`
- `SuperblockRingCheckpointPointer`
- `CheckpointCoveredLsnRange`
- `CheckpointPageLsnFrontier`
- `CheckpointRootPosture`
- `SharpCheckpointCertificationMode`
- `FuzzyCheckpointCertificationModeDenial`
- `CheckpointRedoBoundary`
- `CheckpointPublicationPlan`
- `CheckpointCutoverReceipt`
- `WalRetentionEligibility`
- `CheckpointValidationDenial`

**Warnings**

- Do not publish checkpoint validity before manifest, root, pageLSN frontier,
  and durability evidence are admitted together.
- Do not treat a checkpoint manifest as authority over corrupted or S.3-blocked
  records.
- Do not rely on directory listing residue to decide latest valid checkpoint;
  directory listing may discover candidates only.
- Do not imply fuzzy checkpointing unless begin/end checkpoint records, redo
  frontier, dirty-page table evidence, and interleaving constraints are
  explicitly present.
- Do not truncate or recycle WAL before a covering checkpoint and contiguous
  tail basis are durably admitted.

**Test requirements**

- Adversarial equivalence: two independently constructed checkpoint manifests
  over the same physical basis validate to the same checkpoint identity.
- Adversarial denial: torn manifest, lost directory entry, missing root,
  stale pageLSN frontier, and S.3 recovery-blocking damage reject checkpoint
  validity.
- Locator proof: candidate checkpoints discovered by directory listing,
  backend residue, or orphaned manifests cannot become selected checkpoint
  basis without a durable root selector, superblock ring pointer, manifest
  pointer, or equivalent Store-owned locator.
- Cutover proof: crash before, during, and after checkpoint cutover yields one
  deterministic selected checkpoint or typed no-valid-checkpoint posture.
- Capture-mode proof: S.4 certifies `SharpCheckpointCertificationMode`; fuzzy
  checkpoint attempts deny unless they carry begin/end records, redo boundary,
  dirty-page table evidence, and S.5-compatible interleaving assumptions.
- WAL-retention proof: attempts to delete, recycle, or exclude a WAL segment
  before covering checkpoint admission and contiguous-tail proof deny.

**Engineering decisions**

- Checkpoints bound recovery; they are not semantic authority.
- Checkpoint cutover is a publication boundary with durability preconditions.
- Checkpoint discovery and checkpoint admission are separate phases.
- S.4 starts with sharp checkpoint certification; S.5 may later broaden stable
  interleavings rather than S.4 pretending to own them early.
- WAL retention is correctness law, not cleanup optimization.
- Checkpoint validation precedes WAL tail replay planning.

**Open questions**

- None.

### Phase 6: Define Recovery Source Precedence

Phase 6 fixes the recovery decision graph before redo execution can choose
what to trust.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-physical-integrity`
- `worth-store-certification`

**Relevant APIs**

- `RecoverySourcePrecedenceGraph`
- `RecoverySourceCandidate`
- `RecoveryCandidateDiscoveryTrace`
- `AdmittedRecoverySource`
- `RecoverySourceApplicationRole`
- `BackendResidueRejection`
- `CheckpointBaseAdmission`
- `WalTailRedoSource`
- `PageLsnSkipApplyDecision`
- `CompactionCutoverRecoveryPosture`
- `CompactionGenerationVisibility`
- `CompactionArtifactResidueRejection`
- `RecoverySourceDecisionTrace`

**Warnings**

- Do not let pages, manifests, WAL, checkpoint records, compaction products, or
  snapshots race by whichever is easiest to parse.
- Do not let backend residue outrank admitted checkpoint/WAL evidence.
- Do not treat checkpoint locators, checkpoint manifests, WAL frames, page
  images, pageLSNs, compaction products, and residue as interchangeable
  "sources"; they have different recovery roles.
- Do not implement compaction cutover authority here beyond S.4 recovery
  classification; S.5/S.8/S.10 own later movement and repair depth.

**Test requirements**

- Adversarial equivalence: the same persisted bytes always select the same
  recovery sources under the same profile.
- Adversarial denial: stale residue, orphaned checkpoint manifests, invalid
  compaction products, and S.3-blocked records cannot become admitted sources.
- Precedence proof: checkpoint-plus-WAL-tail, WAL-only, no-valid-checkpoint,
  and recovery-blocked classifications remain distinct.
- Role proof: valid checkpoint locator beats orphaned manifests; valid
  checkpoint plus contiguous WAL tail beats stale data pages; WAL tail supplies
  redo while pageLSN decides skip/apply; backend residue discovers candidates
  but certifies none.
- Compaction proof: visible compaction products require generation identity,
  admitted cutover record, and old-generation recoverability until cutover
  durability is admitted; otherwise they reject as residue.

**Engineering decisions**

- Source precedence is a typed graph, not `if exists then use it`.
- Source precedence has three layers: candidate discovery, source admission,
  and application role.
- Residue is evidence only when admitted by the source graph.
- S.4 owns only minimum compaction visibility needed for crash recovery; later
  sequences own compaction strategy and movement depth.
- Recovery planning consumes source decisions rather than rediscovering them.

**Open questions**

- None.

### Phase 7: Plan And Execute Idempotent Redo Replay

Phase 7 lowers admitted recovery sources into a redo plan and applies it
without executor-side rediscovery.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-buffer-pool`
- `worth-store-certification`
- `worth-proof`

**Relevant APIs**

- `RecoveryRedoPlan`
- `WalValidPrefix`
- `TornWalTailClassification`
- `MiddleWalCorruptionDenial`
- `MissingAcknowledgedWalRangeDenial`
- `AdmittedRedoFrame`
- `RedoRecordGrammar`
- `RedoRecordTargetGeneration`
- `RedoRecordOperationForm`
- `RedoRecordIdempotenceBasis`
- `RedoApplicationCursor`
- `RedoExecutionReceipt`
- `SkippedRedoFrameReport`
- `RecoveredPhysicalState`

**Warnings**

- Do not let the redo executor choose recovery strategy.
- Do not revalidate integrity in redo execution except at explicit trust
  boundary shifts; S.3 already supplied integrity proof.
- Do not update pages whose pageLSN proves the redo already landed.
- Do not treat a torn WAL tail, middle corruption, missing acknowledged range,
  stale segment, or wrong generation as the same recovery condition.
- Do not admit a redo record whose target page/generation, operation form,
  integrity binding, idempotence basis, or pageLSN comparison basis is missing.

**Test requirements**

- Adversarial equivalence: checkpoint-plus-tail replay and canonical control
  rebuild converge to the same recovered physical root for acknowledged work.
- Adversarial denial: frames outside admitted source range, wrong pageLSN
  basis, or recovery-blocking damage cannot enter the redo plan.
- Valid-prefix proof: recovery admits the maximal contiguous integrity-vetted
  WAL prefix from the selected basis; torn tail suffix, middle corruption,
  stale generation, and missing acknowledged range produce distinct outcomes.
- Grammar proof: redo records without target generation, operation form,
  integrity binding, idempotence basis, or pageLSN basis deny before execution.
- Idempotence proof: replaying the same admitted tail twice produces identical
  recovered state and exact skipped-frame counters.

**Engineering decisions**

- Planning and execution are separate proof-bearing phases.
- Redo consumes only admitted frames and pageLSN facts.
- Redo effects are exactly-once in recovered state; redo records may be scanned,
  planned, skipped, or reapplied repeatedly according to pageLSN/idempotence
  law.
- Skipped frames are evidence, not silent optimization.

**Open questions**

- None.

### Phase 8: Classify Partial Publication And Unacknowledged Work

Phase 8 makes crash-edge ambiguity explicit instead of allowing recovery to
guess based on residue.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-certification`

**Relevant APIs**

- `PartialPublicationClassification`
- `UnacknowledgedPublicationOutcome`
- `TornPublicationDenial`
- `AmbiguousPublicationReport`
- `RecoveredOrRejectedPartialPublication`
- `UnadmittedDurablePageMutationDenial`
- `NoUndoPartialPublicationClassification`
- `RollbackImageRequiredPosture`

**Warnings**

- Do not silently treat unacknowledged residue as acknowledged truth.
- Do not discard durable WAL records merely because acknowledgment was not
  observed in live memory.
- Do not classify a flushed page containing unadmitted physical mutations as
  harmless under redo-only recovery.
- Do not collapse completed-through-replay, rejected, ambiguous, and blocked
  outcomes into success/failure.

**Test requirements**

- Adversarial equivalence: repeated recovery over identical partial
  publication bytes produces identical classification.
- Adversarial denial: backend residue, live acknowledgment memory, and logs
  cannot promote unacknowledged work into acknowledged truth.
- No-undo denial: unacknowledged dirty bytes already present in durable page
  images require typed denial, rollback-image posture, or explicit
  undo-capable deferral; redo-only replay cannot silently accept them.
- Edge proof: crash before WAL append, after WAL append before durability,
  after durability before ack, after ack before page flush, and during
  checkpoint cutover all classify distinctly.

**Engineering decisions**

- Acknowledgment memory is not recovery authority after crash.
- Unacknowledged durable WAL may be replayable, rejected, or classified by
  typed rules, but never guessed from residue.
- Durable page images containing unadmitted physical mutations are a recovery
  hazard under redo-only S.4, not an ordinary ambiguous case.
- Ambiguity is a first-class recovery output when evidence is insufficient.

**Open questions**

- None.

### Phase 9: Bound Recovery Work By Checkpoint Interval And WAL Tail

Phase 9 makes recovery cost a declared contract.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-buffer-pool`
- `worth-store-budgets`
- `worth-store-certification`

**Relevant APIs**

- `RecoveryBudget`
- `CheckpointIntervalContract`
- `WalTailReplayBudget`
- `RecoveryMemoryEnvelope`
- `RecoveryCounterSnapshot`
- `RecoveryBudgetDenial`

**Warnings**

- Do not hide full-store scans inside checkpoint discovery, source precedence,
  or offline verifier lanes.
- Do not accept elapsed time as recovery-boundedness proof.
- Do not let profile-rich diagnostics widen the operational recovery path.

**Test requirements**

- Adversarial equivalence: recovery over equivalent checkpoint/tail envelopes
  produces the same recovered state regardless of total store size.
- Adversarial denial: recovery plans exceeding WAL-tail, memory, allocation,
  or checkpoint-discovery budgets deny or degrade explicitly before execution.
- Counter proof: replayed frames, skipped frames, validated checkpoints,
  scanned segments, page redos, and memory envelope use match exact expected
  counters.

**Engineering decisions**

- Recovery time is a structural counter claim before it is a wall-clock claim.
- Checkpoint cadence is part of the recovery contract.
- Diagnostic richness cannot make ordinary recovery unbounded.

**Open questions**

- The implementation must pick CI-scale checkpoint/tail profiles and name
  larger local-soak and release-certification profiles separately.

### Phase 10: Certify Crash Matrix Through The Roadmap 2 Harness

Phase 10 proves the crash/fault machinery itself is real enough for S.4
closeout.

**Relevant subsystems**

- `worth-store-certification`
- `worth-store-test-support`
- `worth-store-recovery-physics`

**Relevant APIs**

- `RecoveryPhysicsScenarioDefinition`
- `RecoveryPhysicsScenarioPlan`
- `FaultSchedulerDriver`
- `StorageBoundaryInterposerDriver`
- `FreshRuntimeRecoveryDriver`
- `RecoveryPhysicsCrashMatrix`
- `RecoveryPhysicsCertificationMatrix`
- `RecoveryPhysicsMutationValidationMatrix`

**Warnings**

- Do not simulate crashes by keeping the same runtime objects alive.
- Do not mutate private structs after the fact to create crash conditions.
- Do not let test support own oracle meaning.

**Test requirements**

- Adversarial crash matrix: crashes around WAL append, page flush, checkpoint
  write, checkpoint cutover, compaction cutover, acknowledgment, directory sync,
  and rename durability all pass through lowered scenario plans.
- Adversarial denial: live-state reuse, logs, backend residue guessing, direct
  private mutation, and same-run self-comparison fail certification.
- Harness proof: each crash lane names driver, observer, oracle, transcript,
  evidence bundle, seed, backend profile, and exact counter expectations.
- Mutation proof: each required S.4 mutant fails the intended suite lane with
  named oracle evidence, counter evidence, or compile-fail evidence.

**Engineering decisions**

- The crash harness is part of the product architecture for Roadmap 2.
- Faults are delivered through production-like storage boundaries.
- S.4 closeout depends on CI Certification mode for required S.4 harness
  subsystems.

**Open questions**

- None.

### Phase 11: Add Offline Verifier And Recovery Determinism Proof

Phase 11 proves recovery conclusions are independently checkable from
persisted bytes.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-certification`
- `worth-store-test-support`

**Relevant APIs**

- `RecoveryOfflineVerifier`
- `OfflineRecoveryVerificationReport`
- `RuntimeRecoveryComparisonReport`
- `RecoveryDeterminismReport`
- `PersistedRecoveryArtifactDigest`

**Warnings**

- Do not call the live recovery authority path from the offline verifier.
- Do not compare only broad success/failure.
- Do not allow map iteration, wall-clock metadata, thread timing, or environment
  state to change recovery classification.

**Test requirements**

- Adversarial equivalence: identical persisted bytes, format version, backend
  profile, and recovery profile produce identical recovery classifications
  across repeated fresh-runtime recovery runs.
- Adversarial disagreement: verifier/runtime disagreement is surfaced as typed
  evidence, not hidden by the test harness.
- Independence proof: offline verification inspects persisted physical records
  without constructing the live Store runtime or reusing runtime caches.

**Engineering decisions**

- Offline verifier evidence can disagree with live recovery.
- Determinism compares classifications, recovered physical state, counters,
  verifier conclusions, and explicitly allowed nondeterministic metadata.
- Verifier independence is a certification boundary, not a packaging detail.

**Open questions**

- None.

### Phase 12: Materialize Foundational And Proof Evidence

Phase 12 exports executed recovery evidence through shared vocabulary without
moving recovery authority out of Store.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-certification`
- `worth-foundational`
- `worth-proof`

**Relevant APIs**

- `RecoveryPhysicsReceipt`
- `RecoveryPhysicsReport`
- `RecoveryCounterPerformanceReceipt`
- `RecoverySourceDecisionReport`
- `ProofProgressionRecoveryTrace`
- `FoundationalRecoveryEvidenceBundle`
- `AspectValue`
- `CanonicalAspectStateMap`
- `ContractValidatedAspectValue`
- `AspectKey`
- `AspectLocator`
- `AspectValueLocator`
- `BoundaryArtifactLocator`
- `BoundarySourceLocator`
- `BoundaryMismatchLocator`
- `BoundaryArtifactId`
- `BoundaryHandle`
- `BoundaryEpoch`
- `CanonicalDigestId`
- `EquivalenceBasisId`
- `FoundationalBoundaryMaterializationPlan`
- `FoundationalBoundaryMaterializationDecisionRow`
- `FoundationalBoundaryMaterializationBundle`
- `CurrentBasisBoundaryArtifact`
- `BoundaryBridgedCurrentBasisBoundaryArtifact`
- `FoundationalBoundaryEvidenceCompletedReceiptArtifact`
- `FoundationalBoundaryEvidenceProvenanceArtifact`
- `FoundationalBoundaryEvidenceSupportRecoveryPosture`
- `FoundationalBoundaryEvidenceReconstructedEquivalenceArtifact`
- `FoundationalBoundaryEvidenceReplayDerivedLineageArtifact`
- `FoundationalBoundaryEvidenceRestoredLineageArtifact`
- `FoundationalBoundaryEvidenceRuntimeAssumption`
- `FoundationalBoundaryEvidenceRuntimeNonAssumption`
- `FoundationalDiagnosticDecisionRow`
- `FoundationalDiagnosticFailureRow`
- `FoundationalDiagnosticComparisonRow`
- `FoundationalDiagnosticSupportRow`
- `FoundationalDiagnosticNamedGap`
- `FoundationalDiagnosticAbsenceCause`
- `FoundationalDiagnosticOutcomeKind`
- `FoundationalDiagnosticSupportReport`
- `FoundationalDiagnosticExplanationBundle`
- `FoundationalCertifiedDiagnosticBundle`
- `BoundaryBridgedCertifiedDiagnosticBundle`
- `FoundationalCounterBackedPerformanceReceipt`
- `FoundationalPolicyAdmissionReceipt`
- `FoundationalPerformanceLayoutIntent`
- `FoundationalPerformanceEvidenceStrength`
- `FoundationalPerformanceExecutionTemperature`
- `FoundationalPerformanceFreshnessRetentionPosture`
- `FoundationalPerformanceFallbackDebtPosture`
- `FoundationalPerformanceBreadthLocalityPosture`
- `FoundationalCertifiedPerformanceBundle`
- `CanonicalBasisBundle`
- `CanonicalDerivedDigest`
- `FoundationalProfileSet`
- `FoundationalProfileMaterializationPlan`
- `Artifact`
- `Recipe<Lowered>`
- `ExecutionReadyRecipe`
- `ExecutedRecipe`
- `TransitionReadiness`
- `TransitionOutcome`
- `AuthorityWitness`
- `CapabilityWitness`
- `AssumptionBasis`
- `FreshnessScopedBasis`
- `BoundaryBridged`
- `CanonicalVec`
- `UniqueVec`
- `NonEmpty`

**Warnings**

- Do not build receipts from plans alone.
- Do not let Foundational reports become Store recovery witnesses.
- Do not let Proof progression wrappers own WAL, checkpoint, or pageLSN
  meaning.
- Do not let reduced-richness profiles change recovered state.
- Do not use raw canonical digests or raw basis rows as current-basis recovery
  authority.
- Do not use Proof recipes as dynamic recovery engines; they encode the
  already-planned Store recovery progression.
- Do not skip Foundational production-readiness requirements for the specific
  surface families S.4 consumes.
- Do not let JSON-shaped payloads, raw bytes, debug strings, or display names
  become the canonical boundary meaning of recovery evidence.
- Do not collapse missing evidence, redacted evidence, partial diagnostic
  coverage, verifier disagreement, and domain denial into one failure bucket.
- Do not let policy-admission performance receipts, support-derived replay
  claims, or stale/readmitted performance claims satisfy counter-backed
  current-basis execution-cost truth.
- Do not model WAL replay or checkpoint recovery as branch merge, committed
  authority transition, scoped merge, selected-node, selected-aspect, or
  cherry-pick vocabulary.

**Test requirements**

- Adversarial equivalence: the same executed recovery findings materialize the
  same Foundational report, receipt, performance, and profile basis through
  independent constructors.
- Adversarial denial: planned recovery, copied receipt fields, log excerpts,
  and same-run self-comparison cannot satisfy recovery evidence APIs.
- Aspec boundary proof: recovery evidence payloads, diagnostics, mismatch
  loci, counters, and S.5 handoff fields lower through Foundational values,
  identities, handles, epochs, digest ids, and locators; JSON-shaped payloads,
  raw bytes, debug strings, display names, and producer-private names deny.
- Profile proof: reduced-richness profiles remove optional forensic detail
  while preserving recovered state, source decisions, denials, and counters.
- Authority proof: Foundational reports and Proof-compatible traces cannot be
  fed back into Store APIs that require `RecoveredPhysicalState`,
  `RecoveryRedoPlan`, or `DurableAckReceipt`.
- Canonical basis proof: materialized recovery reports, receipts, performance
  rows, diagnostic rows, and evidence bundles lower through Foundational
  canonical basis APIs and produce stable digests across independent
  construction paths.
- Current-basis proof: current-basis boundary artifacts and evidence
  attachments require Store recovery authority plus Foundational admission or
  readmission; raw digests, raw reports, and boundary-bridged stale forms deny.
- Diagnostics proof: recovery source decisions, partial publication outcomes,
  verifier disagreements, missing evidence, redaction, unsupported evidence,
  named gaps, partial coverage, and budget denials materialize as typed
  Foundational diagnostic rows, reports, explanation bundles, certified
  bundles, and readmitted bundles with locator/subject meaning, not prose.
- Performance proof: recovery budget, replay breadth, checkpoint validation,
  skipped-frame, page-redo, verifier-read, and residue-rejection counters
  materialize as Foundational performance claims that distinguish
  recovery-only, cold replay, verifier, materialization, support-only,
  policy-admission, counter-backed, freshness/retention, fallback/debt, and
  certified/readmitted surfaces with exact counter assertions.
- Lineage/provenance proof: replay-derived, restored/readmitted, reconstructed
  equivalence, direct continuity, runtime assumption, and runtime
  non-assumption postures remain distinct in recovery evidence.
- Non-applicable-surface proof: branch, merge, commit, scoped-merge,
  selected-node, selected-aspect, skipped-scope, and cherry-pick surfaces fail
  when used as WAL replay, source-precedence, checkpoint-cutover, or recovered
  state admission mechanisms.
- Proof progression proof: recovery entry, lowered redo plan,
  execution-ready replay, executed replay, stale/rebind-required restart, and
  boundary readmission use Proof checked outcomes and assumptions rather than
  local typestate lookalikes.
- Proof collection proof: WAL replay order uses canonical-order proof where
  required, source candidates use uniqueness/non-empty proofs where required,
  and disjoint recovery source families cannot be represented as overlapping
  inputs.

**Engineering decisions**

- Store counters and executed recovery findings are the evidence source.
- Foundational standardizes exported boundary meaning.
- Proof standardizes progression shape where S.4 states need sealed movement.
- Foundational readiness requirements are prerequisites for exporting the
  corresponding S.4 evidence family.
- Proof assumptions carry recovery basis freshness and trust-boundary
  readmission; Store still owns the recovery basis meaning.

**Open questions**

- None.

### Phase 13: Close WAL, Checkpoint, LSN, And Recovery Physics

Phase 13 runs the named S.4 suites, rejects synthetic shortcuts, verifies
bounded recovery, and publishes S.5 physical-isolation readiness.

**Relevant subsystems**

- `worth-store-recovery-physics`
- `worth-store-certification`
- `worth-store-readiness`
- `worth-store-buffer-pool`
- `worth-store-physical-integrity`
- `worth-foundational`
- `worth-proof`

**Relevant APIs**

- `WalCheckpointLsnRecoveryPhysicsSuite`
- `RecoveryPhysicsCertificationBundle`
- `RecoveryPhysicsCloseoutReport`
- `SyntheticRecoveryShortcutRejectionReport`
- `S5PhysicalIsolationRecoveryReadiness`

**Warnings**

- Do not close S.4 on "restart worked once."
- Do not close S.4 on same-process recovery.
- Do not claim S.5 isolation, S.6 I/O QoS, S.10 PITR/repair, S.11 security, or
  S.12 aerospace-grade certification from S.4.
- Do not leave S.5 with ambiguous recovered roots or untyped page stability
  assumptions.

**Test requirements**

- Adversarial closeout: crash points around WAL append, page flush, checkpoint
  manifest write, checkpoint cutover, compaction cutover, acknowledgment,
  directory sync, and rename durability recover deterministically.
- Adversarial denial: raw bytes, live-state reuse, backend residue guessing,
  unsupported durability claims, invalid pageLSNs, torn checkpoints, and
  unbounded recovery plans deny or classify at named S.4 boundaries.
- Boundedness proof: recovery counters prove work is bounded by checkpoint
  interval and WAL tail.
- Handoff proof: S.5 receives a recovered physical root, admitted pageLSN
  frontier, replay receipts, source-precedence trace, and stability assumptions
  explicit enough to begin physical isolation work.
- Line-cap and composition proof: recovery modules and tests stay under the
  workspace line cap unless explicitly exempted and keep WAL, checkpoint,
  pageLSN, source precedence, replay, verifier, and evidence responsibilities
  separate.

**Engineering decisions**

- S.4 closeout proves physical recovery only.
- S.4 explicitly reserves physical read stability, I/O QoS, blobs, repair,
  security, and full physical database certification for later sequences.
- The closeout handoff must be concrete enough for S.5 to start without
  reopening recovery physics.

**Open questions**

- None.

## Must Ship

- typed consumption of `S4RecoveryPhysicsIntegrityReadiness`
- typed consumption of S.2 recovery memory envelopes and dirty publication
  evidence
- WAL segment, LSN interval, replay cursor, gap, duplicate, and stale-segment
  law
- valid WAL prefix classification for torn tails, middle corruption, missing
  acknowledged ranges, and stale-generation segments
- redo record grammar with target generation, operation form, integrity
  binding, idempotence basis, and pageLSN comparison basis
- WAL append, flush, sync, directory, rename, and backend durability barrier
  contracts
- named backend durability profile matrix with profile-scoped
  `DurableAckReceipt` evidence
- acknowledgment precondition law and illegal acknowledgment denials
- pageLSN comparison, stale-page classification, and WAL-before-data ordering
- redo-only/no-undo publication law preventing unadmitted dirty page bytes from
  durable publication without rollback image or later undo posture
- checkpoint manifest, covered range, pageLSN frontier, root posture, durable
  publication, and cutover law
- checkpoint locator law, sharp checkpoint certification mode, fuzzy-checkpoint
  denial unless fully specified, and WAL retention/truncation law
- deterministic recovery source precedence graph
- source role separation for candidate discovery, source admission, and
  application role, plus minimum compaction visibility law
- idempotent redo planning and execution
- partial publication and unacknowledged work classification
- recovery budget contracts tied to checkpoint interval and WAL tail
- Roadmap 2 crash/fault scenario definitions, lowered plans, drivers,
  observers, oracles, transcripts, and evidence bundles
- S.4 certification matrix tying every suite to lane, fixture/profile, fault
  seam, observer, oracle, counter, evidence, and expected mutant failure
- mutation-style harness validation for known S.4 recovery defects
- offline verifier and recovery determinism proof
- Foundational boundary artifacts, role claims, materialization plans,
  current-basis boundary exports, aspec-native boundary payloads, canonical
  basis bundles, diagnostics, boundary-evidence attachments,
  profile-controlled materialization, performance and layout/freshness/cost
  posture, production-readiness gates, and projection-authority denials from
  executed Store recovery findings
- explicit non-applicability denial for Foundational branch, merge, commit,
  scoped-merge, selected-node, selected-aspect, skipped-scope, and cherry-pick
  vocabulary as recovery physics mechanisms
- Proof artifacts, proof sets, authority witnesses, capability witnesses,
  recipes, checked transition outcomes, transition readiness, assumptions,
  boundary readmission forms, proof-bearing collections, and fixed-shape
  join/fork helpers where S.4 needs proof-bearing recovery progression
- synthetic shortcut rejection for live-state reuse, logs, backend residue
  guessing, private mutation, and same-run self-comparison
- concrete `S5PhysicalIsolationRecoveryReadiness` handoff payload

## Must Preserve

- Store owns recovery physics.
- `worth-relational` owns semantic truth and transaction meaning.
- `worth-foundational` owns shared boundary evidence vocabulary, not recovery
  authority.
- `worth-proof` owns progression law, not WAL semantics.
- S.3 owns physical integrity classification; S.4 consumes it.
- WAL is not semantic authority.
- Checkpoints are recovery anchors, not unrebuildable truth.
- Backend residue never outranks admitted recovery sources.
- Recovery tests discard live heap and cache state.
- Reduced-richness profiles do not change recovered state.

## Acceptance Evidence

S.4 is complete only when the store satisfies the Roadmap 2 named suite:

- `WAL/checkpoint/LSN recovery-physics test`

Required acceptance suites:

- `recovery_entry_authority_suite`
  proves S.4 consumes typed S.3/S.2 handoff evidence and rejects raw recovery
  bytes, copied reports, and unbounded envelopes.
- `wal_lsn_topology_suite`
  proves WAL segment ordering, LSN gaps, duplicates, stale segments, and
  replay cursor determinism.
- `valid_wal_prefix_classification_suite`
  proves torn tail suffixes, middle corruption, stale-generation segments, and
  missing acknowledged WAL ranges classify differently and cannot be silently
  skipped.
- `durability_barrier_and_ack_suite`
  proves acknowledgment cannot precede WAL durability barriers required by the
  backend profile.
- `backend_profile_certification_suite`
  proves every durable ack receipt and recovery verdict is scoped to a named
  backend profile and that profile-specific receipts cannot certify other
  profiles.
- `wal_before_data_page_lsn_suite`
  proves page flush and pageLSN behavior cannot outrank WAL-before-data law.
- `no_undo_publication_suite`
  proves S.4's redo-only mode prevents unadmitted dirty page bytes from durable
  publication unless rollback-image posture or explicit undo-capable future
  posture is present.
- `checkpoint_manifest_publication_suite`
  proves checkpoint manifests, pageLSN frontiers, root posture, and durable
  cutover validate or deny together.
- `checkpoint_locator_capture_mode_suite`
  proves directory listing discovers candidates only, durable checkpoint
  locators select admitted checkpoint basis, sharp checkpoint mode is the S.4
  certification mode, and fuzzy checkpoint attempts deny unless fully specified.
- `wal_retention_truncation_suite`
  proves WAL segments cannot be deleted, recycled, or excluded from recovery
  until a covering checkpoint, recoverable locator, admitted frontier, and
  contiguous WAL tail are all proven.
- `recovery_source_precedence_suite`
  proves checkpoint, WAL tail, pages, manifests, compaction products, and
  backend residue follow typed precedence rather than parse order.
- `source_role_compaction_visibility_suite`
  proves checkpoint locators, manifests, WAL tails, pageLSN/page images,
  compaction products, and backend residue keep distinct discovery, admission,
  and application roles, and that uncutover compaction artifacts reject.
- `idempotent_redo_replay_suite`
  proves repeated redo over the same checkpoint and WAL tail converges with
  exact replayed/skipped counters.
- `redo_record_grammar_suite`
  proves redo records cannot enter execution without target generation,
  operation form, integrity binding, idempotence basis, and pageLSN comparison
  basis.
- `partial_publication_classification_suite`
  proves unacknowledged, torn, incomplete, ambiguous, completed-through-replay,
  and rejected outcomes remain distinct.
- `bounded_recovery_budget_suite`
  proves recovery work is bounded by checkpoint interval plus WAL tail and
  exact resource-envelope counters.
- `crash_matrix_fault_scheduler_suite`
  proves every required crash seam is delivered through lowered plans,
  production-like storage boundaries, and the crash harness.
- `fresh_runtime_crash_isolation_suite`
  proves crash recovery discards live heap, caches, buffer-pool state, mmap
  views, singletons, and runtime references.
- `offline_verifier_independence_suite`
  proves persisted bytes can be inspected without the live recovery authority
  path and that verifier/runtime disagreement is explicit evidence.
- `recovery_determinism_suite`
  proves identical persisted bytes, profiles, and format versions recover to
  identical classifications, counters, and verifier conclusions.
- `foundational_recovery_evidence_suite`
  proves executed Store recovery findings materialize into Foundational
  reports, receipts, and performance evidence without becoming Store recovery
  authority.
- `foundational_aspec_boundary_payload_suite`
  proves recovery evidence payloads, diagnostic subjects, mismatch loci,
  counters, canonical basis entries, and S.5 handoff payloads lower through
  Foundational aspec-native values, identities, handles, epochs, digest ids,
  and locators rather than JSON-shaped objects, debug strings, raw bytes, or
  producer-private names.
- `foundational_diagnostic_recovery_bundle_suite`
  proves S.4 recovery decisions, failures, mismatches, missing evidence,
  named gaps, partial coverage, certified coverage, materialization
  availability, and readmitted diagnostic bundles use Foundational diagnostics
  while remaining descriptive and non-authoritative.
- `foundational_recovery_performance_claim_suite`
  proves S.4 recovery-only, cold replay, verifier, materialization,
  support-only, policy-admission, and counter-backed performance claims use
  Foundational performance boundary, evidence-strength, freshness/retention,
  layout-intent, allocation, fallback/debt, and exact-counter vocabulary
  without implying shared storage, shared telemetry, or current-basis hot-path
  truth.
- `foundational_recovery_adoption_suite`
  proves S.4 uses available Foundational boundary-artifact, role,
  materialization, current-basis, aspec, identity, locator, canonicalization,
  profile, diagnostic, boundary-evidence, lineage/provenance/receipt,
  performance, and production-readiness surfaces, while rejecting derived
  projections, support-only artifacts, planned-work artifacts,
  receipt-evidence artifacts, raw canonical digests, and copied boundary
  evidence as Store recovery witnesses.
- `foundational_non_applicable_surface_denial_suite`
  proves branch, merge, commit, scoped-merge, selected-node, selected-aspect,
  skipped-scope, and cherry-pick vocabulary cannot model WAL replay, recovery
  source precedence, checkpoint cutover, or S.4 recovered-state admission.
- `proof_progression_recovery_state_suite`
  proves integrity-vetted, recovery-admitted, replay-planned,
  replay-executed, checkpoint-validated, recovered, and closeout-ready forms
  remain mechanically distinct.
- `proof_recovery_adoption_suite`
  proves S.4 uses available Proof artifacts, proof sets, authority witnesses,
  capability witnesses, recipes, checked transitions, readiness outcomes,
  assumptions, boundary readmission forms, canonical/unique/non-empty
  collections, and fixed-shape join/fork helpers, while denying any Proof
  object as the owner of WAL, LSN, checkpoint, pageLSN, or recovery-source
  semantics.
- `synthetic_recovery_test_rejection_suite`
  proves logs, live-state reuse, backend residue guessing, direct private
  mutation, same-run self-comparison, and test-support-owned oracles fail
  certification.
- `recovery_mutation_validation_suite`
  proves every required S.4 mutant fails the intended suite lane and emits
  mutation coverage evidence.
- `s5_recovery_readiness_handoff_suite`
  proves S.5 receives recovered root, pageLSN frontier, replay receipts,
  source-precedence trace, recovery counters, and explicit stability
  assumptions.

Every suite must map to its scenario definitions, lowered plans, required
drivers, observers, proof oracles, transcript families, evidence bundle fields,
positive control lane, hostile lane, reopen lane, and forbidden-shortcut lane.
Every suite must also name exact counters that must be positive, exact counters
that must remain zero, and any mutants it is expected to kill.

## Allowed Debt

S.4 may reserve advanced checkpoint profitability heuristics, backend-specific
WAL layout acceleration, and richer operator recovery UX for later sequences
when the ordinary recovery law already exists.

S.4 may not mark these as debt:

- typed S.3/S.2 entry consumption
- WAL segment and LSN topology
- valid WAL prefix classification
- redo record grammar
- durability barriers for acknowledged work
- named backend durability profile certification
- WAL-before-data ordering
- redo-only/no-undo publication enforcement
- pageLSN redo eligibility
- checkpoint manifest and cutover validity
- checkpoint locator authority
- declared checkpoint capture mode
- WAL retention and truncation safety
- deterministic source precedence
- source discovery/admission/application-role separation
- compaction visibility and cutover-admission posture
- idempotent redo
- partial publication classification
- bounded recovery by checkpoint interval and WAL tail
- crash harness with fresh-runtime recovery
- offline verifier independence
- recovery determinism
- exact recovery counters
- Foundational adoption of boundary artifacts, role claims, materialization,
  current-basis boundary export, aspec payloads, identities, locators,
  canonicalization, profiles, diagnostics, boundary evidence,
  lineage/provenance/receipt surfaces, performance receipts, layout/cost
  posture, and production-readiness gates
- Foundational non-applicability denials for branch, merge, commit,
  scoped-merge, selected-node, selected-aspect, skipped-scope, and cherry-pick
  vocabulary as recovery mechanisms
- Proof adoption of artifacts, witnesses, recipes, checked outcomes,
  transition readiness, assumptions, boundary readmission, structural
  collections, and fixed-shape progression helpers
- projection-versus-authority denial for all Foundational and Proof evidence
  surfaces that are not Store recovery witnesses
- S.4 certification matrix
- mutation-style validation for required recovery defects
- synthetic shortcut rejection
- concrete S.5 readiness handoff

## Sequencing Notes

S.4 belongs immediately after S.3 because recovery must consume
integrity-vetted records and typed damage maps. It belongs before S.4.5 because
the shared physical simulation harness needs S.4's crash/fault mechanics,
fresh-runtime recovery discipline, and S.5 readiness evidence before it can
generalize the Roadmap 2 harness substrate. It belongs before S.5 through S.4.5
because physical isolation needs both recovered roots/pageLSN frontiers and the
deterministic hostile interleaving harness that will exercise stable read plans
during maintenance.

Later sequences consume S.4 as follows:

- S.4.5 consumes S.4 crash/fault harness lessons, recovered roots, pageLSN
  frontiers, replay receipts, source-precedence traces, and recovery stability
  assumptions to build the reusable physical simulation harness.
- S.5 consumes recovered roots, pageLSN frontiers, replay receipts, recovery
  stability assumptions, and S.4.5 simulation harness readiness.
- S.6 consumes durability-barrier and backend-profile facts before expanding
  hardware-aware I/O and QoS.
- S.7 consumes recovery ordering for chunk-tree and blob-publication recovery
  once native blob chunks exist.
- S.8 consumes WAL/checkpoint recovery rules when index/layout families define
  rebuild and amplification contracts.
- S.9 formalizes the crash, recovery, checkpoint, and source-precedence state
  machines.
- S.10 consumes S.4 recovery physics for PITR, offline verification, backup,
  restore, disaster recovery, and repair workflows.
- S.12 consumes S.4 as one required physical database certification lane.

## Required Self-Check

Before closeout, answer these with evidence:

- Does S.4 make crash recovery deterministic, bounded, and physical-rule driven
  without residue guessing, live state, duplicate materialized effects, or lost
  acknowledged truth?
- Does every phase still map to named Store-owned types, modules, and tests
  rather than relying on Foundational, Proof, or Relational authority?
- Does every declared recovery law have at least one positive lane, hostile
  lane, forbidden-shortcut lane, exact counter expectation, and mutant it is
  expected to kill?
- Does S.5 receive recovered roots, pageLSN frontier, replay receipts,
  WAL-retention posture, source-precedence trace, backend profile, checkpoint
  capture mode, and explicit stability assumptions?

Reopen S.4 if any of these become true:

- redo requires executor-side source rediscovery or strategy choice
- checkpoint discovery performs an unbounded store scan
- directory listing, backend residue, or orphaned manifests decide checkpoint
  freshness or generation
- an unacknowledged or unadmitted dirty page can reach durable media without
  rollback-image posture or later undo-capable admission
- a backend profile can issue `DurableAckReceipt` without naming its durability
  contract and completed barrier set
- a receipt certified under one backend durability profile satisfies another
  profile
- WAL truncation or recycling can happen before covering checkpoint admission,
  recoverable locator proof, and contiguous-tail proof
- a torn WAL tail, middle corruption, stale generation, and missing
  acknowledged WAL range collapse into one condition
- a redo record can enter planning without target generation, operation form,
  integrity binding, idempotence basis, or pageLSN comparison basis
- compaction artifacts can be admitted without generation identity, cutover
  basis, and old-generation recoverability
- recovered-state "exactly once" is implemented as "never scan or skip a redo
  record twice" instead of "physical effects materialize once in recovered
  state"
