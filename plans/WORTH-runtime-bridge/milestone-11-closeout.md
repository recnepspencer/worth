# Milestone 11 Closeout: Cross-Runtime Policy Propagation And Clean Configuration Model

## Status

Milestone 11 is closed as of 2026-04-09.

The runtime bridge now treats cross-runtime policy declaration, validation,
admission, lowering, provenance, replay, diagnostics explanation, and
certification as a first-class bridge protocol rather than as builder folklore,
request-local convenience knobs, or ambient runtime state.

The semantic center shipped in this milestone is:

one canonical bridge-owned policy declaration lowers through one explicit
validation and admission path into one lowered execution policy, route planning
consumes only a monotonic projection of that lowered policy, replay
reconstructs policy-scoped meaning from canonical records rather than from live
runtime memory, diagnostics explanations derive from canonical policy artifacts
instead of introspection, and certification bundles prove builder-order parity,
typed illegality, replay sufficiency, and ambient leak resistance across the
named Milestone 11 hostile lanes.

This is not "the bridge has some runtime mode flags now."
Milestone 11 made policy source separation, typed rejection, canonical policy
identity, route-policy propagation, replay-safe provenance, and certification
proofs explicit, typed, and replay-safe.

The bridge now owns:

- a dedicated `policy/` subsystem for taxonomy, declaration, validation,
  admission, lowering, provenance, replay bundles, counters, reports, and
  typed rejection
- bridge-owned `BridgePolicyDeclaration`,
  `ValidatedBridgePolicyDeclaration`, `AdmittedBridgePolicyContract`,
  `LoweredBridgeExecutionPolicy`, `BridgeRoutePlanningPolicy`,
  `BridgePolicyProvenanceRecord`, `BridgePolicyReplayBundle`,
  `BridgePolicyCounters`, `BridgePolicyProvenanceReport`,
  `BridgePolicyRejection`, and `BridgePolicyRejectionStage` surfaces
- explicit baseline partitioning for execution, diagnostics, and artifact
  policy inside `BridgeRuntimePolicy`
- request-scoped route-policy projection and route-proof propagation through
  planning, delivery, route records, canonical route records, and replay
- replay-safe policy and route-policy reconstruction from canonical records
  alone
- bridge-owned policy explanation surfaces derived from canonical policy
  records rather than hidden runtime state
- policy certification bundles and proof tests satisfying Milestone 11 suites
  16 through 18

## Shipped Scope

Milestone 11 delivered:

- a dedicated bridge-owned `policy/` subsystem split across `admission`,
  `counters`, `contracts`, `declaration`, `lowering`, `provenance`, `replay`,
  `rejection`, `report`, `taxonomy`, and `validation`
- explicit runtime baseline partitioning for execution, diagnostics, and
  artifact policy in `BridgeRuntimePolicy`
- canonical `BridgePolicyDeclaration` and
  `ValidatedBridgePolicyDeclaration` surfaces with explicit request kind,
  execution class, diagnostics tier, replay-artifact requirement, and
  route-artifact requirement
- typed validation-stage rejection for self-conflicting policy declarations and
  typed admission-stage rejection for baseline-incompatible policy declarations
- canonical admitted policy contracts with source-localized resolution entries
  and stable digests
- one lowered execution-policy artifact and one route-planning-policy
  projection, consumed by route planning and bulk planning without reopening
  policy legality inside the executor
- canonical policy provenance records and replay bundles that include real
  resolution-entry content in their identity basis
- policy counters and provenance reports derived from bridge-owned canonical
  artifacts rather than harness-invented arithmetic
- route-policy digests propagated through route scope, planning proofs,
  delivery summaries, route diagnostics, canonical route records, and canonical
  replay
- diagnostics explanation surfaces for policy provenance reports and typed
  policy rejections
- machine-checkable policy certification bundles for provenance equivalence,
  typed rejection, and ambient leak resistance
- hardened compile-fail boundary tests keeping builder lifecycle and phase
  boundaries mechanically enforced

## Acceptance Mapping

Milestone 11 is considered closed against the roadmap, the engineering spec,
and `test-requirements.md` because the required acceptance surfaces are now
covered directly.

### `Identical policy inputs preserve meaning across builder-order, replay, and host variation`

Covered by:

- `facade::tests::policy_phase2::runtime_policy_provenance_is_stable_for_same_inputs`
- `facade::tests::policy_phase2::policy_contract_digest_changes_when_resolution_entries_change`
- `facade::tests::policy_phase2::policy_provenance_digest_changes_when_resolution_entries_change`
- `builder::tests::build_policy_sections_are_order_invariant`
- `harness::tests::policy_certification::policy_provenance_equivalence_bundle_is_builder_order_and_replay_stable`

What is proven:

- identical declarations against equivalent baselines produce identical
  contract, lowering, and provenance identities
- materially different resolution outcomes change canonical policy identity
- builder section order does not change baseline policy meaning
- replay reconstructs the same policy-scoped meaning from canonical records
  alone
- suite 16 bundles emit canonical machine-checkable policy digest, provenance,
  request-policy matrix, route-policy matrix, routing digest, replay digest,
  diagnostics digest, and counter snapshot

### `Illegal policy combinations fail typed before execution and do not degrade into fallback`

Covered by:

- `facade::tests::policy_phase2::runtime_rejects_optimized_authoritative_policy_requests`
- `facade::tests::policy_phase2::runtime_rejects_replay_without_route_artifacts`
- `facade::tests::policy_phase2::runtime_rejects_replay_with_minimal_diagnostics`
- `facade::tests::policy_phase2::runtime_rejects_replay_when_baseline_forbids_replay_artifacts`
- `harness::tests::policy_certification::policy_rejection_bundle_stays_typed_and_leaves_zero_fallback_residue`

What is proven:

- self-conflicting declarations fail in validation with typed rejection stage
  and localized policy fields
- baseline-incompatible declarations fail in admission rather than being
  silently narrowed into legal execution
- rejection bundles remain typed and explicit across host variation and replay
  lanes
- suite 17 bundles keep failure meaning machine-checkable through
  `failure_digest`, typed rejection rows, and counter snapshots with zero false
  fallback

### `Requests remain request-scoped and do not inherit policy from ambient runtime history`

Covered by:

- `facade::tests::policy_phase2::policy_admission_remains_structurally_distinct_from_truth_view_policy_resolution`
- `facade::tests::policy_phase2::runtime_projects_route_planning_policy_and_stamps_planned_route`
- `facade::tests::policy_phase2::bulk_route_planning_policy_is_carried_by_every_planned_route`
- `harness::tests::policy_certification::ambient_policy_leak_resistance_bundle_preserves_preview_equivalence_under_interleave`

What is proven:

- truth-view policy resolution and bridge request-policy admission remain
  distinct architectural paths
- route planning consumes explicit route-policy projection rather than ambient
  runtime mode
- semantically equivalent preview requests preserve the same semantic policy
  and route-policy meaning under interleave
- semantically different requests remain distinguishable in the same
  certification bundle
- suite 18 bundles prove request-scoped policy meaning through canonical
  request-policy matrices, replay digest, diagnostics digest, and exact policy
  counters

### `Replay and diagnostics derive from canonical policy records rather than live runtime inspection`

Covered by:

- `facade::tests::policy_phase2::policy_scoped_route_round_trips_through_canonical_replay`
- `harness::tests::routing::replay_parity::replayed_policy_scoped_route_preserves_route_policy_digest_in_route_record`
- `diagnostics::facade::policy` surfaces
- `harness::tests::policy_certification::*`

What is proven:

- canonical replay can reconstruct route-policy-scoped planning meaning from
  route records and route proofs alone
- route-policy digest is preserved through delivery, record retention, and
  replay
- policy explanations are derived from canonical policy artifacts and typed
  rejections rather than live runtime rediscovery

## Additional Hardening Added Before Close

Milestone 11 closeout includes these extra hardening outcomes beyond the
minimum phase labels:

- validation and admission were separated honestly rather than cosmetically:
  self-conflict declaration checks now fail in validation, while runtime
  baseline incompatibility fails in admission
- rejection artifacts now carry explicit stage identity so diagnostics and
  certification can distinguish where legality failed
- canonical contract and provenance digests were hardened to include actual
  resolution-entry content rather than only entry counts
- route replay was hardened so policy-bearing replay reconstructs through
  canonical route-policy records instead of reusing the normal live-baseline
  compatibility path
- policy counters were moved away from harness-invented arithmetic into
  bridge-owned derivation helpers over declarations, contracts, provenance, and
  replay bundles
- rejection certification was tightened to prove both validation-stage and
  admission-stage failures instead of two variants of the same validation story
- policy-scoped route replay tests were aligned with the explicit artifact
  retention contract so canonical retention occurs only when the policy
  requests route artifacts
- compile-fail goldens were refreshed so the phase-boundary privacy suite keeps
  enforcing the actual current builder state topology

These changes were made because the closeout bar was not "policy seems to work."
The closeout bar was explicit request-scoped policy semantics, replay-safe
canonical policy truth, proof-grade rejection surfaces, honest counter
derivation, and architectural separation strong enough to keep the bridge from
degrading into a second scheduler or an ambient mutable policy bag.

## Explicit Non-Goals Preserved

Milestone 11 intentionally does not include:

- bridge-mediated writeback policy or authority expansion
- signal-runtime scheduling ownership inside the bridge
- policy inference from prior requests, hidden globals, or diagnostics history
- executor-side legality rediscovery after lowering
- a loosely mutable global "current bridge mode"

The bridge now owns policy declaration, legality, lowering, provenance,
request-scoped route-policy projection, and replay-safe explanation.
It does not own downstream scheduling semantics and it does not redefine truth
authority.

## Verification

Closeout verification passed with:

- `cargo test -p worth-runtime-bridge`
- `cargo test -p worth-runtime-bridge policy_phase2 -- --nocapture`
- `cargo test -p worth-runtime-bridge policy_certification -- --nocapture`

No remaining meaningful in-scope QA findings were left open at close.
