# Milestone 8 Closeout: Structural-Identity-Aware Remapping

## Status

Milestone 8 is closed as of 2026-04-08.

The runtime bridge now treats structural identity as a first-class,
replay-safe, bridge-owned advisory protocol for remapping, reuse, and branch
comparison rather than as adapter folklore or host-local "looks similar"
heuristics.

The semantic center shipped in this milestone is:

one canonical structural declaration lowers through admitted structural
contract truth into one validated structural phase, that phase lowers into one
bridge-planned structural packet set over one explicit truth-view basis and
one explicit equivalence contract, the packet set reduces into one typed
structural outcome before any publication occurs, publication remains split
between advisory remap and branch comparison artifacts, and canonical records,
diagnostics, replay, and certification all consume those same bridge-owned
artifacts rather than rediscovering structural meaning from ambient latest
truth, host iteration order, or custom scoring folklore.

This is not "the bridge can find similar things."
Milestone 8 made structural sameness, ambiguity, authority separation, branch
drift resistance, and replay sufficiency explicit, typed, and certifiable.

The bridge now owns:

- a canonical structural declaration, validation, contract, planning,
  reduction, publication, record, and explanation vocabulary
- schema-scoped structural equivalence contracts with explicit
  `fingerprint_semantics_version`
- bridge-owned structural fingerprint materialization from admitted truth-view
  reads rather than host-side structural scoring
- typed structural outcomes for exact advisory match, advisory reuse,
  ambiguous structural match, no structural match, identity authority conflict,
  and lineage structural divergence
- replay-safe structural remap and branch-comparison records with retained
  canonical planning and artifact truth
- structural diagnostics and explanation surfaces derived from canonical
  records rather than host logs
- harness certification for ambiguity rejection, reuse without identity
  fusion, branch-local drift determinism, replay parity, diagnostics-tier
  invariance, and generated adversarial oscillation workloads

## Shipped Scope

Milestone 8 delivered:

- a dedicated bridge-owned `structural/` subsystem split across declaration,
  taxonomy, validation, contracts, planning, reduction, publication,
  fingerprints, and matching
- canonical `StructuralIdentityDeclaration`,
  `ValidatedStructuralIdentityDeclaration`,
  `AdmittedStructuralComparisonContract`,
  `PlannedStructuralMatchPacketSet`,
  `ReducedStructuralMatchSet`,
  `PublishedStructuralRemapArtifact`, and
  `PublishedBranchComparisonArtifact` surfaces
- builder-time structural registration, canonical structural registry freezing,
  duplicate and ambiguous declaration rejection, and explicit admission
  through the frozen registry
- bridge-owned structural fingerprint materialization from truth-view reads for
  advisory remap and branch-pair comparison
- deterministic structural candidate classification and reduction rather than
  host-local winner selection
- canonical structural remap and branch-comparison records retained in bridge
  diagnostics
- replay from canonical structural records, including typed rejection for
  incompatible record versions and truncated replay basis
- structural counter surfaces covering declaration, contract, fingerprint,
  candidate, ambiguity, mismatch, identity conflict, lineage divergence,
  branch comparison, branch diff, widened-scan, and replay request/mismatch
  accounting
- harness-grade structural certification bundles for Milestone 8 suites 7, 8,
  and 9
- generated adversarial structural certification workloads for rejection
  matrices, branch-head oscillation, replay parity, and unrelated-publication
  locality

## Acceptance Mapping

Milestone 8 is considered closed against the roadmap, the engineering spec,
and `test-requirements.md` because the required acceptance surfaces are now
covered directly.

### `Structural identity can assist remapping without overriding authoritative identity`

Covered by:

- `facade::tests::structural::runtime_derives_structural_candidates_from_read_packets`
- `facade::tests::structural::runtime_derives_identity_authority_conflict_from_same_snapshot_same_structure`
- `harness::tests::structural::certification::bridge_harness_structural_suite_7_emits_match_and_ambiguity_truth_without_winner_selection`
- `harness::tests::structural::certification::bridge_harness_structural_suite_8_preserves_identity_separation_and_replay`

What is proven:

- exact structural matches lower through bridge-owned packet planning and
  reduction
- same-shape different-authority candidates reduce to typed identity conflict
  rather than continuity fabrication
- ambiguous and no-safe-match lanes fail closed with canonical failure digests
- remap publication occurs only from admissible reduced structural truth

### `Structural reuse remains advisory and replay-safe`

Covered by:

- `facade::tests::structural::runtime_canonicalizes_and_replays_structural_remap_record`
- `facade::tests::structural::runtime_replay_rejects_truncated_structural_remap_basis`
- `facade::tests::structural::runtime_rejects_structural_declaration_with_different_semantics_version`
- `harness::tests::structural::certification::bridge_harness_structural_suite_8_preserves_identity_separation_and_replay`
- `harness::tests::structural::generated::generated_structural_rejection_matrix_preserves_typed_fail_closed_outcomes`

What is proven:

- replay reproduces the same advisory remap artifact from canonical structural
  records
- replay fails explicitly when the retained structural basis is truncated
- semantics-version drift is an explicit contract mismatch, not best-effort
  compatibility
- diagnostics-tier changes do not create a third reuse meaning

### `Branch comparison remains deterministic, local, and explainable under drift`

Covered by:

- `facade::tests::structural::runtime_derives_branch_comparison_candidates_from_branch_pair_reads`
- `facade::tests::structural::runtime_branch_comparison_ignores_read_result_order_when_structure_is_equal`
- `facade::tests::structural::runtime_canonicalizes_and_replays_structural_branch_comparison_record`
- `facade::tests::structural::runtime_replay_rejects_truncated_structural_branch_basis`
- `harness::tests::structural::certification::bridge_harness_structural_suite_9_preserves_branch_diff_and_replay_determinism`
- `harness::tests::structural::certification::branch_head_structural_comparison_oscillates_predictably_under_branch_drift`
- `harness::tests::structural::generated::generated_branch_head_oscillation_sequence_remains_local_and_replay_safe`

What is proven:

- branch comparison is derived from an explicit branch-pair basis, not ambient
  latest truth
- read-result order does not change structural branch judgment
- unrelated publication on other branches does not contaminate branch-local
  comparison
- branch-head oscillation produces the expected `1 -> 0 -> 1` diff behavior
- replay preserves the same branch comparison artifact and localized diff truth

### `Milestone 8 certification bundles are machine-checkable and offline-auditable`

Covered by:

- `harness::tests::structural::certification::bridge_harness_structural_suite_7_emits_match_and_ambiguity_truth_without_winner_selection`
- `harness::tests::structural::certification::bridge_harness_structural_suite_8_preserves_identity_separation_and_replay`
- `harness::tests::structural::certification::bridge_harness_structural_suite_9_preserves_branch_diff_and_replay_determinism`
- `harness::tests::structural::generated::generated_exact_match_control_and_candidate_runs_preserve_same_certification_bundle`

What is proven:

- structural certification emits canonical bundle fields for
  `structural_match_digest`, `ambiguity_report`, `remap_artifact_digest`,
  `failure_digest`, `structural_reuse_digest`, `identity_separation_report`,
  `branch_compare_digest`, `structural_diff_report`, `replay_digest`,
  `diagnostics_digest`, and `counter_snapshot`
- equivalent runs compare equal across independently produced bundles
- rejection lanes remain typed and machine-checkable rather than log-shaped
- widened-scan and replay-mismatch counters remain mechanically zero on default
  paths

## Additional Hardening Added Before Close

Milestone 8 closeout includes these extra hardening outcomes beyond the minimum
phase labels:

- structural equivalence was hardened against read-result order so persistent
  naming and geometry-style structural identity do not drift with adapter
  ordering
- derived structural candidate identities now use stable semantic names rather
  than enum ordinals
- structural proof-bearing phase constructors were sealed so callers cannot
  synthesize progressed structural states outside the bridge path
- structural counters were widened to cover replay requests, replay mismatch,
  and branch-drift rejection floor fields in the core structural counter type
  rather than only in harness-local summaries
- closeout added explicit replay-basis truncation proofs for both remap and
  branch-comparison records
- closeout added explicit semantics-version incompatibility proof so differing
  structural equivalence contracts fail typed instead of degrading into ambient
  compatibility
- certification assertions were tightened to prove zero widened-scan and zero
  replay-mismatch residue on the default admitted paths
- generated adversarial certification workloads were added so Milestone 8 is
  not justified only by hand-authored happy-path fixtures

These changes were made because the closeout bar was not "structural matching
works on a fixture." The closeout bar was trust-grade authority separation,
geometry-safe deterministic naming behavior, replay sufficiency, cost honesty,
and certification evidence strong enough to support Milestone 9 history work
without reopening structural identity rules.

## Explicit Deferrals

Milestone 8 intentionally does not include:

- merge-aware structural interpretation across ordered multi-parent histories
- speculative preview structural semantics
- cross-runtime policy provenance on structural publication
- bridge-mediated writeback or structural commit strategy production
- cross-schema structural reuse without an explicit shared equivalence contract
- automatic widened-scan admission as a default execution strategy

Those remain later roadmap work and were not smuggled into Milestone 8 under
ambiguous names.

## Verification Baseline

At closeout, the verification baseline for the milestone implementation is:

- `cargo fmt --all`
- `cargo test -p worth-runtime-bridge`

This passes cleanly and includes:

- 283 unit and harness tests
- 1 no-`inc.rs` structural test
- 1 compile-fail boundary test crate
- 8 trybuild/UI phase-boundary tests
- structural runtime, replay, diagnostics, certification, and generated
  adversarial coverage

## Operational Conclusion

Milestone 8 is now closed at the bridge level.

The runtime bridge no longer treats structural likeness as an implicit host
convenience. It now owns a real structural protocol: canonical declaration and
equivalence-contract identity, proof-bearing admission, bridge-owned
fingerprint materialization, deterministic packet planning and reduction,
typed ambiguity and authority conflict outcomes, replay-safe structural
records, branch-local deterministic comparison, explicit counter surfaces, and
certification evidence strong enough to carry Milestone 9 and later work
without reopening the structural authority boundary.
