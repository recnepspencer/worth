# Milestone 9.3.5 Closeout: Intent Admission Decision Lattice And Decision Trace

## Status

Milestone 9.3.5 is closed as of 2026-05-18 for the Query-owned intent
admission decision lattice in `worth-query`.

This closeout covers:

- phase-typed progression from raw intent through eligibility, admitted,
  advisory, and violation decisions, admitted plans, admitted execution
  handoffs, decision traces, and certification
- concrete covered-family adoption for authoritative mutation, batch mutation,
  effect-triggered pending write intents, read-family execution,
  basis-context read execution, live read execution, derived materialization,
  generic and derived inspection, existing-truth probe routing, basis
  observation, and projection consumption
- compile-visible family inventory, support matrix, coverage inventory,
  surface catalog, mutation-entrypoint audit, and representative-family
  certification artifacts
- canonical common-path and advanced-path DX for the covered families, with
  thin convenience wrappers documented and certified as delegates over the same
  lattice
- bridge-backed execution bindings and provenance retention for the covered
  runtime-backed families
- compile-fail boundaries preventing external construction of proof-bearing
  family, decision, handoff, trace, audit, and certification artifacts
- phase-six certification closure, including public-boundary, proof-shape,
  topology, oracle, parity, support-traceability, slope, output-manifest, and
  crate-doc example audits
- phase-seven crate documentation installed into the categorized
  `crates/worth-query/docs` tree with compile-checked examples and certification
  linkage

This closeout does not claim durable decision archives, restart-stable trace
reload, store-backed admission replay, temporal query-basis admission, async or
resource-backed admission, or mixed truth/time/async delivery admission. Those
remain later-milestone scope exactly as the 9.3.5 spec declared.

## Governing Source Summary

- `MENTALITY.md`: closure required one honest fail-first admission surface,
  not prettier post-hoc errors.
- `arch_laws.md`: closure required rejection before expensive work, sealed
  phase progression, typed handoffs, and provenance-carrying execution.
- `composition_laws.md`: closure required separate ownership for family
  inventory, eligibility, decisions, handoffs, trace, DX, support, and
  certification.
- `domain_structure_laws.md`: closure required the `intent_admission`
  subdomains to be physically real and certification-owned topology to match
  the shipped tree.
- `perf_laws.md`: closure required exact counters and slope evidence for family
  lookup, eligibility resolution, trace assembly, and certification coverage.
- `milestone-9.3.5.md`: the shipped surface now satisfies the phase contract,
  product decision lock, required verification outputs, and closeout standard.

## Adversarial Constraint Closed

Milestone 9.3.5 had to survive Query-crossing callers attempting to:

- treat admission as a binary `Result`
- reach covered execution without a typed admitted handoff
- flatten advisory, deferred, and violation postures into one generic failure
- reconstruct basis, routing, or projection posture from lower-runtime
  artifacts after execution
- document or certify a smaller runtime floor than the real post-5.5 covered
  family set

The closed surface now enforces one shared progression:

1. `RawIntent`
2. `IntentEligibility`
3. `AdmissionDecision`
4. `AdmittedIntentPlan | AdvisoryDecision | ViolationDecision`
5. `AdmittedExecutionHandoff | AdvisoryStop | ViolationStop`
6. `DecisionTraceEnvelope`
7. `IntentAdmissionCertificationBundle`

For covered runtime-backed families, execution now consumes admitted handoffs
and bindings with retained provenance. For admitted non-runtime families, the
same lattice terminates honestly in scoped basis or bound-contract artifacts
instead of speculative pseudo-execution.

## Phase Closure

Phase 1 closed with:

- compile-visible family inventory, support matrix, coverage inventory, and
  surface catalog
- honest covered versus deferred posture for the declared 9.3.5 floor

Phase 2 closed with:

- real eligibility artifacts carrying pre-decision facts
- decision code consuming eligibility instead of rediscovering from raw input

Phase 3 closed with:

- family-specific admitted plans, handoffs, and terminal stop artifacts
- compile-fail sealing for admitted and non-admitted proof-bearing artifacts

Phase 4 closed with:

- binding-driven covered execution
- provenance retention across success, denial, and routing-failure paths

Phase 5 and 5.5 closed with:

- adoption of the concrete post-5.5 family set
- mutation-entrypoint delegation audit
- direct bundle-facing support for basis and projection families
- synchronized inventory, support, DX, and doc-example surfaces

Phase 6 closed with:

- the named 9.3.5 certification suite
- exact output manifest, topology, parity, oracle, slope, and doc-example
  audits
- hostile compile-fail boundaries and observational-parity checks

Phase 7 closed with:

- categorized crate documentation under `crates/worth-query/docs`
- one capability doc per major public feature surface
- product-facing intent-admission documentation aligned with the post-5.5
  covered family set and certified compile-checked examples

## Verification Summary

The closeout surface was validated with:

- `cargo fmt --all`
- `cargo test -p worth-query public_doc_wording -- --nocapture`
- `cargo test -p worth-query certification -- --nocapture`
- `cargo test -p worth-query --test phase_boundaries_intent_admission_compile_fail -- --nocapture`
- `cargo test -p worth-query --quiet`

## Handoff To 9.3.6

Milestone 9.3.6 inherits:

- the post-5.5 covered-entrypoint inventory and support posture
- the canonical admitted handoff into already-supported runtime-backed
  execution seams
- the categorized crate-doc surface and compile-checked public DX examples
- the closed non-bypass certification and topology contracts for
  `intent_admission`

9.3.6 therefore starts from a closed admission and documentation surface. It
must not reopen the 9.3.5 question of which families are adopted or how the
shared lattice behaves.
