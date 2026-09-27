# Milestone 9.3.1 Closeout: Cross-Runtime Causal Diagnostics And Query Inspection

## Status

Milestone 9.3.1 is closed as of 2026-05-12 for the runtime-backed
cross-runtime causal diagnostics and Query inspection surface in `worth-query`
and `worth-runtime-bridge`.

This closeout covers:

- Query-owned causal observation anchors, evidence references, inspection
  requests, admission decisions, redaction policy, materialization policy, and
  public causal inspection artifacts
- runtime-bridge-owned causal explanation envelopes, evidence bindings,
  receipts, denials, retained-record lookup, and facade assembly
- admitted, advisory, denied, missing-evidence, redaction, representative
  matrix, proof-shape, boundary-audit, and performance certification rows
- compile-fail boundaries preventing external construction of causal proof
  artifacts and preventing `worth-query` from exporting bridge-owned causal
  constructors
- final Phase 6 hardening proving bridge envelope assembly has its own named
  slope digest and drift rejection

This closeout does not claim durable causal archives, store-backed replay
reconstruction, restart-stable expanded explanation reload, or persisted causal
narrative materialization. Those remain later-milestone debt exactly as
Milestone 9.3.1 specified.

## Governing Source Summary

- `MENTALITY.md`: closure required the adversarial version first: one
  machine-checkable explanation path for changed, suppressed, denied, redacted,
  branch/preview, and replay observations rather than debug prose.
- `arch_laws.md`: closure required self-describing boundary envelopes,
  proof-bearing phase progression, typed denial, and facade-only access across
  lower-runtime boundaries.
- `composition_laws.md`: closure required honest ownership: Query owns public
  inspection and materialization; runtime bridge owns bridge causal envelope
  authority; relational and signal remain authority surfaces for their
  evidence.
- `domain_structure_laws.md`: closure required causal inspection, bridge
  envelope authority, materialization, certification, and boundary audit to
  remain visible as separate responsibilities.
- `perf_laws.md`: closure required diagnostic richness to stay cold-path and
  all claimed slopes to have exact counters and certification rows.
- `milestone-9.3.1.md`: the shipped runtime-backed surface now satisfies the
  phase progression, forbidden-debt, acceptance evidence, and closeout standard.

## Adversarial Constraint Closed

Milestone 9.3.1 had to survive a downstream domain asking why a
query-observed artifact changed, did not change, was denied, was redacted, or
was replayed, while still preventing that domain from stitching together
runtime-bridge diagnostics, relational internals, and signal graph internals
directly.

The closed surface enforces this through one typed progression:

1. `QueryObservationReceipt`
2. `CausalObservationAnchor`
3. `CausalEvidenceReferenceSet`
4. `CausalInspectionRequest`
5. `CausalInspectionProofFlow`
6. runtime-bridge `BridgeCausalExplanationEnvelope` or typed bridge denial
7. `QueryCausalInspectionArtifact`
8. `CausalInspectionCertificationBundle`

Query may request, admit, redact, materialize, and certify causal explanations.
It does not define bridge causal envelope authority, retained bridge indexes,
or bridge evidence bindings. Bridge-owned envelope assembly lives in
`worth-runtime-bridge` and is reached through the runtime-bridge facade.

## Shipped Scope

Milestone 9.3.1 delivered:

- causal observation receipts, anchors, requests, admission, evidence
  references, reference resolution, and proof flow in
  [crates/worth-query/src/runtime/inspection/causal](../../crates/worth-query/src/runtime/inspection/causal)
- Query-owned materialized admitted, advisory, and denied causal artifacts in
  [crates/worth-query/src/runtime/inspection/causal/materialization](../../crates/worth-query/src/runtime/inspection/causal/materialization)
- Phase 6 certification scope, bundle, representative matrix, proof-shape
  certification, boundary audit, row digest inventory, failure evidence, and
  performance certification in
  [crates/worth-query/src/runtime/inspection/causal/certification](../../crates/worth-query/src/runtime/inspection/causal/certification)
- bridge-owned causal envelope authority, evidence references, bindings,
  counters, denials, identity, receipts, retained mapping, and facade assembly
  in
  [crates/worth-runtime-bridge/src/diagnostics/causal_envelope](../../crates/worth-runtime-bridge/src/diagnostics/causal_envelope)
- runtime-bridge facade exposure for bridge causal envelope assembly in
  [crates/worth-runtime-bridge/src/diagnostics/facade/query.rs](../../crates/worth-runtime-bridge/src/diagnostics/facade/query.rs)
- Query causal inspection tests in
  [crates/worth-query/src/runtime/tests/causal_inspection](../../crates/worth-query/src/runtime/tests/causal_inspection)
- runtime-bridge causal envelope tests in
  [crates/worth-runtime-bridge/src/facade/tests/causal_envelope](../../crates/worth-runtime-bridge/src/facade/tests/causal_envelope)
- compile-fail proof boundaries in
  [crates/worth-query/tests/ui](../../crates/worth-query/tests/ui) and
  [crates/worth-runtime-bridge/tests/ui/causal_envelope](../../crates/worth-runtime-bridge/tests/ui/causal_envelope)

## Acceptance Mapping

Milestone 9.3.1 is considered closed against:

- [milestone-9.3.1.md](./milestone-9.3.1.md)
- [worth_query_roadmap.md](./worth_query_roadmap.md)
- [worth_query_vision.md](./worth_query_vision.md)
- [test-requirements.md](./test-requirements.md)
- [test-requirements-milestone-9_3-and-runtime-gates.md](./test-requirements-milestone-9_3-and-runtime-gates.md)
- [milestone-9.3.md](./milestone-9.3.md)

because Query inspection now exposes one authority-preserving public causal
explanation surface backed by a bridge-owned envelope and closed by
machine-checkable certification artifacts.

### `Cross-Runtime Causal Explanation Envelope Test`

Covered by:

- [crates/worth-query/src/runtime/tests/causal_inspection](../../crates/worth-query/src/runtime/tests/causal_inspection)
- [crates/worth-query/src/runtime/tests/causal_inspection/certification](../../crates/worth-query/src/runtime/tests/causal_inspection/certification)
- [crates/worth-runtime-bridge/src/facade/tests/causal_envelope](../../crates/worth-runtime-bridge/src/facade/tests/causal_envelope)
- [crates/worth-query/tests/ui](../../crates/worth-query/tests/ui)
- [crates/worth-runtime-bridge/tests/ui/causal_envelope](../../crates/worth-runtime-bridge/tests/ui/causal_envelope)

What is proven:

- changed, suppressed, denied, advisory-redacted, branch/preview, historical
  replay, policy-redacted, Worth-style query-only, lower-runtime evidence, and
  scale-honesty lanes are represented in the Phase 6 matrix
- missing bridge route evidence, missing signal invalidation evidence, missing
  signal evaluation evidence, relational authority mismatch, redaction policy
  overclaim, unsupported explanation family, direct lower-runtime explanation
  paths, durable causal archive overclaim, and store-backed replay
  reconstruction overclaim are typed rejection rows
- public Query artifacts carry lower-runtime authority names and evidence
  digests without flattening bridge, relational, or signal ownership into a
  Query-owned narrative
- runtime bridge owns envelope construction, retained bridge lookup, evidence
  binding, denial, identity, receipt, and counter authority
- Query materialization consumes sealed bridge facade envelope results and
  never reconstructs bridge causal facts from retained bridge diagnostics
- proof-shape certification rejects phase skipping, raw collection
  substitution, stale proof reuse, and WORTHd authority witnesses
- compile-fail boundaries prevent external construction of proof-bearing
  Query artifacts and bridge causal envelope internals

## Final Audit Finding Closed

The final closeout audit found one Phase 6 gap: bridge envelope assembly slope
was recorded in `CausalInspectionScaleCounterSnapshot`, but certification did
not yet enforce that slope or surface `causal_bridge_envelope_slope_digest` on
the final certification bundle.

That gap is closed by:

- enforcing `bridge_envelope_slope_counter() == 1` in certification validation
- surfacing named scale, anchor-derivation, reference-resolution, admission,
  bridge-envelope, materialization, and artifact-serialization slope digests on
  the performance certification bundle, certification scope, and final
  certification bundle
- adding a hostile test that mutates bridge-envelope slope to `2` and proves
  certification rejects it as `ScaleSlopeDrift`
- updating the private-field compile-fail fixtures so the new slope digest
  fields remain non-constructible by external callers

## Closeout Standard

The Milestone 9.3.1 closeout standard is satisfied because:

- the named 9.3.1 certification suite exists and passes
- Query exposes public causal inspection artifacts for admitted, advisory, and
  denied explanations
- bridge-owned causal envelopes are implemented in `worth-runtime-bridge`,
  returned through the runtime-bridge facade, and consumed by Query as sealed
  facade results
- bridge envelopes bind route/evaluation/source/structural/stream/preview/
  writeback/replay evidence, relational authority references, signal
  invalidation/evaluation references, forensic availability, lineage,
  provenance, replay posture, and materialization evidence
- redaction and richness policy remain cold-path materialization concerns
- Worth-style ordinary consumer explanations use Query causal artifacts instead
  of direct lower-runtime stitching
- compile-fail boundaries prevent public construction of proof-bearing causal
  inspection and bridge causal envelope artifacts
- proof-shape certification covers phase skipping, raw collection
  substitution, stale proof reuse, and WORTHd authority witnesses
- performance certification proves anchor derivation, reference resolution,
  admission, bridge envelope assembly, materialization, and artifact
  serialization each obey named slope counters
- durable/store-backed causal replay claims remain explicit later-milestone
  debt

## Verification Baseline

The closeout state is verified by:

- `cargo fmt --check -p worth-query`
- `cargo fmt --check -p worth-runtime-bridge`
- `cargo test -p worth-query runtime::tests::causal_inspection::certification --quiet`
- `cargo test -p worth-query runtime::tests::causal_inspection --quiet`
- `cargo test -p worth-runtime-bridge causal_envelope --quiet`
- `cargo test -p worth-query --test phase_boundaries_compile_fail --quiet`
- `cargo test -p worth-query --quiet`
- `cargo test -p worth-runtime-bridge --quiet`
- `git diff --check`

## Deferred Scope That Remains Explicitly Deferred

The following are not part of Milestone 9.3.1 closeout:

- durable causal explanation archives
- persisted expanded narratives
- store-backed replay reconstruction
- restart-stable expanded explanation reload
- cross-process causal-envelope reload
- durable/store-backed causal artifact materialization

These remain Milestone 10, Milestone 11, or later diagnostic-certification
scope exactly as the milestone spec declared.
