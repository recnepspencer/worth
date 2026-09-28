# Milestone 10 Closeout: Speculative Truth-Branch To Signal-Branch Coordination And Preview Flows

## Status

Milestone 10 is closed as of 2026-04-09.

The runtime bridge now treats speculative truth branches, speculative signal
branches, preview sessions, discard boundaries, and preview-to-authority
promotion as a first-class bridge protocol rather than as adapter folklore,
UI-local preview state, or cleanup-by-convention.

The semantic center shipped in this milestone is:

one canonical preview declaration lowers through one bridge-owned speculative
branch binding into one closed preview typestate progression, admitted preview
work lowers into one canonical non-authoritative execution record, discard
consumes that active preview state into one residue-classified terminal record,
promotion consumes that same active preview state only through one closed
admissibility proof into one explicit authoritative-boundary record, replay and
diagnostics reconstruct meaning from those canonical records alone, and
certification bundles prove that preview churn stays isolated, discard leaves
zero authoritative residue, and promotion cannot be confused with preview or
discard under replay, diagnostics-tier variation, restart, or host variation.

This is not "the host can run a preview."
Milestone 10 made speculative authority boundaries, typestate progression,
discard finality, replay sufficiency, and promotion provenance explicit, typed,
and certifiable.

The bridge now owns:

- a dedicated `speculation/` subsystem for branch binding, preview declaration,
  typestate sessions, execution, discard, promotion, replay, contracts,
  counters, residue taxonomy, and validation
- bridge-owned `BridgeRequestKind`, `BridgeSpeculativeBranchBinding`,
  `BridgePreviewSessionDeclaration`, `BridgePreviewSession<...>`,
  `BridgePreviewExecutionRecord`, `BridgePreviewDiscardRecord`,
  `BridgePreviewPromotionRecord`, `BridgePromotionAdmissibilityProof`,
  `BridgePreviewReuseEquivalence`, `BridgePreviewReplayBundle`,
  `BridgePreviewResidueClass`, and `BridgeSpeculationCounters` surfaces
- a closed preview lifecycle progression:
  `Declared -> Admitted -> Active -> Discarded | Promoted`
- runtime-owned preview session identity reservation so speculative lifecycle
  identity cannot be forked or re-admitted after terminal discard/promotion
- explicit discard residue classification with fail-closed rejection when any
  authoritative residue remains
- explicit promotion provenance with stale, duplicate, and post-discard
  promotion rejection
- replay-safe preview, discard, and promotion bundles reconstructed from
  canonical retained records rather than ambient host state
- diagnostics and explanation surfaces derived from canonical speculation
  records rather than hidden runtime-local state
- certification bundles and proof tests satisfying Milestone 10 suites 13, 14,
  and 15

## Shipped Scope

Milestone 10 delivered:

- a dedicated bridge-owned `speculation/` subsystem split across `binding`,
  `contracts`, `counters`, `declaration`, `discard`, `execution`, `promotion`,
  `replay`, `session`, `taxonomy`, and `validation`
- canonical `BridgePreviewSessionDeclaration`,
  `ValidatedBridgePreviewSessionDeclaration`,
  `BridgeSpeculativeBranchBinding`,
  `BridgePreviewSession<Declared>`,
  `BridgePreviewSession<Admitted>`,
  `BridgePreviewSession<Active>`,
  `BridgePreviewSession<Discarded>`,
  and `BridgePreviewSession<Promoted>` surfaces
- bridge-owned request-kind separation between authoritative and preview work,
  with discard and promotion modeled as lifecycle transitions rather than peer
  execution modes
- closed promotion admissibility proofs covering preview-session identity,
  execution-record identity, branch binding, truth-view basis, request shape,
  source-capability digest, and retained artifact schema/version digest
- exact preview reuse admission through explicit equivalence proofs rather than
  ambient cache reuse
- replay-safe preview execution records and canonical discard/promotion records
- residue taxonomy and discard residue reports over concrete artifact classes
  rather than one informal cleanup bucket
- speculation-specific diagnostics queries and explanation surfaces for
  execution, discard, promotion, and replay
- runtime-owned reservation of preview session identity in diagnostics state so
  session identity is authoritative and cannot be reused after discard or
  promotion
- speculation counters covering preview execution width, discard breadth,
  retained non-authoritative artifacts, promotion proof checks, and replay
  bundle width
- harness-grade speculation certification bundles for suites 13 through 15

## Acceptance Mapping

Milestone 10 is considered closed against the roadmap, the engineering spec,
and `test-requirements.md` because the required acceptance surfaces are now
covered directly.

### `Discarded preview flows leave zero authoritative residue`

Covered by:

- `facade::tests::speculation::runtime_activates_and_discards_preview_session_with_zero_authoritative_residue`
- `facade::tests::speculation::runtime_rejects_preview_discard_when_authoritative_residue_remains`
- `facade::tests::speculation::runtime_replays_discarded_preview_bundle_from_retained_records`
- `facade::tests::speculation::runtime_rejects_post_discard_reentry_and_preserves_canonical_discard_bundle`
- `harness::tests::speculation_certification::speculative_discard_zero_residue_bundle_is_canonical_and_host_parity_safe`

What is proven:

- discard emits an explicit residue report and rejects any authoritative residue
- discard replay reconstructs the same canonical discarded outcome from
  retained preview artifacts alone
- discarded preview identity remains terminal and cannot be re-entered through
  hostile re-admission
- discarded preview flows emit the required suite 13 fields:
  `speculative_resource_digest`, `discard_residue_report`, `routing_digest`,
  and `counter_snapshot`
- equivalent discard lanes compare equal across host variation while preserving
  zero authoritative residue

### `Promotion happens only through explicit canonical preview-to-authority records`

Covered by:

- `facade::tests::speculation::runtime_promotes_preview_session_and_replays_promoted_bundle`
- `facade::tests::speculation::runtime_rejects_stale_duplicate_and_post_discard_promotion`
- `facade::tests::speculation::runtime_explains_preview_promotion_and_replay_from_retained_records`
- `harness::tests::speculation_certification::speculative_commit_boundary_bundle_stays_replay_safe_and_tier_explicit`

What is proven:

- promotion consumes an active preview session plus a closed admissibility proof
- stale proof use fails typed rather than degrading into best-effort promotion
- preview identity conflicts prevent duplicate lifecycle admission
- promotion replay reconstructs the same preview-origin promotion record after
  restart
- suite 14 bundles keep promoted preview, discarded sibling preview, and
  authoritative routing truth mechanically distinct through
  `speculative_commit_digest`, `preview_vs_authoritative_matrix`,
  `replay_digest`, and `diagnostics_digest`
- diagnostics-tier variation changes retained detail only and does not create a
  third promotion meaning

### `Preview churn stays bounded, isolated, and non-authoritative`

Covered by:

- `facade::tests::speculation::runtime_admits_and_activates_reused_preview_session_only_for_exact_equivalence`
- `facade::tests::speculation::runtime_rejects_preview_reuse_when_target_basis_drifts`
- `harness::tests::counters::speculation_counters_capture_preview_discard_promotion_and_replay_widths`
- `harness::tests::speculation_certification::preview_lifecycle_churn_bundle_stays_bounded_and_branch_isolated`

What is proven:

- reuse is admitted only when the full preview basis remains exactly equivalent
- drift in structural basis or request basis fails reuse admission typed
- named speculation counters remain exact and visible to callers
- authoritative routing stays stable while preview sessions churn and discard
- suite 15 bundles emit `preview_lifecycle_digest`, `resource_bound_report`,
  `branch_isolation_matrix`, and `counter_snapshot`
- repeated preview churn remains request-scoped rather than degrading into
  ambient cross-session state

### `Replay and diagnostics are derived from canonical speculation artifacts`

Covered by:

- `facade::tests::speculation::runtime_replays_discarded_preview_bundle_from_retained_records`
- `facade::tests::speculation::runtime_promotes_preview_session_and_replays_promoted_bundle`
- `facade::tests::speculation::runtime_explains_preview_promotion_and_replay_from_retained_records`
- `facade::tests::speculation::runtime_explains_preview_discard_from_retained_records`
- `harness::tests::speculation::bridge_harness_speculation_discard_replay_remains_queryable`

What is proven:

- replay reconstructs preview execution plus exactly one terminal outcome:
  discard or promotion
- diagnostics explanation surfaces are derived from retained canonical records
  and remain subordinate to those records
- replay rejects illegal terminal-state ambiguity rather than inventing a
  mixed preview meaning

## Additional Hardening Added Before Close

Milestone 10 closeout includes these extra hardening outcomes beyond the minimum
phase labels:

- preview typestate constructors and lifecycle transitions were sealed to
  crate-owned surfaces so callers cannot synthesize progressed preview states
  outside bridge authority
- preview sessions were made move-only rather than cloneable so duplicate
  discard/promotion-capable handles are mechanically suppressed
- preview session identity is now runtime-reserved and remains reserved after
  terminal discard or promotion, preventing hostile re-entry from forking the
  canonical lifecycle
- discard terminality was hardened with an adversarial replay-preservation test
  so rejected re-entry attempts cannot overwrite or blur the retained discard
  record
- speculation certification was split out of a 600+ line god file into
  dedicated discard, promotion, churn, and shared modules that match the
  milestone concepts directly
- suite 14 certification now proves a promoted preview plus discarded sibling
  preview in the same bundle rather than a too-happy all-promoted story
- suite 15 certification now proves authoritative routing stability during
  discard-heavy churn instead of only summarizing churn after the fact
- speculation counters were tightened into exact proof tests for preview,
  discard, promotion, and replay widths rather than presence-only checks

These changes were made because the closeout bar was not "preview seems to
work." The closeout bar was trust-grade authority separation, discard finality,
explicit promotion provenance, replay sufficiency, cost honesty, and
certification evidence strong enough to support Milestone 11 policy propagation
without reopening speculative authority rules.

## Explicit Deferrals

Milestone 10 intentionally does not include:

- cross-runtime policy vocabulary or policy provenance beyond leaving room for
  Milestone 11
- bridge-mediated writeback semantics or authoritative mutation planning
- new speculative semantics invented inside the truth or signal runtimes
- ambient preview caches, shared cross-session temporary state, or implicit
  preview-to-authority shortcuts
- end-to-end causality bundle unification, which remains later roadmap work

Those remain later roadmap work and were not smuggled into Milestone 10 under
ambiguous names.

## Verification Baseline

At closeout, the verification baseline for the milestone implementation is:

- `cargo fmt --all`
- `cargo test -p worth-runtime-bridge --lib`

This passes cleanly and includes:

- speculation facade tests for declaration, reuse, discard, promotion, replay,
  explanation, and adversarial terminality
- harness tests for speculation diagnostics, counters, and certification
- certification coverage for suites 13, 14, and 15 with canonical
  machine-checkable bundles

## Operational Conclusion

Milestone 10 is now closed at the bridge level.

The runtime bridge no longer treats preview and speculative coordination as an
ambient adapter behavior. It now owns a real speculation protocol: canonical
branch binding, bridge-owned preview declaration and lifecycle identity, closed
typestate progression, fail-closed reuse admission, zero-residue discard,
explicit promotion provenance, replay-safe canonical records, derived
diagnostics and explanations, exact counter surfaces, and certification
evidence strong enough to carry Milestone 11 and later work without reopening
preview-versus-authoritative authority boundaries.
