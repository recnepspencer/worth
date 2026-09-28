# Milestone 9.3.7 Closeout: Domain Capability Contributions And Canonical Runtime Materialization

## Status

Milestone 9.3.7 is closed as of 2026-05-22 for the Query-owned
domain-capability contribution seam in `worth-query`.

This closeout covers:

- one proof-bearing contribution lifecycle from request through eligibility,
  admitted, materialization-ready, and canonical runtime materialization
- category-complete contribution coverage for:
  - admission
  - support and traceability
  - invariants and capability posture
  - workflow and preview
  - continuity
  - aftermath
  - explanation
- Query-owned canonical runtime materializers and category-specific hooks that
  keep canonical runtime artifacts Query-owned even when semantic posture is
  downstream-domain-authored
- an ordinary invariant-registration facade on `WORTHQueryRuntime::builder()`
  that lowers into relational invariant authority without making relational
  builder plumbing the ordinary downstream path
- a closed Phase 6 public-lane surface with common, checked, proof, and raw
  lanes, compile-checked goldens, compile-fail boundaries, and hostile
  certification
- categorized crate-local feature documentation under
  `crates/worth-query/docs/domain-capabilities/...`, with one feature doc per
  feature and doc QA that closed the remaining "lost to history" risks

This closeout does not claim store-backed domain-capability replay, durable
contribution archives, restart-stable contribution reload, temporal query-basis
capability contribution, async/resource capability contribution, or mixed
truth/time/async contribution delivery. Those remain later-milestone scope
exactly as the 9.3.7 spec declared.

## Governing Source Summary

- `MENTALITY.md`: closure required one reusable platform seam rather than
  domain-local folklore adapters.
- `arch_laws.md`: closure required Query-owned canonical artifact authority,
  typed lane degradation, non-bypass public surfaces, and honest category
  distinctions.
- `composition_laws.md`: closure required distinct ownership for authoring,
  eligibility, materialization, DX, certification, and docs rather than one
  giant bag.
- `domain_structure_laws.md`: closure required physically real contribution
  subdomains plus category-owned workflow, continuity, aftermath, explanation,
  and certification surfaces.
- `perf_laws.md`: closure required live width and slope evidence bound to the
  executable certification bundle rather than symbolic placeholder math.
- `milestone-9.3.7.md`: the shipped surface now satisfies the locked phase
  order, category coverage, proof/foundational integration, required topology,
  required verification outputs, and closeout standard.

## Adversarial Constraint Closed

Milestone 9.3.7 had to survive downstream domains attempting to:

- mint pseudo-Query runtime artifacts locally instead of contributing semantic
  posture through Query
- flatten advisory, support, workflow, continuity, aftermath, or explanation
  meaning into strings or ad hoc JSON
- bypass the ordinary public lane by reaching crate-private or lower-lane
  artifact construction directly
- leave one or more named categories in a "typed but not really finished"
  state
- certify only manifests and inventories instead of live canonical artifacts,
  compile-fail boundaries, and width-aware closeout evidence
- write docs that still required the milestone spec or oral history to find the
  real stable entry points

The closed surface now enforces one shared progression:

1. `DomainCapabilityContributionRequest`
2. `DomainCapabilityContributionEligibility`
3. `AdmittedDomainCapabilityContribution`
4. `CanonicalRuntimeMaterialization`
5. category-specific canonical Query artifacts, reports, reviews, or support
   rows
6. `DomainCapabilityCertificationSurface`
7. `DomainCapabilityCertificationBundle`
8. categorized crate-local feature docs

## Phase Closure

Phase 1 closed with:

- real `worth-proof` contribution progression
- sealed stronger forms and compile-fail boundaries preventing direct minting
- target-family typing across declaration-bound, admitted-plan-bound, and
  lower-runtime-bound contribution lanes
- the first shipped Query-owned target-binding substrate slice that later
  `9.3.8` phases should generalize rather than duplicate

Phase 2 closed with:

- real `worth-foundational` descriptive row, provenance, and profile
  integration
- explicit freshness/provenance mapping on descriptive outputs instead of one
  flattened default posture

Phase 3 closed with:

- canonical Query artifact materialization across every named category
- category-specific runtime hooks where generic canonical wrappers were not
  enough
- closure of the fake-public-posture gaps in support breadth, discard-required
  workflow, support-only admission, and invariant registration

Phase 4 closed with:

- the ordinary Query invariant-registration facade on
  `WORTHQueryRuntime::builder()`
- lowering into relational invariant authority without mixed-authority
  overwrite holes

Phase 5 closed with:

- category-complete public/runtime-facing coverage for workflow, continuity,
  aftermath, explanation, and invariant/capability integration
- replacement of synthetic preview/runtime-preflight shortcuts where the real
  authority path now exists

Phase 6 closed with:

- one honest common lane, checked lane, proof lane, and raw lane across the
  named categories
- compile-checked golden transcripts, DX compile-fail boundaries, and
  certification compile-fail boundaries
- executable certification surfaces proving parity, distinction, no-bypass, and
  live width/slope evidence
- a synchronized certification inventory and public-lane closeout harness

Phase 7 closed with:

- category-organized crate docs under
  `crates/worth-query/docs/domain-capabilities`
- one feature doc per feature instead of an omnibus milestone dump
- doc QA that closed discoverability gaps, removed milestone-history fluff, and
  documented the sharper proof/raw lanes where they are the honest first step

## Verification Summary

The closeout surface was validated through the milestone's integrated Phase 6
gate and the final Phase 7 doc QA pass, including:

- `cargo fmt -p worth-query`
- `cargo test -p worth-query domain_capabilities::certification_closeout_tests --quiet`
- `cargo test -p worth-query --test phase_boundaries_domain_capabilities_dx_compile_fail --quiet`
- `cargo test -p worth-query --test phase_boundaries_domain_capabilities_certification_compile_fail --quiet`
- `cargo test -p worth-query --quiet`

## Handoff To Runtime API Public Stabilization Gate

The runtime API public stabilization gate now inherits:

- one closed domain-capability contribution seam instead of category-local
  pseudo-Query adapters
- one closed ordinary/common lane that downstream geometry and workflow domains
  can build on without knowing the proof substrate first
- one certification surface, boundary digest, representative report, and slope
  report that the stabilization gate can consume directly
- one categorized crate-doc surface that teaches the same ordinary, checked,
  proof, and raw lanes the code and certification actually export
- one typed contribution target-binding seam that later `9.3.8` declaration,
  orchestration, continuation, and ergonomic phases should broaden into the
  shared Query binding substrate rather than re-solving locally
- the first of those broadenings is now the immediate post-Phase-24 shared
  retained target-binding extraction rather than a distant cleanup, followed by
  the explicit declaration-entry aspect-contract/granularity addendum and the
  now-shipped Phase 25 typed binding / extractor / resolver pipeline, the
  now-shipped Phase 26 denial-preserving ordinary outcome layer, and the
  now-shipped Phase 27 prepared/executed continuation pipeline, the
  now-shipped Phase 28 signal-compatibility orchestration seam, and the
  now-shipped Phase 29 contribution-composed orchestration seam, the
  now-shipped Phase 30 orchestration inventory / anti-drift audit seam, and
  the now-shipped Phase 31 denial-and-recovery boundary, and the now-shipped
  Phase 32 family-helper projection seam, and the now-shipped Phase 33
  grouped-authoring seam, and the now-shipped Phase 34 public-doc-coverage
  seam, and the now-shipped Phase 35 platform-entry closeout seam; later
  widening should extend those shipped seams
  phase by phase with in-flight feature-doc
  synchronization and inventory/certification synchronization rather than as
  one final documentation pass

The stabilization gate must not reopen 9.3.7's category-coverage, public-lane,
or certification-honesty questions. It should consume the shipped
domain-capability seam as the closed precondition the 9.3.7 spec required.
