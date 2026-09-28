# Storage Foundation S.3 Engineering Spec: Physical Integrity, Scrub, Quarantine, And Corruption Localization

> **Status:** Planned
>
> **Roadmap parent:** [physical-database-roadmap.md](./physical-database-roadmap.md)
>
> **Primary prerequisite:** `S.2 Buffer Pool, Memory Budgets, And Zero-Copy Record Access`
>
> **Follow-on storage-foundation sequence:** `S.4 WAL, Checkpoint, LSN, And Recovery Physics`
>
> **Primary architectural driver:** detect physical damage before logical decode,
> localize the damage to the smallest honest physical boundary, and publish
> proof-bearing integrity evidence without confusing checksums, digests,
> authenticity, recovery, or repair.

## Goal

Make Worth Store reject damaged physical bytes before they can become ambiguous
semantic failures.

S.3 turns checksums, torn-write detection, stale-generation detection, scrub,
quarantine, and corruption-localization reports into Store-owned physical
database law. It is complete when page, frame, manifest, index-page, WAL-frame,
extent, and blob-chunk damage are detected at the physical boundary through
bounded protected-byte access inherited from S.2, and when every outcome is
typed as intact, rebuildable derived damage, quarantined damage, or
unrecoverable authority damage without pretending to perform S.4 recovery or
S.10 repair.

## Why This Sequence Exists

S.1 gave bytes physical shape. S.2 made access to those bytes bounded and
protected. S.3 exists before WAL recovery and physical isolation because later
sequences must never have to ask whether the bytes they are replaying,
checkpointing, moving, or repairing were already physically admissible.

Physical integrity is not logical artifact identity. A canonical digest can
prove semantic identity and still say nothing about which page, frame, sector,
chunk, or manifest region was damaged. A checksum can prove physical damage and
still say nothing about authenticity. S.3 keeps those meanings separate.

## Governing Summaries

- `MENTALITY.md`
  protects hard-problem-first design. S.3 starts with hostile media damage and
  torn publication, not with a friendly checksum helper.
- `arch_laws.md`
  protects proof-bearing phase transitions. S.3 must consume S.2 readiness,
  inspect protected physical bytes, and produce typed integrity findings before
  any logical decoder or later recovery step can act.
- `composition_laws.md`
  protects named semantic steps. S.3 must not collapse checksum calculation,
  header validation, generation comparison, quarantine, scrub planning,
  evidence export, and S.4 handoff into one verifier path.
- `domain_structure_laws.md`
  protects responsibility boundaries. Page/frame integrity, manifest integrity,
  index-page integrity, WAL-frame integrity, blob-chunk integrity, scrub,
  quarantine, and evidence all fail differently and need separate Store-owned
  modules.
- `perf_laws.md`
  protects visible, testable cost. S.3 must expose exact checked-byte,
  checked-page, checksum, skipped-decode, quarantine, and scrub-window counters
  at the boundaries that claim bounded physical integrity work.
- `physical-database-roadmap.md`
  places S.3 after bounded residency and before WAL recovery. The roadmap
  requires damaged bytes to localize before logical decode and before recovery
  can trust physical records.
- `runtime-integration-roadmap.md`
  keeps semantic durability above physical storage. S.3 may deny damaged bytes
  and classify physical trust, but it must not reinterpret transaction,
  branch, snapshot, subscription-support, or semantic artifact meaning.
- `worth_foundational_roadmap.md`
  protects shared boundary vocabulary without stealing local representation.
  S.3 should use Foundational diagnostic, provenance, receipt, profile, layout,
  and performance language at evidence boundaries while Store keeps physical
  integrity authority.
- `worth_proof_roadmap.md`
  protects proof-bearing progression law. S.3 must make intact, damaged,
  quarantined, and handoff-ready states mechanically distinct, sealed, and
  impossible to forge through raw bytes or copied report fields.

## Adversarial Constraint

S.3 must survive this hostile condition:

> A store with page, frame, manifest, index-page, WAL-frame, extent, and
> blob-chunk structures receives byte flips, torn writes, stale generation
> reuse, misplaced physical references, truncated frames, checksum collisions
> within the declared algorithm model, damaged derived structures, damaged
> authority structures, and interrupted scrub work while foreground readers and
> later recovery planners request protected physical byte access. Every damaged
> byte sequence must deny before logical decode, localize to the smallest
> honest physical boundary, preserve exact counters, and produce proof-bearing
> quarantine or handoff evidence without whole-store materialization or
> backend-private residue guessing.

If corrupted bytes can reach a semantic decoder, if an artifact digest is used
as a substitute for page/frame/chunk integrity, if a checksum result is treated
as authenticity, if quarantine can be synthesized from copied report fields, or
if scrub work can exceed the S.2 resident/allocation envelope, S.3 is not
closed.

## Product Decision Lock

- S.3 owns physical integrity admission, not semantic truth interpretation.
- S.3 consumes `S3PhysicalIntegrityReadiness`; it must not accept raw buffers,
  backend handles, or unprotected page views as entry authority.
- Checksums prove physical byte integrity inside a declared algorithm and
  placement boundary. They do not prove authenticity, tenant authorization, or
  semantic identity.
- Artifact digests and canonical bases may correlate integrity evidence, but
  they do not replace page, frame, manifest, WAL, index, or chunk checks.
- Scrub is an integrity-inspection workflow, not repair. It may produce repair
  inputs for S.10, but S.3 does not mutate authority as repair.
- S.4 recovery may consume S.3 integrity handoff evidence, but S.3 does not
  decide WAL replay, checkpoint source precedence, or crash completion.
- S.3 may inspect physical bytes, but every integrity classification,
  quarantine record, scrub report, diagnostic bundle, canonical basis, receipt,
  counter surface, and S.4 handoff must be typed aspect-native
  Store/Foundational evidence. JSON, serde-shaped objects, debug strings,
  display names, copied reports, raw buffers, and producer-private names may
  only appear as rejected hostile inputs or explicitly named compatibility debt.

## Integrity Exclusivity Laws

S.3 has several meaning boundaries that must be named because later storage
sequences will be tempted to reuse the nearest successful proof as if it proved
something broader:

- Integrity Meaning Separation Law: a checksum may admit or deny physical bytes
  for a declared physical scope. A canonical digest may correlate evidence
  identity. An authenticity proof may establish trusted origin or
  authorization. A recovery receipt may establish replay outcome. A repair
  receipt may establish mutation or reconstruction. None of these may
  substitute for another at S.3 boundaries.
- Integrity Entry Authority Law: after Phase 1, platform-grade S.3 lanes may
  consume only `S3PhysicalIntegrityReadiness`, `ProtectedPhysicalByteView`,
  `IntegrityInspectionLease`, or stronger admitted integrity entry forms. Raw
  buffers, file paths, backend handles, and copied readiness reports cannot
  enter integrity admission.
- Checksum Detection Model Law: each checksum declaration must state its
  detection model, collision posture, intended corruption class, and whether
  adversarial collision resistance is claimed. Unless later security or
  authenticity authority explicitly admits that claim, checksum success means
  admitted under the declared physical corruption-detection model, not
  impossible to forge.
- Checksum Scope Authority Law: no checksum result may be interpreted unless it
  was produced under a declared algorithm, physical scope, coverage map,
  version posture, and compatibility rule.
- Physical Scope Admission Law: bytes must belong to the claimed physical
  reference, generation, manifest scope, root posture, and checksum scope before
  family-specific validation can treat them as candidates for page, frame,
  manifest, WAL, index, or chunk integrity.
- Pre-Decode Integrity Law: logical decoders, semantic artifact readers,
  recovery planners, and repair planners may consume only integrity-checked
  physical forms or typed damage maps. Physical corruption must not surface as
  an ordinary semantic parse error.
- Logical Decode Non-Interference Law: when physical integrity admission fails,
  logical decode, semantic artifact construction, semantic index lookup, and
  domain-object materialization remain uninvoked unless a scenario explicitly
  tests post-integrity semantic parity over admitted bytes.
- Physical Locality Honesty Law: S.3 may localize damage only to the smallest
  boundary supported by physical evidence. When the surviving structure is
  insufficient for narrower localization, S.3 must emit an indeterminate or
  ambiguous-boundary damage class instead of claiming false precision.
- Quarantine Minting Law: quarantine records may be minted only from executed
  integrity findings, scrub findings, or injection-certified verifier paths.
  Logs, copied fields, fixture labels, raw paths, and expected error messages
  cannot mint quarantine.
- Foundational Boundary Role Law: Store-owned physical witnesses remain the
  authority for page, frame, manifest, WAL, chunk, quarantine, and handoff
  meaning. When S.3 exports those findings through Foundational boundary
  surfaces, each surface must carry an explicit role claim:
  `AuthoritativeCurrent` only for Store-admitted current physical authority
  artifacts, `DerivedProjection` only for derived reports with an intact
  Store-owned authority basis, `SupportOnly` only for diagnostics and operator
  explanations, `PlannedWork` only for unevaluated plans, and `ReceiptEvidence`
  only for completed receipts. No Foundational role claim may be fed back into
  Store as an integrity witness without the Store-owned physical proof type
  that minted it.
- Recovery Non-Claim Law: S.3 WAL, checkpoint, page, and manifest integrity
  reports may block or admit inputs for S.4, but they may not claim
  replayability, source precedence, crash completion, checkpoint validity, or
  acknowledged semantic truth.

## Planned Directory Skeleton

`workspaces/worth-store/crates/worth-store-physical-integrity/src/`

- `lib.rs`
  aggregates the public S.3 facade and re-exports only proof-bearing integrity
  boundary types.
- `readiness.rs`
  consumes `S3PhysicalIntegrityReadiness` and produces S.3 entry witnesses.
- `checksum_algorithm.rs`
  owns declared checksum algorithms, algorithm ids, compatibility posture, and
  algorithm mismatch denials.
- `checksum_scope.rs`
  owns page, frame, manifest, index-page, WAL-frame, extent, and chunk checksum
  coverage declarations.
- `physical_scope_admission.rs`
  owns physical reference, generation, manifest-scope, root-posture, and
  checksum-scope membership admission before family-specific validation.
- `integrity_admission.rs`
  owns pre-decode admission for protected byte views and rejects raw buffers.
- `page_integrity.rs`
  owns page-header and page-body validation.
- `frame_integrity.rs`
  owns physical frame validation, length law, and torn-frame detection.
- `manifest_integrity.rs`
  owns root and segment manifest validation.
- `index_page_integrity.rs`
  owns derived index-page validation and rebuildable-damage classification.
- `wal_frame_integrity.rs`
  owns WAL-frame integrity classification only; S.4 owns replay and recovery.
- `chunk_integrity.rs`
  owns extent and future blob-chunk checksum surfaces without claiming S.7
  chunk-tree lifecycle.
- `generation_integrity.rs`
  owns stale-generation and misplaced-reference physical denials.
- `scrub_plan.rs`
  owns bounded scrub request planning and S.2 envelope consumption.
- `scrub_execution.rs`
  owns execution over protected byte windows and exact scrub counters.
- `quarantine.rs`
  owns quarantine records, quarantine receipts, and locality reports.
- `damage_classification.rs`
  owns intact, rebuildable-derived, quarantined,
  indeterminate-physical-boundary, and unrecoverable-authority
  classifications.
- `s4_handoff.rs`
  owns the typed physical-integrity readiness payload for WAL/checkpoint work.
- `foundational_boundary_roles.rs`
  maps Store-owned S.3 physical witnesses, derived reports, plans,
  diagnostics, and receipts to Foundational boundary role claims without moving
  physical authority out of Store.
- `counters.rs`
  owns executed checksum, scrub, skipped-decode, quarantine, and localization
  counters.

`workspaces/worth-store/crates/worth-store-certification/src/`

- `physical_integrity_scenario_definitions.rs`
  defines S.3 scenario grammar over the Roadmap 2 harness.
- `physical_integrity_scenario_plans.rs`
  lowers S.3 definitions into required capabilities, drivers, observers,
  oracles, denial boundaries, and transcript identity.
- `physical_integrity_injection_drivers.rs`
  owns byte-flip, torn-frame, stale-generation, manifest, index, WAL, and
  chunk damage injection mechanics.
- `physical_integrity_observers.rs`
  registers checksum, skipped-decode, quarantine, scrub-window, and
  localization observers.
- `physical_integrity_oracles.rs`
  owns proof judgments for pre-decode denial, locality, authority/derived
  classification, bounded scrub, and S.4 handoff.
- `physical_integrity_evidence.rs`
  maps executed S.3 evidence into Foundational and Proof-facing evidence
  surfaces without moving Store authority.
- `physical_integrity_transcripts.rs`
  emits replay-comparable S.3 story, damage, counter, denial, and handoff
  transcripts.

`workspaces/worth-store/crates/worth-store-test-support/src/`

- `corruption_injectors.rs`
  provides deterministic byte and frame corruption mechanics only.
- `adversarial_storage_backend.rs`
  simulates torn reads/writes and stale physical placement for certification.
- `scrub_fixtures.rs`
  creates bounded scrub workloads over persisted stores larger than memory.
- `integrity_fixture_catalog.rs`
  catalogs intact authority, damaged authority, damaged derived, damaged
  manifest, damaged WAL, damaged index, and damaged chunk fixtures.

## Roadmap 2 Harness Test Plan

S.3 must extend the inherited Roadmap 2 physical scenario harness. It must not
create a second integrity harness, put oracle meaning in test support, or treat
logs and successful decode failures as proof.

The inherited proof pipeline remains:

`PhysicalScenarioDefinition` -> `PhysicalScenarioPlan` ->
`PhysicalScenarioExecution` -> `ObservedPhysicalTrace` ->
`PhysicalProofOracleVerdict` -> `PhysicalStoryTranscript`

S.3 must preserve S.1's plan-accountability rule. Every S.3 scenario plan must
declare, before execution:

- the physical law being proved or attacked
- required S.1 physical-format capabilities
- required S.2 protected-byte, resident, allocation, and scrub-envelope
  capabilities
- corruption injector class and exact injection locus
- expected storage boundary crossings
- expected physical footprint and fixture scale class
- expected checksum scope and algorithm posture
- expected physical reference, generation, manifest-scope, root-posture, and
  checksum-scope membership posture
- expected denied or admitted integrity boundary
- expected logical-decode gate behavior
- expected semantic decoder invocation count
- expected locality boundary for any damage
- expected authority-versus-derived damage classification
- expected quarantine or S.4 handoff posture
- expected Foundational boundary category and role claim, where evidence is
  exported outside Store-owned physical witnesses
- required drivers, observers, and proof oracles
- exact counter expectations
- artifact policy and transcript identity basis

Execution may run a lowered plan, but it may not choose a different injector,
observer set, checksum scope, locality target, damage classification, artifact
policy, or proof oracle after seeing the bytes. If the executor must re-decide
any of those, the plan was not honest enough to certify S.3.

S.3 scenario definitions must read like physical integrity stories. Required
definition vocabulary includes:

- `given_s3_integrity_readiness`
- `given_declared_checksum_algorithm`
- `given_declared_checksum_detection_model`
- `given_protected_page_view`
- `given_bounded_scrub_envelope`
- `when_page_byte_is_flipped`
- `when_frame_is_torn`
- `when_manifest_entry_points_to_wrong_generation`
- `when_intact_page_belongs_to_wrong_segment`
- `when_intact_wal_frame_belongs_to_wrong_checkpoint_scope`
- `when_semantically_parseable_corrupt_bytes_are_presented`
- `when_index_page_checksum_fails`
- `when_wal_frame_checksum_fails`
- `when_chunk_checksum_fails`
- `then_deny_before_logical_decode`
- `then_logical_decoder_invocation_count_is_zero`
- `then_deny_intact_bytes_with_wrong_scope`
- `then_localize_to_physical_boundary`
- `then_classify_indeterminate_boundary_when_locality_is_not_honest`
- `then_classify_authority_or_rebuildable_damage`
- `then_export_foundational_boundary_role_claim`
- `then_reject_foundational_projection_as_store_authority`
- `then_publish_s4_integrity_handoff`

Required lane families:

- `integrity_entry_lane`
  proves S.3 consumes typed S.2 readiness and protected byte access.
- `checksum_scope_lane`
  proves every checked physical family declares algorithm, scope, covered
  fields, excluded fields, and compatibility behavior.
- `physical_scope_admission_lane`
  proves individually intact bytes deny when their physical reference,
  generation, manifest membership, root posture, checkpoint adjacency, or
  checksum scope does not match the claimant.
- `pre_decode_denial_lane`
  proves damaged physical bytes deny before semantic decoders can run.
- `semantic_decoder_non_interference_lane`
  proves failed physical admission leaves logical decoders, semantic artifact
  construction, semantic indexes, and domain-object materialization uninvoked.
- `page_frame_damage_lane`
  proves page and frame damage localizes to page, frame, slot, and extent
  boundaries with exact counters.
- `manifest_index_damage_lane`
  proves manifest and derived-index damage are distinguished and localized.
- `wal_frame_integrity_lane`
  proves WAL frame damage can be classified without claiming S.4 replay.
- `chunk_integrity_lane`
  proves extent and future blob-chunk damage localizes without claiming S.7
  chunk lifecycle.
- `stale_generation_lane`
  proves generation mismatches deny as physical identity failures, not decode
  failures.
- `intact_wrong_scope_lane`
  repeats the intact-but-wrong adversary across pages, frames, manifests, WAL
  frames, chunks, and derived index pages.
- `scrub_bounded_execution_lane`
  proves online and offline scrub consume S.2 memory and allocation envelopes.
- `quarantine_lane`
  proves quarantines are sealed physical records with locality and damage
  classification.
- `foundational_proof_evidence_lane`
  proves evidence materializes from executed Store findings into shared
  vocabulary without replacing Store integrity authority.
- `foundational_boundary_role_lane`
  proves S.3 evidence exports use Foundational roles accurately and reject
  projection, support, planned-work, or receipt surfaces as Store physical
  authority.
- `s4_handoff_lane`
  proves recovery receives integrity-vetted physical records and typed damage
  maps, not raw bytes.

Required drivers extend the S.1/S.2 driver set:

- existing `PlatformBackendDriver`, `PersistedFileDeviceDriver`,
  `AdversarialByteDeviceDriver`, `CrashInterposerDriver`,
  `MemoryPressureDriver`, `AllocationSentinelDriver`, and
  `BackgroundMaintenanceDriver`
- S.3 `ByteFlipInjectionDriver`, `TornFrameInjectionDriver`,
  `StaleGenerationInjectionDriver`, `ManifestDamageInjectionDriver`,
  `IndexPageDamageInjectionDriver`, `WalFrameDamageInjectionDriver`,
  `ChunkDamageInjectionDriver`, and `ScrubWindowDriver`

Required observers:

- existing `CounterObserver`, `StorageBoundaryObserver`,
  `MaterializationObserver`, `RuntimeLayoutObserver`,
  `DenialBoundaryObserver`, `EvidenceExportObserver`,
  `ResidentSetObserver`, and `AllocationEnvelopeObserver`
- S.3 `ChecksumObserver`, `PreDecodeAdmissionObserver`,
  `PhysicalScopeAdmissionObserver`, `SemanticDecoderInvocationObserver`,
  `PhysicalLocalityObserver`, `ScrubProgressObserver`, `QuarantineObserver`,
  `DamageClassificationObserver`, `FoundationalBoundaryRoleObserver`, and
  `S4HandoffObserver`

Required proof oracles:

- `RawBytesCannotEnterIntegrityOracle`
- `ChecksumScopeCoverageOracle`
- `ChecksumDetectionModelHonestyOracle`
- `PhysicalScopeAdmissionPrecedesFamilyValidationOracle`
- `DamagedBytesDenyBeforeLogicalDecodeOracle`
- `LogicalDecodeNonInterferenceOracle`
- `PhysicalDamageLocalizesToSmallestBoundaryOracle`
- `IndeterminateBoundaryPreventsFalseLocalityOracle`
- `DigestDoesNotSubstituteForChecksumOracle`
- `ChecksumDoesNotSubstituteForAuthenticityOracle`
- `ScrubRespectsBoundedResidencyOracle`
- `DerivedDamageCannotOutrankAuthorityOracle`
- `QuarantineRecordsCannotBeForgedOracle`
- `S4HandoffContainsIntegrityVettedRecordsOracle`
- `FoundationalBoundaryRolesDoNotReplaceStoreAuthorityOracle`
- `ProjectionSupportPlanAndReceiptCannotReenterIntegrityOracle`
- `TestSupportCannotOwnIntegrityMeaningOracle`

Required transcripts and traces:

- `physical_integrity_story_transcript`
- `checksum_scope_trace`
- `checksum_detection_model_trace`
- `physical_scope_admission_trace`
- `damage_injection_trace`
- `pre_decode_denial_trace`
- `semantic_decoder_non_interference_trace`
- `scrub_window_counter_trace`
- `quarantine_locality_trace`
- `damage_classification_trace`
- `s4_integrity_handoff_trace`
- `foundational_integrity_receipt_trace`
- `foundational_boundary_role_trace`
- `projection_authority_denial_trace`

Required fixture classes:

- `intact_minimal_store_fixture`
  proves the entry, checksum, scrub, and handoff lanes work on the smallest
  honest persisted physical store.
- `intact_large_store_fixture`
  proves integrity checks and scrub run on a persisted store larger than the
  configured resident-memory budget.
- `page_header_damage_fixture`
  injects damage into magic, version, kind, length, generation, checksum slot,
  and reserved-field posture.
- `page_body_damage_fixture`
  injects damage into page payload regions while preserving header readability
  where the story requires that distinction.
- `frame_header_damage_fixture`
  injects kind, length, generation, and checksum-scope corruption.
- `torn_frame_fixture`
  injects truncated, overlong, partially overwritten, and boundary-crossing
  frame bodies.
- `slot_directory_damage_fixture`
  injects slot offset, slot length, moved-slot, and stale slot generation
  damage.
- `manifest_damage_fixture`
  injects root, segment, extent, allocation-map, free-space-map, and misplaced
  physical-reference damage.
- `derived_index_damage_fixture`
  injects rebuildable and non-rebuildable index-page damage with explicit
  authority basis posture.
- `wal_frame_damage_fixture`
  injects WAL-frame checksum, length, and checkpoint-adjacent corruption while
  proving no S.4 replay claim is made.
- `chunk_damage_fixture`
  injects extent and future blob-chunk header, payload, and boundary damage
  through bounded streaming windows.
- `stale_generation_fixture`
  reuses physical slots, extents, and root references with new generations and
  proves stale references deny even when bytes locally checksum.
- `checksum_algorithm_mismatch_fixture`
  proves unknown, deprecated, wrong-scope, wrong-version, and digest-substitute
  algorithms deny before inspection.
- `intact_wrong_scope_fixture`
  presents locally intact pages, frames, WAL frames, chunk-like regions, and
  derived index pages under the wrong segment, generation, manifest membership,
  root posture, checkpoint scope, or extent claim.
- `semantic_decoder_poisoned_input_fixture`
  presents physically damaged bytes that remain syntactically plausible enough
  for a semantic decoder to parse if the pre-decode gate is missing.
- `ambiguous_boundary_damage_fixture`
  damages headers, lengths, roots, or torn-write boundaries so the smallest
  honest locality is broader than the apparent byte position.
- `over_budget_scrub_fixture`
  proves scrub denies or defers before exceeding S.2 resident, pin,
  allocation, or streaming-window limits.
- `synthetic_integrity_shortcut_fixture`
  attempts to close S.3 with logs, expected error text, fixture labels,
  in-memory-only byte arrays, copied report fields, and test-support-owned
  oracle meaning.

Production-grade S.3 tests must include five layers:

- Contract and compile-fail tests for sealed checksum, quarantine, entry, and
  handoff construction.
- Harness certification tests where S.3 scenario definitions lower into plans,
  run through drivers and observers, and receive oracle verdicts.
- Persisted large-store corruption tests using real physical fixtures,
  deterministic corruption injection, bounded scrub windows, exact counter
  assertions, and materialization observers proving semantic decode did not
  run after physical denial.
- Semantic-decoder poisoned-input tests where physically damaged but
  syntactically plausible bytes are denied before semantic decoders, semantic
  index lookups, or domain-object constructors run.
- Runtime/verifier parity tests where live runtime inspection and offline
  verifier inspection independently observe the same persisted bytes and
  converge on the same integrity finding, locality, counters, and transcript
  identity for the shared evidence basis.
- Synthetic-rejection tests where fake harnesses, test-support-owned proof,
  logs-as-proof, fixture-label proof, same-run self-comparison, in-memory-only
  buffers, and copied evidence fields fail at plan admission or oracle
  judgment.

Unit tests that only compute a checksum, detect one flipped byte, or produce an
error are insufficient. S.3 tests must prove the architectural properties:
entry authority, lowered-plan honesty, pre-decode denial, physical locality,
authority/derived classification, bounded scrub execution, sealed quarantine,
evidence source honesty, S.4 handoff correctness, and synthetic shortcut
rejection.

Minimum S.3 certification matrix, grouped by damage family:

| Damage family | Suite | Harness lane | Required fixture classes | Required proof outputs |
| --- | --- | --- | --- | --- |
| Entry and checksum scope | `checksum_detection_model_honesty_suite` | `checksum_scope_lane` | `checksum_algorithm_mismatch_fixture`, `synthetic_integrity_shortcut_fixture` | `checksum_detection_model_trace`, `scenario_denial_trace`, `foundational_integrity_receipt_trace` |
| Entry and checksum scope | `integrity_entry_authority_suite` | `integrity_entry_lane` | `intact_minimal_store_fixture`, `synthetic_integrity_shortcut_fixture` | `physical_integrity_story_transcript`, `pre_decode_denial_trace`, `shortcut_rejection_trace` |
| Entry and checksum scope | `checksum_scope_coverage_suite` | `checksum_scope_lane` | `checksum_algorithm_mismatch_fixture`, `page_header_damage_fixture` | `checksum_scope_trace`, `scenario_denial_trace`, `foundational_integrity_receipt_trace` |
| Physical scope admission | `physical_scope_admission_suite` | `physical_scope_admission_lane` | `intact_wrong_scope_fixture`, `stale_generation_fixture` | `physical_scope_admission_trace`, `integrity_basis_trace`, `scenario_denial_trace` |
| Pre-decode gate | `pre_decode_physical_denial_suite` | `pre_decode_denial_lane` | `page_body_damage_fixture`, `frame_header_damage_fixture`, `slot_directory_damage_fixture` | `pre_decode_denial_trace`, `scenario_counter_trace`, `physical_proof_oracle_verdict` |
| Pre-decode gate | `semantic_decoder_non_interference_suite` | `semantic_decoder_non_interference_lane` | `semantic_decoder_poisoned_input_fixture`, `page_body_damage_fixture` | `semantic_decoder_non_interference_trace`, `skipped_logical_decode_counter`, `compile_fail_boundary_report` |
| Page, frame, slot, and extent damage | `page_frame_extent_damage_suite` | `page_frame_damage_lane` | `page_header_damage_fixture`, `page_body_damage_fixture`, `frame_header_damage_fixture`, `torn_frame_fixture`, `slot_directory_damage_fixture` | `damage_injection_trace`, `quarantine_locality_trace`, `skipped_logical_decode_counter` |
| Page, frame, slot, and extent damage | `ambiguous_boundary_locality_suite` | `page_frame_damage_lane` | `ambiguous_boundary_damage_fixture`, `torn_frame_fixture`, `manifest_damage_fixture` | `quarantine_locality_trace`, `damage_classification_trace`, `physical_proof_oracle_verdict` |
| Manifest, root, and allocation damage | `manifest_root_allocation_integrity_suite` | `manifest_index_damage_lane` | `manifest_damage_fixture`, `stale_generation_fixture` | `damage_classification_trace`, `scenario_denial_trace`, `physical_story_transcript` |
| Derived index damage | `derived_index_damage_classification_suite` | `manifest_index_damage_lane` | `derived_index_damage_fixture`, `manifest_damage_fixture` | `damage_classification_trace`, `quarantine_locality_trace`, `foundational_integrity_receipt_trace` |
| WAL and checkpoint-adjacent damage | `wal_frame_integrity_without_recovery_suite` | `wal_frame_integrity_lane` | `wal_frame_damage_fixture`, `torn_frame_fixture` | `damage_injection_trace`, `s4_integrity_handoff_trace`, `recovery_claim_absence_report` |
| Chunk and large-payload damage | `chunk_integrity_without_blob_lifecycle_suite` | `chunk_integrity_lane` | `chunk_damage_fixture`, `intact_large_store_fixture` | `scrub_window_counter_trace`, `damage_classification_trace`, `blob_lifecycle_claim_absence_report` |
| Stale generation and scope mismatch | `stale_generation_and_scope_mismatch_suite` | `stale_generation_lane` | `stale_generation_fixture`, `checksum_algorithm_mismatch_fixture` | `scenario_denial_trace`, `integrity_basis_trace`, `physical_story_transcript` |
| Stale generation and scope mismatch | `intact_wrong_scope_cross_family_suite` | `intact_wrong_scope_lane` | `intact_wrong_scope_fixture`, `derived_index_damage_fixture`, `wal_frame_damage_fixture`, `chunk_damage_fixture` | `physical_scope_admission_trace`, `integrity_basis_trace`, `physical_story_transcript` |
| Scrub and quarantine | `bounded_online_offline_scrub_suite` | `scrub_bounded_execution_lane` | `intact_large_store_fixture`, `over_budget_scrub_fixture` | `scrub_window_counter_trace`, `runtime_verifier_parity_trace`, `resource_envelope_report` |
| Scrub and quarantine | `quarantine_sealing_suite` | `quarantine_lane` | every damaged fixture class, `synthetic_integrity_shortcut_fixture` | `quarantine_locality_trace`, `shortcut_rejection_trace`, `physical_proof_oracle_verdict` |
| Evidence, proof, and handoff | `foundational_integrity_evidence_suite` | `foundational_proof_evidence_lane` | `intact_minimal_store_fixture`, `derived_index_damage_fixture`, `manifest_damage_fixture` | `foundational_integrity_receipt_trace`, `foundational_diagnostic_bundle`, `foundational_counter_backed_performance_receipt` |
| Evidence, proof, and handoff | `foundational_boundary_role_mapping_suite` | `foundational_boundary_role_lane` | `intact_minimal_store_fixture`, `derived_index_damage_fixture`, `synthetic_integrity_shortcut_fixture` | `foundational_boundary_role_trace`, `projection_authority_denial_trace`, `physical_proof_oracle_verdict` |
| Evidence, proof, and handoff | `proof_progression_integrity_state_suite` | `foundational_proof_evidence_lane` | `page_header_damage_fixture`, `synthetic_integrity_shortcut_fixture` | `proof_progression_trace`, `compile_fail_boundary_report`, `scenario_denial_trace` |
| Evidence, proof, and handoff | `s4_integrity_handoff_suite` | `s4_handoff_lane` | `wal_frame_damage_fixture`, `manifest_damage_fixture`, `intact_minimal_store_fixture` | `s4_integrity_handoff_trace`, `recovery_blocking_integrity_report`, `physical_story_transcript` |
| Synthetic rejection | `synthetic_integrity_test_rejection_suite` | every S.3 lane family | `synthetic_integrity_shortcut_fixture` | `shortcut_rejection_trace`, `test_support_authority_denial_report`, `physical_proof_oracle_verdict` |

Every matrix row must have at least one adversarial positive case and one
adversarial negative case. Every negative case must name the boundary where it
failed: plan admission, entry admission, checksum-scope admission, pre-decode
gate, locality oracle, quarantine sealing, evidence materialization, or S.4
handoff admission.

Production-grade S.3 closeout requires a `PhysicalIntegrityCertificationBundle`
that contains:

- all scenario definitions
- all lowered scenario plans
- execution reports
- observed traces
- proof-oracle verdicts
- replay-comparable story transcripts
- exact counter traces
- checksum detection model traces
- physical scope admission traces
- denial traces
- semantic decoder non-interference traces
- ambiguous-boundary locality traces
- runtime/verifier parity traces
- shortcut-rejection traces
- fixture adversary reports
- resource envelope reports
- hardware/backend assumption reports
- Foundational evidence bundles
- S.4 handoff evidence

The bundle is not allowed to contain raw logs, screenshots, loose artifacts, or
free-form narrative as proof. Narrative may explain the bundle, but only the
machine-checkable rows close S.3.

## Phases

### Phase 1: Consume S.3 Readiness And Seal Integrity Entry Authority

Phase 1 closes the S.2-to-S.3 boundary. It admits only typed readiness,
protected byte views, bounded scrub envelopes, and S.1/S.2 authority recaps.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-buffer-pool`
- `worth-store-readiness`
- `worth-store-certification`
- `worth-proof`

**Relevant APIs**

- `S3PhysicalIntegrityReadiness`
- `ProtectedPhysicalByteView`
- `IntegrityInspectionLease`
- `VerifierResidentEnvelope`
- `ScrubPlanningAllocationEnvelope`
- `IntegrityEntryWitness`

**Warnings**

- Do not accept raw byte slices, backend handles, file paths, or unprotected
  page views as S.3 entry authority.
- Do not let `S3PhysicalIntegrityReadiness` be a symbolic token. It must carry
  concrete protected access, counters, envelope limits, and authority recaps.
- Do not let integrity inspection extend a lease or pin lifetime beyond the
  S.2 proof that admitted it.

**Test requirements**

- Adversarial parity: two independently produced S.2 closeout handoffs with
  equivalent protected-byte evidence lower into the same S.3 entry witness and
  scrub envelope limits.
- Adversarial denial: raw buffers, backend-private handles, copied readiness
  reports, and expired protected views cannot enter S.3 integrity admission.
- Compile-fail: external callers cannot synthesize `IntegrityEntryWitness` or
  forge stronger protected-byte access from public fields.

**Engineering decisions**

- S.3 begins with entry authority because every later physical check depends
  on bounded, protected byte access.
- `worth-proof` progression vocabulary should be used where S.3 entry,
  admitted inspection, damaged, quarantined, and handoff-ready states repeat
  shared proof-bearing patterns.
- Store remains the authority for the physical meaning of the bytes.

**Open questions**

- None.

### Phase 2: Declare Checksum Algorithms, Scope, And Compatibility Law

Phase 2 defines which checksum algorithms Store may claim, where each checksum
applies, what bytes are covered, and how versioned algorithm changes deny or
readmit.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-physical-format`
- `worth-store-certification`
- `worth-foundational`

**Relevant APIs**

- `ChecksumAlgorithmId`
- `ChecksumScopeDeclaration`
- `ChecksumCoverageMap`
- `ChecksumDetectionModel`
- `ChecksumCompatibilityPosture`
- `ChecksumAlgorithmMismatchDenial`
- `PhysicalFormatVersion`

**Warnings**

- Do not use an artifact digest as a page, frame, chunk, or WAL checksum.
- Do not treat checksum success as authenticity or authorization.
- Do not let algorithm selection be backend-private folklore.
- Do not make checksum coverage depend on serializer field order.
- Do not claim collision resistance unless the declared algorithm and later
  authenticity posture actually support that claim.

**Test requirements**

- Adversarial equivalence: independently constructed checksum declarations for
  the same physical format version produce the same coverage basis and
  Foundational evidence identity.
- Adversarial denial: unknown algorithms, mismatched algorithm ids, missing
  coverage fields, digest-as-checksum substitutions, and checksum-as-authenticity
  claims deny before inspection.
- Detection-model proof: each algorithm declaration names corruption class,
  collision posture, covered header fields, excluded header fields, checksum
  field handling, mutable publication fields, reserved fields, generation
  fields, length fields, body or payload region, padding bytes, compatibility
  fields, unknown-field posture, and whether the checksum covers serialized
  bytes or canonicalized fields.
- Compatibility proof: a format-version change that alters checksum coverage
  requires explicit readmission or denial rather than silent reuse.

**Engineering decisions**

- S.3 should start with CRC32c or a stronger declared algorithm, but the spec
  requires algorithm identity and compatibility law rather than hard-coding
  folklore into callers.
- Checksum scope is a Store physical-format contract.
- `ChecksumCoverageMap` must be concrete enough that every physical family can
  answer which bytes were covered, excluded, preserved, skipped, or denied.
- Foundational may describe the evidence basis; it does not choose the
  checksum algorithm.

**Open questions**

- None.

### Phase 3: Enforce Pre-Decode Physical Admission

Phase 3 makes physical integrity admission mandatory before any logical
decoder, semantic artifact reader, or higher-level recovery planner can consume
physical bytes.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-physical-format`
- `worth-store-certification`
- `worth-store-readiness`

**Relevant APIs**

- `PhysicalIntegrityAdmission`
- `IntegrityCheckedPage`
- `IntegrityCheckedFrame`
- `IntegrityCheckedManifest`
- `IntegrityCheckedWalFrame`
- `LogicalDecodeGate`
- `SkippedLogicalDecodeCounter`
- `SemanticDecoderInvocationCounter`

**Warnings**

- Do not let semantic decode catch physical corruption as a parse error.
- Do not build logical artifact identity before physical admission passes.
- Do not consult semantic indexes or construct domain objects after physical
  admission fails.
- Do not collapse unsupported checksum posture and damaged bytes into the same
  denial.
- Pre-Decode Law: after this phase, ordinary Store logical decode APIs may
  consume only integrity-checked physical forms or stronger handoff forms.

**Test requirements**

- Adversarial replay: the same intact physical bytes admitted twice produce the
  same integrity-checked physical form and decode-gate identity.
- Adversarial denial: byte-flipped pages, truncated frames, checksum mismatch,
  unsupported checksum algorithm, and stale generation all skip logical decode
  and emit exact skipped-decode counters.
- Poisoned-input denial: physically damaged bytes that remain syntactically
  plausible enough for semantic parse are denied before any semantic decoder,
  semantic index lookup, or domain-object constructor is invoked.
- Compile-fail: semantic decoder APIs cannot accept raw physical byte views or
  only checksum-planned forms.

**Engineering decisions**

- Logical decode is downstream of integrity admission.
- Pre-decode denial must preserve typed reason and physical locality so
  operators and later recovery do not receive ambiguous parse failures.
- Skipped logical decodes are success evidence for S.3 when corruption is
  present.
- Zero semantic decoder invocations after failed physical admission are an
  audited negative capability, not an incidental side effect.

**Open questions**

- None.

### Phase 4: Admit Physical Scope Before Family-Specific Validation

Phase 4 proves that bytes belong to the physical identity that claims them
before any page, frame, manifest, WAL, index, or chunk validator can interpret
local structure.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-physical-format`
- `worth-store-readiness`
- `worth-store-certification`

**Relevant APIs**

- `PhysicalScopeAdmission`
- `PhysicalReferenceScope`
- `ManifestMembershipProof`
- `RootManifestIntegrityPosture`
- `GenerationIntegrityReport`
- `ChecksumScopeMismatchDenial`
- `IntactWrongScopeDenial`

**Warnings**

- Do not call bytes valid merely because their local checksum passes.
- Do not let family-specific validators decide whether bytes belong to the
  claimed segment, generation, root, checkpoint, extent, or checksum scope.
- Do not substitute S.2 resident-frame generation for S.1 durable physical
  generation.
- Do not let copied physical references rebind to current bytes without
  manifest, root-posture, and generation proof.

**Test requirements**

- Adversarial replay: slot reuse with a new durable physical generation changes
  integrity basis and rejects stale references deterministically before page or
  frame validation.
- Adversarial denial: locally checksummed bytes from the wrong page, segment,
  extent, manifest scope, checkpoint adjacency, root posture, or generation
  deny as misplaced or stale physical identity.
- Cross-family proof: page, frame, WAL, manifest, chunk-like, and derived-index
  validators all consume `PhysicalScopeAdmission` rather than re-deciding scope
  membership internally.

**Engineering decisions**

- Integrity means bytes are intact for the physical identity that claimed them,
  not merely locally checksummed.
- Scope mismatch is an integrity failure because correct bytes can represent
  the wrong physical record.
- Basis digests may summarize the proof basis, but authority remains in the
  checked physical references, manifest membership, root posture, and
  generation facts.

**Open questions**

- None.

### Phase 5: Validate Page, Frame, Extent, And Slot Integrity

Phase 5 validates the core S.1 physical containers: page headers, page bodies,
frame headers, frame payloads, extent boundaries, and slot-directory records.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-physical-format`
- `worth-store-buffer-pool`
- `worth-store-certification`

**Relevant APIs**

- `PageIntegrityReport`
- `FrameIntegrityReport`
- `ExtentIntegrityReport`
- `SlotDirectoryIntegrityReport`
- `SlotStateIntegrityReport`
- `AmbiguousBoundaryDamage`
- `TornFrameDenial`
- `PhysicalBoundaryLocalization`

**Warnings**

- Do not let a page checksum failure become a generic record decode failure.
- Do not accept a frame whose length, kind, generation, or checksum contradicts
  its header witness.
- Do not scan beyond the admitted protected page/extent window to improve
  localization precision.
- Do not call a page intact just because one record inside it decodes.
- Do not over-localize when damaged headers, torn lengths, or destroyed slot
  structure make narrower localization dishonest.
- Slot State Integrity Law: occupied slots require admitted in-page frame
  integrity; deleted, free, and reserved slots require valid structural
  encoding without payload exposure; moved slots require an admitted bounded
  forwarding target or deny as integrity failure; multi-hop moved-slot chains
  deny unless S.1 explicitly certified them.

**Test requirements**

- Adversarial parity: independently read protected views of the same intact
  page and extent produce the same page, frame, slot, and extent integrity
  reports with identical counters.
- Adversarial localization: injected byte flips in page header, page body,
  frame header, frame body, length field, and slot directory localize to the
  smallest honest page/frame/slot/extent boundary and deny before decode.
- Ambiguous-boundary proof: when damage destroys the structure needed for
  narrower locality, S.3 emits `AmbiguousBoundaryDamage` instead of claiming
  false slot, frame, page, or extent precision.
- Torn-frame proof: truncated or overlong frame bodies deny as torn or
  malformed physical frames before any record view can be constructed.

**Engineering decisions**

- Page and frame reports are physical evidence, not semantic artifact reports.
- S.2 protected-byte leases remain the access mechanism for inspection.
- Localization precision must be honest; the system may not pretend to know a
  smaller damaged region than the physical evidence supports.

**Open questions**

- None.

### Phase 6: Validate Manifest, Root, And Allocation-Map Integrity

Phase 6 validates physical root manifests, segment manifests, allocation maps,
free-space maps, and physical-reference tables before they can guide reads,
scrub, or recovery planning.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-physical-format`
- `worth-store-certification`
- `worth-store-readiness`

**Relevant APIs**

- `ManifestIntegrityReport`
- `RootManifestIntegrityReport`
- `RootManifestIntegrityPosture`
- `SegmentManifestIntegrityReport`
- `AllocationMapIntegrityReport`
- `ManifestReferenceMismatchDenial`
- `ManifestGenerationMismatchDenial`

**Warnings**

- Do not trust a manifest pointer because the pointed-to page is individually
  intact.
- Do not let a stale physical reference satisfy a current manifest entry.
- Do not infer missing manifest truth from backend directory residue.
- Do not promote a derived index or scan result over a damaged authoritative
  root manifest.
- Do not choose recovery source precedence when multiple plausible roots exist.

**Test requirements**

- Adversarial convergence: two independent manifest walks over the same intact
  root produce the same manifest integrity report, allocation-map counters, and
  physical-reference basis.
- Adversarial denial: stale manifest generation, wrong segment id, mismatched
  extent id, damaged allocation map, missing root page, and backend residue
  fallback all deny as typed manifest integrity failures.
- Root-posture proof: single admitted root, damaged root, torn root pointer,
  multiple valid roots, root generation mismatch, residue root rejection, and
  recovery-blocking root damage produce distinct `RootManifestIntegrityPosture`
  values without deciding S.4 recovery precedence.
- Source-precedence proof: intact derived structures cannot override a damaged
  or mismatched authoritative manifest entry.

**Engineering decisions**

- Manifest integrity is prerequisite physical trust for later recovery source
  precedence, but S.4 owns the actual recovery decision graph.
- When root evidence is ambiguous, S.3 emits typed root posture for S.4 rather
  than selecting the winning root.
- Manifest denials must carry enough physical locality for S.10 repair planning
  without performing repair.
- Allocation-map integrity remains physical placement law, not semantic
  retention or branch truth.

**Open questions**

- None.

### Phase 7: Distinguish Derived Index Damage From Authority Damage

Phase 7 validates derived index pages and classifies damage that can be rebuilt
from intact authority separately from damage to authoritative physical records.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-certification`
- `worth-store-readiness`
- `worth-store-physical-format`

**Relevant APIs**

- `IndexPageIntegrityReport`
- `DerivedDamageClassification`
- `RebuildableDerivedDamage`
- `RebuildableDerivedDamagePrerequisites`
- `IndeterminatePhysicalDamage`
- `UnrecoverableAuthorityDamage`
- `AuthorityDamageBoundary`
- `DerivedRebuildInput`

**Warnings**

- Do not let a derived index become authority because it is easier to read than
  the damaged source.
- Do not call damage rebuildable unless the authority basis is intact and
  explicitly identified.
- Do not rebuild the index in S.3. Produce the repair/rebuild input only.
- Do not classify all index corruption as harmless; index pages may be required
  for bounded access until a later rebuild path is admitted.
- Rebuildable Derived Damage Prerequisites: S.3 must identify the damaged
  derived physical boundary, intact authoritative physical basis, derivation
  family, generation or manifest scope linking derived to authority, bounded
  rebuild input shape, and proof that no semantic truth decision depends on the
  damaged derived structure before rebuild.

**Test requirements**

- Adversarial parity: the same damaged derived index plus intact authority
  basis produces the same rebuildable-damage classification across independent
  inspection paths.
- Adversarial denial: damaged authority, missing authority basis, stale index
  generation, and copied rebuild reports cannot classify damage as rebuildable.
- Indeterminate proof: if any rebuildability prerequisite is missing, the
  damage class becomes quarantined or indeterminate rather than rebuildable.
- Boundary proof: derived damage reports cannot be passed to APIs requiring
  intact authority or S.4 recovery-ready records.

**Engineering decisions**

- Authority-versus-derived classification must be explicit before S.4 and S.10
  consume integrity findings.
- Rebuildability is a proof-bearing input for later work, not an action S.3
  performs.
- Derived structures may be quarantined or marked rebuildable; neither outcome
  changes semantic truth.

**Open questions**

- None.

### Phase 8: Validate WAL-Frame And Checkpoint-Adjacent Integrity Without Claiming Recovery

Phase 8 validates WAL-frame and checkpoint-adjacent physical bytes only as
integrity-bearing records. It does not replay WAL, choose recovery precedence,
or decide acknowledged truth.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-recovery-physics`
- `worth-store-certification`
- `worth-store-physical-format`

**Relevant APIs**

- `WalFrameIntegrityReport`
- `CheckpointRecordIntegrityReport`
- `WalTailIntegrityPosture`
- `WalFrameDamageDenial`
- `CheckpointAdjacentDamageDenial`
- `RecoveryPhysicsIntegrityInput`

**Warnings**

- Do not call a WAL frame replayable because its checksum passes.
- Do not decide pageLSN, redo, checkpoint source precedence, or acknowledgment
  truth in S.3.
- Do not let S.4 consume raw WAL bytes when S.3 can provide integrity-checked
  records and typed damage maps.
- Do not treat truncated tails as successful recovery behavior; S.3 only
  classifies physical integrity.
- Do not collapse intact WAL frames, torn WAL frames, unsupported integrity,
  unknown integrity, checkpoint-adjacent damage, and
  recovery-precedence-required posture into one result.

**Test requirements**

- Adversarial equivalence: intact WAL-frame bytes admitted through independent
  protected reads produce the same integrity report and S.4 input identity.
- Adversarial denial: checksum failure, torn WAL frame, mismatched length,
  unsupported algorithm, and checkpoint-adjacent corruption deny before replay
  and produce no recovery conclusion.
- WAL-tail proof: intact tail, torn tail, unsupported tail integrity, unknown
  tail integrity, checkpoint-adjacent damage, and recovery-precedence-required
  posture remain distinct without S.3 truncating or replaying the tail.
- Boundary proof: S.3 WAL integrity reports cannot be used as S.4 replay
  receipts or checkpoint-validity decisions.

**Engineering decisions**

- S.3 gives S.4 integrity-vetted bytes, typed damage maps, and counters.
- S.4 remains the owner of LSN, pageLSN, checkpoint, redo, and crash recovery
  physics.
- WAL integrity counters must be exact even when replay is deliberately not
  attempted.

**Open questions**

- None.

### Phase 9: Validate Extent And Future Blob-Chunk Integrity Without Claiming Blob Lifecycle

Phase 9 validates large extent and future blob-chunk byte integrity while
reserving content-addressed chunk-tree lifecycle, resumable upload, dedupe, and
reachability for S.7.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-blob-chunks`
- `worth-store-buffer-pool`
- `worth-store-certification`

**Relevant APIs**

- `ChunkIntegrityReport`
- `ExtentChunkIntegrityReport`
- `PhysicalChunkBoundary`
- `ChunkLikeIntegrityScope`
- `FutureBlobChunkCompatibilityPosture`
- `StreamingChunkInspectionWindow`
- `ChunkChecksumMismatchDenial`
- `ChunkDamageLocalization`

**Warnings**

- Do not claim S.7 native blob storage from S.3 chunk checks.
- Do not require whole-object residency to inspect chunk integrity.
- Do not treat content digest success as proof that each physical chunk is
  intact at its storage location.
- Do not let a chunk-local checksum failure become a whole-blob semantic
  failure without localization.
- Do not freeze S.7 chunk-tree, dedupe, upload, or reachability architecture
  by naming S.3 integrity primitives as final blob lifecycle types.

**Test requirements**

- Adversarial parity: the same multi-extent physical chunk inspected through
  independent bounded streaming windows produces the same chunk integrity
  report and window counters.
- Adversarial localization: byte flips at chunk header, chunk payload, chunk
  boundary, and extent boundary localize without reading the whole object.
- Boundary proof: S.3 chunk integrity reports cannot claim dedupe, reachability,
  resumability, or blob-retention correctness.

**Engineering decisions**

- S.3 validates physical chunk bytes and locality only.
- S.3 names chunk-compatible physical scopes defensively so S.7 can consume
  them without inheriting a premature blob model.
- S.2 streaming windows are the memory boundary for large chunk inspection.
- S.7 will consume these integrity primitives when it builds native blob
  lifecycle semantics.

**Open questions**

- None.

### Phase 10: Certify Cross-Family Intact-Wrong-Scope Rejection

Phase 10 repeats the Phase 4 scope law across all S.3 physical families so
locally intact bytes cannot sneak through under the wrong page, segment,
extent, manifest, checkpoint, chunk, derived-index, root, generation, or
checksum-scope claim.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-physical-format`
- `worth-store-readiness`
- `worth-store-certification`

**Relevant APIs**

- `GenerationIntegrityReport`
- `StalePhysicalGenerationDenial`
- `MisplacedReferenceDenial`
- `ChecksumScopeMismatchDenial`
- `PhysicalReferenceScope`
- `IntactWrongScopeDenial`
- `CrossFamilyScopeMismatchReport`
- `IntegrityBasisDigest`

**Warnings**

- Do not call bytes valid merely because their local checksum passes.
- Do not substitute S.2 resident-frame generation for S.1 durable physical
  generation.
- Do not let copied physical references rebind to current bytes without
  manifest, root-posture, checkpoint, extent, and generation proof.
- Do not collapse stale generation and checksum failure into one denial.

**Test requirements**

- Adversarial replay: repeated cross-family wrong-scope scenarios reject at the
  same scope-admission boundary regardless of which family validator would have
  run next.
- Adversarial denial: individually checksummed bytes from the wrong page,
  segment, extent, manifest scope, checkpoint scope, chunk scope, root posture,
  derived-index authority basis, or generation deny as misplaced or stale
  physical identity.
- Generation separation proof: S.1 durable physical generation and S.2
  resident-frame generation cannot substitute for each other in S.3 integrity
  admission.

**Engineering decisions**

- Integrity means bytes are intact for the physical identity that claimed them,
  not merely locally checksummed.
- Scope mismatch is an integrity failure because it can cause correct bytes to
  represent the wrong physical record.
- Basis digests may summarize the proof basis, but authority remains in the
  checked physical references, root posture, manifest membership, checkpoint
  adjacency, chunk scope, authority basis, and generation facts.

**Open questions**

- None.

### Phase 11: Build Bounded Online And Offline Scrub Execution

Phase 11 builds scrub as a bounded inspection workflow over protected byte
windows. It covers online and offline inspection surfaces without performing
repair, recovery, or semantic rebuild.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-buffer-pool`
- `worth-store-maintenance`
- `worth-store-certification`
- `worth-foundational`

**Relevant APIs**

- `ScrubPlan`
- `ScrubExecutionReceipt`
- `ScrubWindow`
- `ScrubProgressReport`
- `ScrubResumeToken`
- `ScrubCounterSnapshot`
- `OfflineScrubInspectionInput`

**Warnings**

- Do not let scrub bypass S.2 resident, pin, and allocation envelopes.
- Do not claim that successful scrub proves semantic replay, WAL recovery, or
  repair readiness.
- Do not make offline scrub trust live runtime state unavailable to an offline
  verifier.
- Do not let scrub materialize all pages, records, diagnostics, or blobs in
  memory.
- Do not count planned, interrupted, skipped, deferred, or over-budget scrub
  windows as inspected.
- Scrub Progress Honesty Law: a scrub receipt may report only completed
  inspected windows. Resume tokens identify inspection progress only; they do
  not prove unchanged bytes unless revalidated or tied to stable physical
  generation evidence.

**Test requirements**

- Adversarial convergence: online scrub and offline scrub over the same intact
  persisted fixture produce the same physical integrity findings, locality
  summaries, and checksum counters where their declared evidence basis
  overlaps.
- Adversarial denial: scrub plans that exceed resident memory, allocation,
  streaming window, or protected-read limits deny or defer before inspection.
- Performance proof: checked-page, checked-byte, window-count, skipped-decode,
  and yielded-background-work counters are exact for scrub workloads larger
  than memory.
- Interruption proof: interrupted and resumed scrub reports preserve completed,
  skipped, deferred, over-budget, and revalidated windows as distinct counter
  classes.

**Engineering decisions**

- Scrub is a bounded integrity inspection program.
- Online scrub consumes S.2 protection; offline scrub consumes physical files
  through declared verifier inputs without trusting live runtime internals.
- Foundational performance and receipt vocabulary may describe scrub evidence
  at boundaries.
- Scrub progress is evidence of inspected windows, not proof that unvisited
  bytes are currently intact.

**Open questions**

- None.

### Phase 12: Seal Quarantine Records And Damage Classification

Phase 12 defines quarantine as a sealed Store physical record with locality,
damage class, evidence basis, and next-owner handoff posture.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-readiness`
- `worth-store-certification`
- `worth-foundational`
- `worth-proof`

**Relevant APIs**

- `QuarantineRecord`
- `QuarantineReceipt`
- `QuarantineLifecyclePosture`
- `DamageClassification`
- `IntactPhysicalBoundary`
- `IndeterminatePhysicalDamage`
- `AmbiguousBoundaryDamage`
- `RebuildableDamageInput`
- `AuthorityDamageInput`
- `PhysicalLocalityReport`
- `QuarantineHandoffPosture`

**Warnings**

- Do not let callers synthesize quarantine records from raw locality strings.
- Do not let quarantine imply repair or recovery has happened.
- Do not classify damage as rebuildable without intact authority proof.
- Do not let damaged derived artifacts outrank intact authority.
- Quarantine Seal Law: quarantine records are minted only by executed integrity
  admission, scrub, or injection-certified verifier paths.
- Quarantine Non-Mutation Law: S.3 quarantine records do not mutate
  authoritative physical bytes. They may block access, mark damage posture, and
  provide handoff evidence. Mutation, deletion, rebuild, release, or repair
  belongs to later recovery or repair authority.

**Test requirements**

- Adversarial equivalence: equivalent executed findings produce the same
  quarantine record, locality report, damage class, and Foundational receipt
  basis across independent materialization paths.
- Adversarial denial: copied report fields, logs, raw path strings, test
  fixture labels, and derived-only evidence cannot mint quarantine records.
- Authority proof: derived damage, rebuildable damage, quarantined damage, and
  unrecoverable authority damage remain distinct and non-substitutable at API
  boundaries.
- Classification proof: `IntactPhysicalBoundary`, `RebuildableDerivedDamage`,
  `QuarantinedPhysicalDamage`, `UnrecoverableAuthorityDamage`, and
  `IndeterminatePhysicalDamage` remain distinct. Ambiguous physical evidence
  must quarantine the broader honest boundary rather than invent narrower
  precision.
- Lifecycle posture proof: proposed, sealed, superseded-by-recovery,
  released-after-repair, retained-for-audit, and invalidated-by-root-change
  quarantine postures are representable without S.3 performing those later
  lifecycle transitions.

**Engineering decisions**

- Quarantine is a physical containment and evidence boundary, not repair.
- Damage classification must be strong enough for S.4 recovery and S.10 repair
  to consume without reinterpreting ambiguous logs.
- Quarantine lifecycle posture is designed for later S.4 and S.10 ownership;
  S.3 mints sealed evidence and handoff posture only.
- `worth-proof` sealing patterns should be used where they prevent forged
  quarantine or handoff forms.

**Open questions**

- None.

### Phase 13: Materialize Foundational And Proof-Compatible Integrity Evidence

Phase 13 turns executed Store integrity findings into shared boundary evidence
without moving physical-integrity authority out of Store.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-certification`
- `worth-foundational`
- `worth-proof`

**Relevant APIs**

- `PhysicalIntegrityEvidenceBundle`
- `FoundationalBoundaryRoleMapping`
- `StorePhysicalAuthorityBoundaryClaim`
- `StoreDerivedProjectionBoundaryClaim`
- `StoreSupportOnlyBoundaryClaim`
- `StorePlannedWorkBoundaryClaim`
- `StoreReceiptEvidenceBoundaryClaim`
- `IntegrityPerformanceReceipt`
- `IntegrityProvenanceAttachment`
- `IntegrityDiagnosticReport`
- `IntegrityProofProgressionReport`
- `PhysicalIntegrityCertificationReceipt`

**Warnings**

- Do not build evidence from planned checks, injected labels, or expected
  corruption alone. Evidence requires executed findings.
- Do not let Foundational diagnostic or receipt surfaces become Store runtime
  integrity authority.
- Do not export S.3 evidence without declaring whether the boundary claim is
  `AuthoritativeCurrent`, `DerivedProjection`, `SupportOnly`, `PlannedWork`,
  or `ReceiptEvidence`.
- Do not claim `AuthoritativeCurrent` for a summary, report, diagnostic, plan,
  receipt, derived index report, or support explanation.
- Do not feed `DerivedProjection`, `SupportOnly`, `PlannedWork`, or
  `ReceiptEvidence` back into Store APIs that require `IntegrityCheckedPage`,
  `QuarantineRecord`, or `S4RecoveryPhysicsIntegrityReadiness`.
- Do not let Proof progression vocabulary own physical damage semantics.
- Do not hide profile-rich forensic materialization on the operational hot
  path.

**Test requirements**

- Adversarial equivalence: the same executed integrity findings materialize the
  same Foundational diagnostic, provenance, performance, and receipt basis
  through independent constructors.
- Adversarial denial: planned checks, raw strings, copied quarantine fields,
  log excerpts, and same-run self-comparison cannot satisfy S.3 evidence APIs.
- Profile proof: reduced-richness evidence profiles remove optional forensic
  materialization while preserving integrity outcome, locality, counters, and
  denials.
- Authority denial: Foundational receipts and Proof-compatible reports cannot
  be fed back into Store as integrity-checked pages, quarantine records, or S.4
  handoff authority.
- Role-mapping proof: Store-admitted current physical authority artifacts map
  to `AuthoritativeCurrent`; rebuildable derived reports map to
  `DerivedProjection`; diagnostics and operator explanations map to
  `SupportOnly`; scenario and scrub plans before execution map to
  `PlannedWork`; executed quarantine, closeout, and certification receipts map
  to `ReceiptEvidence`.
- Projection-authority denial: Foundational `DerivedProjection`, `SupportOnly`,
  `PlannedWork`, and `ReceiptEvidence` surfaces fail compile-time or plan
  admission when supplied to Store physical authority APIs.

**Engineering decisions**

- Store counters and findings are the source of evidence.
- Foundational standardizes boundary meaning for diagnostics, provenance,
  receipts, profile, performance, and boundary role claims.
- Foundational role claims describe exported boundary posture; Store-owned
  physical witness types still prove byte integrity, quarantine validity, and
  S.4 readiness.
- Proof standardizes progression law where S.3 states need sealed movement, not
  the meaning of damaged media.

**Open questions**

- None.

### Phase 14: Publish S.4 Recovery-Physics Integrity Handoff

Phase 14 publishes the typed input S.4 needs: integrity-vetted physical records,
WAL/checkpoint damage maps, quarantine summaries, and explicit unresolved
physical damage without deciding recovery.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-recovery-physics`
- `worth-store-readiness`
- `worth-store-certification`

**Relevant APIs**

- `S4RecoveryPhysicsIntegrityReadiness`
- `S4IntegrityHandoffPayload`
- `IntegrityVettedWalFrame`
- `IntegrityVettedCheckpointRecord`
- `IntegrityVettedRootManifestRecord`
- `IntegrityVettedSegmentManifestRecord`
- `IntegrityVettedPageFrameRecord`
- `IntegrityDamageMap`
- `RootManifestIntegrityPosture`
- `RecoveryBlockedByIntegrityDamage`
- `RecoveryIntegrityHandoffReceipt`

**Warnings**

- Do not let S.3 choose recovery source precedence.
- Do not mark a damaged WAL frame replayable because integrity localized the
  damage.
- Do not pass raw WAL, page, or checkpoint bytes to S.4 when typed
  integrity-vetted records or damage maps exist.
- Do not hide unresolved physical damage behind a successful S.3 closeout.
- Do not produce a symbolic handoff that omits the physical basis, counters,
  skipped-decode evidence, or proof that raw bytes were not passed.

**Test requirements**

- Adversarial replay: intact WAL/checkpoint/page inputs produce stable S.4
  handoff identity across independent S.3 inspection runs.
- Adversarial denial: damaged WAL frames, damaged checkpoint-adjacent records,
  damaged manifest roots, and unresolved authority damage produce typed
  recovery-blocking integrity evidence rather than replay inputs.
- Boundary proof: S.4 handoff records cannot be constructed from raw bytes,
  copied integrity reports, or quarantine summaries without executed integrity
  receipts.
- Payload proof: `S4RecoveryPhysicsIntegrityReadiness` contains integrity-vetted
  root manifest records, segment or extent manifest records, page/frame records
  relevant to recovery, WAL frames, checkpoint-adjacent records,
  WAL/checkpoint damage maps, root/manifest posture, quarantine summaries,
  unresolved authority damage, recovery-blocking findings, checksum
  algorithm/scope basis, exact counters, skipped-decode counters, bounded
  inspection-envelope evidence, and proof that raw bytes were not passed.

**Engineering decisions**

- S.3 closes the integrity precondition for recovery.
- S.4 owns recovery physics once it receives integrity-vetted records and
  damage maps.
- Recovery blockers are valid S.3 outputs when bytes are physically unsafe.
- A valid handoff is concrete enough for S.4 to start recovery planning without
  re-opening S.3 integrity proof boundaries.

**Open questions**

- None.

### Phase 15: Close Physical Integrity And Corruption Localization

Phase 15 runs the named S.3 suites, rejects synthetic shortcuts, verifies
bounded execution, and records that S.4 can begin from typed integrity evidence.

**Relevant subsystems**

- `worth-store-physical-integrity`
- `worth-store-certification`
- `worth-store-readiness`
- `worth-store-buffer-pool`
- `worth-foundational`
- `worth-proof`

**Relevant APIs**

- `PhysicalIntegrityCloseoutSuite`
- `PhysicalIntegrityCertificationBundle`
- `PhysicalIntegrityCloseoutReport`
- `SyntheticIntegrityShortcutRejectionReport`
- `S4RecoveryPhysicsIntegrityReadiness`

**Warnings**

- Do not close S.3 on checksum success alone.
- Do not close S.3 on logs, expected failure messages, small in-memory buffers,
  or same-run self-comparison.
- Do not claim S.4 recovery, S.7 blob lifecycle, S.10 repair, S.11
  authenticity/security, or S.12 aerospace-grade certification from S.3.
- Do not leave S.4 with raw byte handles or untyped damage reports.

**Test requirements**

- Adversarial closeout: injected byte flips, torn frames, stale generations,
  manifest corruption, index-page corruption, WAL-frame corruption, extent
  damage, and chunk damage all localize to typed physical boundaries before
  any semantic decoder consumes the bytes.
- Adversarial denial: forged checksums, digest-as-checksum claims,
  checksum-as-authenticity claims, raw byte entry, copied quarantine records,
  and over-budget scrub plans all deny at named S.3 boundaries.
- Harness closeout: every S.3 acceptance suite runs through the inherited
  Roadmap 2 harness pipeline from definition to transcript, and the closeout
  bundle names lane, driver, observer, oracle, transcript, and evidence
  families for each suite.
- Synthetic-test rejection: suites that prove integrity only through logs,
  expected errors, in-memory-only buffers, test-support-owned oracles, or
  fixture labels fail before S.3 can close.
- Handoff proof: S.4 receives `S4RecoveryPhysicsIntegrityReadiness` containing
  integrity-vetted physical records, typed damage maps, quarantine summaries,
  exact counters, and recovery-blocking integrity findings where applicable.
- Line-cap and composition proof: production and test modules stay under the
  workspace line cap unless explicitly exempted and keep checksum, scrub,
  quarantine, evidence, and handoff responsibilities separate.

**Engineering decisions**

- S.3 closeout proves physical integrity and localization only.
- S.3 explicitly reserves recovery physics, blob lifecycle, repair, security,
  and full database certification for later Roadmap 2 sequences.
- The closeout must produce enough handoff evidence for S.4 to start without
  re-opening S.3 proof boundaries.

**Open questions**

- None.

## Must Ship

- typed consumption of `S3PhysicalIntegrityReadiness`
- sealed integrity entry witnesses and protected byte inspection leases
- declared checksum algorithm, detection model, scope, coverage, and
  compatibility law
- physical scope admission before family-specific validation
- pre-decode physical admission gates for pages, frames, manifests, WAL
  frames, index pages, extents, and chunks
- audited semantic decoder non-interference after physical denial
- page, frame, extent, slot-directory, manifest, allocation-map, index-page,
  WAL-frame, and chunk integrity reports
- torn-frame, stale-generation, misplaced-reference, checksum-scope, and
  unsupported-algorithm denials
- root-manifest integrity posture and cross-family intact-wrong-scope denials
- explicit checksum versus digest versus authenticity separation
- derived-damage versus authority-damage classification, including
  indeterminate physical damage and honest ambiguous-boundary localization
- bounded online and offline scrub planning and execution
- interrupted-scrub progress reports that distinguish completed, skipped,
  deferred, over-budget, and revalidated windows
- sealed quarantine records, lifecycle postures, quarantine receipts, locality
  reports, and damage classifications
- Roadmap 2 S.3 scenario definitions, lowered plans, drivers, observers,
  oracles, transcripts, and evidence bundles
- deterministic corruption injection mechanics in test support without moving
  proof meaning into test support
- Foundational diagnostics, provenance, performance, profile, and receipt
  evidence materialized from executed Store findings
- Foundational boundary role mapping for Store authority, derived projection,
  support-only diagnostics, planned work, and receipt evidence
- projection-authority denial proving Foundational role claims cannot replace
  Store physical witnesses
- Proof-compatible progression/sealing where S.3 states use shared
  proof-bearing patterns
- concrete `S4RecoveryPhysicsIntegrityReadiness` handoff payload

## Must Preserve

- Store owns physical integrity, corruption localization, quarantine, and
  damage classification.
- `worth-relational` owns semantic truth, transaction meaning, branch meaning,
  and logical decode semantics.
- `worth-foundational` standardizes boundary evidence meaning; it does not own
  page, frame, chunk, WAL, or manifest integrity authority.
- `worth-proof` standardizes proof-bearing progression law; it does not own
  media semantics, checksum algorithms, or recovery decisions.
- Checksums do not prove authenticity.
- Artifact digests do not replace physical integrity checks.
- Derived artifacts never outrank intact authority.
- Scrub and quarantine do not perform repair.
- S.4 recovery physics, S.7 blob lifecycle, S.10 repair, S.11 security, and
  S.12 certification remain later sequence responsibilities.

## Acceptance Evidence

S.3 is complete only when the store satisfies the Roadmap 2 named suite:

- `Physical integrity and corruption-localization test`

Required machine-checkable outputs:

- `physical_integrity_story_transcript`
- `physical_scenario_definition`
- `physical_scenario_plan`
- `physical_scenario_execution_report`
- `physical_proof_oracle_verdict`
- `checksum_scope_trace`
- `checksum_detection_model_trace`
- `physical_scope_admission_trace`
- `damage_injection_trace`
- `pre_decode_denial_trace`
- `semantic_decoder_non_interference_trace`
- `scrub_window_counter_trace`
- `quarantine_locality_trace`
- `damage_classification_trace`
- `s4_integrity_handoff_trace`
- `foundational_integrity_receipt_trace`
- `foundational_boundary_role_trace`
- `projection_authority_denial_trace`
- `runtime_verifier_parity_trace`
- `shortcut_rejection_trace`
- `fixture_adversary_report`
- `resource_envelope_report`
- `hardware_assumption_report`
- `integrity_basis_trace`
- `recovery_claim_absence_report`
- `blob_lifecycle_claim_absence_report`
- `test_support_authority_denial_report`
- `foundational_diagnostic_bundle`
- `foundational_counter_backed_performance_receipt`
- `S4RecoveryPhysicsIntegrityReadiness`

Every acceptance suite must map to the S.3 certification matrix in the harness
test plan. A suite is not accepted unless it supplies its required fixture
classes, lowered scenario plan, required proof outputs, exact counters, and
one positive plus one hostile negative case.

- `integrity_entry_authority_suite`
  proves S.3 consumes typed readiness and rejects raw buffers, backend handles,
  copied readiness, and expired protected views.
- `checksum_scope_coverage_suite`
  proves every admitted physical family declares algorithm, scope, covered
  bytes, excluded bytes, compatibility posture, and mismatch denials.
- `checksum_detection_model_honesty_suite`
  proves checksum success is admitted only under a declared detection model and
  never upgraded into authenticity or impossible-forgery proof.
- `physical_scope_admission_suite`
  proves physical reference, generation, manifest membership, root posture,
  checkpoint adjacency, and checksum scope are admitted before family-specific
  validation.
- `pre_decode_physical_denial_suite`
  proves damaged bytes skip logical decode and emit exact skipped-decode
  counters.
- `semantic_decoder_non_interference_suite`
  proves semantic decoders, semantic indexes, and domain-object constructors
  have zero invocations after failed physical admission, including for
  syntactically plausible corrupt bytes.
- `page_frame_extent_damage_suite`
  proves page, frame, slot-directory, extent, and torn-frame corruption
  localizes before record views or semantic decoders run.
- `ambiguous_boundary_locality_suite`
  proves damaged headers, torn lengths, damaged roots, and boundary-spanning
  torn writes produce indeterminate or broader honest locality instead of false
  precision.
- `manifest_root_allocation_integrity_suite`
  proves root manifests, segment manifests, allocation maps, and free-space
  maps cannot be trusted through backend residue or stale references.
- `derived_index_damage_classification_suite`
  proves derived damage, rebuildable damage, and authority damage remain
  distinct and non-substitutable.
- `wal_frame_integrity_without_recovery_suite`
  proves WAL-frame and checkpoint-adjacent damage is classified without
  claiming replay, redo, or checkpoint source precedence.
- `chunk_integrity_without_blob_lifecycle_suite`
  proves large extent and chunk damage localizes through bounded streaming
  windows without claiming S.7 blob lifecycle.
- `stale_generation_and_scope_mismatch_suite`
  proves individually intact bytes still deny when their physical generation,
  manifest scope, checksum scope, or reference identity is wrong.
- `intact_wrong_scope_cross_family_suite`
  repeats the intact-but-wrong adversary across pages, frames, manifests, WAL
  frames, chunks, and derived index pages.
- `bounded_online_offline_scrub_suite`
  proves online and offline scrub execute within S.2 resident and allocation
  envelopes with exact checked-page, checked-byte, window, and skipped-decode
  counters.
- `quarantine_sealing_suite`
  proves quarantine records can be minted only by executed integrity findings
  and cannot be forged from logs, raw path strings, copied fields, or fixture
  labels.
- `foundational_integrity_evidence_suite`
  proves executed Store findings materialize into Foundational diagnostics,
  provenance, performance, profile, and receipt vocabulary only at evidence
  boundaries.
- `foundational_boundary_role_mapping_suite`
  proves Store-owned authority, derived projections, support-only reports,
  planned work, and receipt evidence map to Foundational roles without letting
  those roles replace Store physical proof types.
- `proof_progression_integrity_state_suite`
  proves intact, damaged, quarantined, rebuildable, unrecoverable, and
  handoff-ready forms remain mechanically distinct.
- `s4_integrity_handoff_suite`
  proves S.4 receives integrity-vetted physical records, typed damage maps,
  quarantine summaries, exact counters, and recovery-blocking findings where
  applicable.
- `synthetic_integrity_test_rejection_suite`
  proves logs, same-run self-comparison, in-memory-only fixtures,
  test-support-owned oracles, expected error text, and fixture labels cannot
  close S.3.

## Allowed Debt

S.3 may reserve stronger future checksum algorithms, additional backend-specific
media integrity acceleration, and richer operator-facing quarantine UI for
later sequences when the ordinary physical-integrity law already exists.

S.3 may not mark these as debt:

- typed S.3 readiness consumption
- pre-decode physical admission
- page/frame/manifest/WAL/index/chunk checksum scope law
- checksum versus digest separation
- checksum versus authenticity separation
- stale generation and scope mismatch detection
- torn-frame detection
- byte-flip and damage injection certification lanes
- derived versus authority damage classification
- bounded online/offline scrub
- sealed quarantine records
- exact integrity and scrub counters
- Foundational boundary evidence from executed findings
- Foundational boundary role mapping for S.3 evidence exports
- projection-authority denial for Foundational role claims
- Proof-compatible sealed progression for S.3 state transitions where used
- concrete S.4 integrity handoff
- physical scope admission before family validation
- semantic decoder non-interference proof
- indeterminate or ambiguous-boundary damage classification
- concrete checksum detection models

## Sequencing Notes

S.3 belongs immediately after S.2 because integrity inspection needs bounded,
protected byte access before it can honestly inspect stores larger than memory.
It belongs before S.4 because recovery physics must consume integrity-vetted
physical records and typed damage maps rather than raw bytes or backend residue.

Later sequences consume S.3 as follows:

- S.4 consumes integrity-vetted WAL, checkpoint, page, manifest, and damage-map
  inputs before deciding recovery physics.
- S.5 consumes physical stability and quarantine locality when maintenance
  moves pages underneath readers.
- S.7 consumes chunk integrity primitives when native blob chunk lifecycle is
  introduced.
- S.10 consumes quarantine records and damage classification as repair and
  offline verification inputs.
- S.11 adds authenticity, encryption, key, tenant, and audit evidence that
  checksums deliberately do not claim.
- S.12 consumes S.3 as one certification lane, not as full aerospace-grade
  database proof.

## Required Self-Check

- Does S.3 solve a real structural problem? Yes: it prevents damaged physical
  bytes from becoming ambiguous semantic failures.
- Is the adversarial constraint precise and load-bearing? Yes: damage must
  deny before decode, localize physically, remain bounded, and produce typed
  evidence.
- Does the roadmap justify this milestone now? Yes: Roadmap 2 places S.3 after
  bounded residency and before WAL recovery so recovery can consume
  integrity-vetted bytes.
- Does the spec preserve crate authority boundaries? Yes: Store owns physical
  integrity; Foundational owns shared evidence vocabulary; Proof owns
  progression law; Relational owns semantic truth.
- Are the phases carrying most of the design information? Yes.
- Is each phase centered on one conceptual detail or boundary? Yes.
- Does each phase contain adversarial tests? Yes: every phase includes
  replay/parity/equivalence plus denial/localization/boundary tests.
- Could a competent engineer map this into honest types, modules, and tests?
  Yes: phases name subsystems, APIs, warnings, tests, and decisions.
- Does the milestone belong in this roadmap sequence? Yes: S.3 is the required
  bridge from bounded byte access to recovery physics.
