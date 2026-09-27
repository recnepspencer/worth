# Milestone 9.3.6 Closeout: Lower-Runtime Capability Routing And Boundary Envelope Closure

## Status

Milestone 9.3.6 is closed as of 2026-05-19 for the Query-owned
lower-runtime capability routing and boundary-envelope lifecycle in
`worth-query`.

This closeout covers:

- one executable crossing inventory for every covered Query-to-lower-runtime
  seam, with explicit seam classification, route kind, authority owner,
  artifact strength, and support posture
- typed capability request, eligibility, route-plan or readmission-handoff,
  boundary-execution receipt, and boundary-envelope progression for the full
  covered seam set
- elimination or closure of the in-scope specialist seams that entered this
  milestone as concrete debt, including historical lowering, effect-backed
  mutation and merge execution, bridge writeback, frontier evidence intake,
  and runtime intent authority execution
- public-surface and internal non-bypass closure over the routed facade and
  boundary adapter surface, with compile-fail enforcement for sealed routing
  and certification artifacts
- runtime-backed representative certification coverage for the covered
  Phase 6 seam set, with zero remaining synthetic tail inside the lower-runtime
  routing certification surface
- the named `9.3.6. Lower-Runtime Capability Routing And Boundary Envelope
  Closure Test`, a public phase artifact manifest, and a public stabilization
  closeout report that binds the final certification bundle, acceptance suite,
  boundary reconciliation report, proof-shape audit, and synthetic-tail report
- Phase 7 exact counter and slope honesty, including:
  - full-profile counter snapshots bound to real emitted certification surface
    widths
  - family-specific slope digests that bind only to the relevant route width,
    evidence width, support width, or deferred-neighbor width
  - hostile proof that unrelated breadth drift does not move those family
    digests

This closeout does not claim store-backed route parity, durable route replay,
persisted boundary receipts, restart-stable boundary-envelope reload, temporal
query-basis routing, async/resource routing, or mixed truth/time/async routing.
Those remain explicit deferred neighbors exactly as the 9.3.6 spec declared.

## Governing Source Summary

- `MENTALITY.md`: closure required executable truth, named adversarial proof,
  and no ceremonial â€œclose enoughâ€ routing surface.
- `arch_laws.md`: closure required a typed proof-bearing lifecycle, sealed
  construction, honest boundary envelopes, and stabilization-facing artifacts
  that consume the same proofs the runtime exports.
- `domain_structure_laws.md`: closure required physically real subdomains for
  inventory, support, eligibility, plans, receipts, boundary certification,
  performance, phase manifest, and closeout artifacts.
- `perf_laws.md`: closure required exact counters and slope digests that bind
  to route width, evidence width, support width, and deferred-neighbor width
  rather than unrelated runtime breadth.
- `milestone-9.3.6.md`: the shipped surface now satisfies the locked seam
  inventory, phase contract, named outputs, required topology, and stabilization
  closeout standard.

## Adversarial Constraint Closed

Milestone 9.3.6 had to survive Query callers and internal modules attempting to:

- reach lower-runtime bridge, relational, signal, or historical seams through
  convenience imports rather than one typed routed lifecycle
- report support or closure posture that diverged from the executable seam
  inventory
- keep specialist seam debt alive after the routing lifecycle was installed
- certify a smaller public/internal boundary surface than the real routed
  facade
- treat slope and counter honesty as a symbolic digest problem instead of a
  proof about the real emitted certification surface
- make the runtime stabilization gate reassemble 9.3.6 from internal modules
  instead of consuming one explicit public closeout artifact

The closed surface now enforces one shared progression for covered
Query-to-lower-runtime contact:

1. `LowerRuntimeCrossingInventory`
2. `LowerRuntimeCapabilityRequest`
3. `LowerRuntimeCapabilityEligibility`
4. `LowerRuntimeRoutePlan | LowerRuntimeReadmissionReceipt`
5. `LowerRuntimeBoundaryExecutionReceipt`
6. `LowerRuntimeBoundaryEnvelope`
7. `LowerRuntimeSupportMatrix`
8. `LowerRuntimeCloseoutRegistry`
9. `LowerRuntimeBoundaryCertificationBundle`
10. `LowerRuntimeClosureTest`
11. `LowerRuntimePhaseManifest`
12. `LowerRuntimeCloseoutReport`

The public runtime-stabilization gate now consumes the explicit closeout report
instead of re-deriving this lifecycle from internal modules.

## Phase Closure

Phase 1 closed with:

- the locked crossing inventory and direct-import audit rows
- explicit classification of every in-scope seam
- compile-visible seam ownership, route kind, artifact strength, and
  authority-owner metadata

Phase 2 closed with:

- elimination of the former transition-only runtime-intent seam
- support and closeout posture that no longer depended on hidden transition
  logic
- executable import-audit closure for the post-transition topology

Phase 3 closed with:

- collapse of the specialist seam debt into explicit lower-runtime contracts or
  typed adapter rows
- no remaining compatibility-debt lane for in-scope covered rows at closeout

Phase 4 closed with:

- typed `CapabilityRequest`, `CapabilityEligibility`, `RoutePlan` or
  `ReadmissionReceipt`, `BoundaryExecutionReceipt`, and `BoundaryEnvelope`
  progression
- bridge-backed and relational-backed seams emitting the same proof-bearing
  boundary lifecycle instead of ad hoc weak returns

Phase 5 closed with:

- support-matrix closure and deferred-neighbor registry closure
- explicit owner, missing contract or later milestone, required closeout, and
  certification row for every remaining deferred seam

Phase 6 closed with:

- public/internal boundary reconciliation across the full routed facade and
  adapter surface
- runtime-backed representative certification for the covered seam set
- zero remaining synthetic tail inside the lower-runtime routing certification
  surface
- compile-fail, non-bypass, parity, proof-shape, and boundary-audit closure

Phase 7 closed with:

- the named `9.3.6. Lower-Runtime Capability Routing And Boundary Envelope
  Closure Test`
- exact certification output manifest and stabilization-facing extension output
  manifest
- public phase artifact manifest and typestate-transition digest
- public stabilization closeout report
- exact counter and slope honesty with hostile proof that irrelevant breadth
  drift does not move the family-specific slope digests

## Verification Summary

The closeout surface was validated with focused lower-runtime routing proof
commands, including:

- `cargo fmt -p worth-query`
- `cargo check -p worth-query`
- `cargo test -p worth-query --lib lower_runtime_routing::certification::phase_manifest::tests::phase_manifest_names_every_closeout_artifact_in_order -- --exact --nocapture`
- `cargo test -p worth-query --lib lower_runtime_routing::certification::tests::stabilization_closeout_report_is_public_and_consumes_final_phase_artifacts -- --exact --nocapture`
- `cargo test -p worth-query --lib lower_runtime_routing::certification::performance::tests::scenario_profiles_are_monotonic_across_width_variants -- --exact --nocapture`
- `cargo test -p worth-query --lib lower_runtime_routing::certification::performance::tests::full_profile_counter_snapshot_matches_exact_producer_widths -- --exact --nocapture`
- `cargo test -p worth-query --lib lower_runtime_routing::certification::performance::tests::slope_digests_ignore_unrelated_width_drift -- --exact --nocapture`
- `cargo test -p worth-query --lib lower_runtime_routing::certification::tests::certification_bundle_emits_required_outputs -- --exact --nocapture`
- `cargo test --manifest-path crates/worth-query/Cargo.toml --test phase_boundaries_lower_runtime_routing_compile_fail -- --nocapture`

## Handoff To Runtime API Public Stabilization Gate

The runtime API public stabilization gate now inherits:

- one closed lower-runtime contact model instead of scattered convenience seams
- one public closeout report that binds the final routing certification bundle,
  named closure test, phase manifest, acceptance suite, boundary
  reconciliation, and synthetic-tail report
- exact output names and digests for the stabilization gate to consume directly
  instead of re-deriving internal routing state
- explicit deferred-neighbor rows for the later store/durable/temporal/async
  routing work that 9.3.6 intentionally did not close

The stabilization gate must not reopen 9.3.6â€™s routing-shape or seam-coverage
questions. It should consume the shipped closeout artifacts as the final
lower-runtime routing authority for the frozen public runtime facade.
