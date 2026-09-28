# Milestone 2 Closeout: Canonical Digest And Canonicalization Substrate

Date: 2026-05-13

## Status

Milestone 2 is implementation-complete for `worth-foundational`.

The crate now owns the shared canonicalization substrate for versioned basis
grammar, Milestone 1 surface lowering, equivalence declarations, mismatch
evidence, export fixtures, digest algorithm slots, derived digest values, and
production-test readiness evidence. The implementation preserves the rule that
canonical basis is semantic authority and digest values are derived
compression, not proof of semantic sameness by themselves.

This milestone is ready for production-shaped testing through `worth-harness`
or adopting-crate migration work. It does not claim that any real adopting
runtime lowering is already correct.

## Completed Surface

- Canonical basis grammar now has versioned rule identity, typed domains,
  typed loci, typed entry kinds, typed value carriers, entry ids, ordered
  sequences, bundles, and visible canonicalization cost counters.
- Canonical basis readiness and bundle readiness are proof-bearing
  `worth-proof` artifacts with canonical order, uniqueness, domain coherence,
  rule-version binding, and cost observation evidence.
- Milestone 1 surfaces can lower into canonical basis evidence for contracts,
  masks, authoritative state, authoritative patches, identities, locators, and
  compatibility-lowered state.
- Equivalence comparison requires an explicit `CanonicalEquivalenceBasis`
  before comparison and produces structured outcomes rather than boolean
  equality.
- Mismatch evidence is self-describing and reports canonical mismatch kinds
  and loci that blind consumers can interpret without producer-private state.
- Canonical export bundles carry manifest rows, producer shape, rule version,
  equivalence basis, entry counts, cost counters, and trust-boundary freshness
  behavior.
- Boundary-bridged exports are readable as snapshots but cannot satisfy
  current-validity APIs until readmitted by the milestone-owned authority.
- Digest algorithm slots are typed by domain, rule version, and input shape for
  single-sequence, domain-bundle, and export-bundle inputs.
- Derived digest values carry algorithm, rule-version, input-shape, input-id,
  and entry-count metadata and cannot replace declared equivalence basis
  evidence.
- Milestone 2 production-test readiness is a proof-bearing artifact naming
  certified surfaces, hostile pressures, compile-fail boundaries, exact golden
  artifacts, property seed evidence, cost evidence, `worth-harness` expansion
  points, runtime assumptions, non-assumptions, and residual debt.

## Final QA Fixes

- Removed later-milestone implementation work for profiles, boundary artifact
  materialization, and branch/merge/commit transitions so Milestone 2 closes
  only the canonicalization substrate it actually owns.
- Split canonicalization structural hotspots into responsibility-owned homes:
  canonical basis construction/grammar/readiness, equivalence basis/comparison
  readiness/outcome/mismatch search, export comparison, digest material, and
  contract preparation.
- Removed dormant later-milestone canonical-basis variants from Milestone 2
  grammar and digest materialization.
- Added `plans/worth-foundational/milestone-2-acceptance-matrix.md` as the
  final closeout audit map.
- Split readiness golden evidence from broad labels into exact surface rows for
  value families, aspect contract basis, aspect mask basis, authoritative state
  basis, authoritative patch basis, compatibility-lowered state basis,
  identity/locator basis, equivalence basis, mismatch basis, export bundle
  manifest, and digest-slot derived values.
- Added `CanonicalPropertySeedEvidence` so each property seed names its hostile
  dimension, owning certification test, and future `worth-harness` lane.
- Strengthened readiness tests so enum-only evidence cannot pass as a concrete
  production-test handoff.

## Proof Evidence

- Certification tests cover canonical basis grammar, Milestone 1 basis
  builders, equivalence and mismatch readiness, export bundles, golden
  artifacts, digest slots, production readiness, and proof carriage.
- Compile-fail tests prove raw basis sequences, raw indexes, digest-ready
  artifacts, plain phase artifacts, raw bytes, plain basis payloads, plain
  digest slots, boundary-bridged exports, ordinary reports, prose closeouts,
  and caller-minted readiness authorities cannot satisfy proof-bearing APIs.
- Golden artifacts compare semantic canonical basis evidence rather than debug
  output, display text, transport JSON, or fixture names.
- Hostile producer tests prove construction order, insertion order, local map
  layout, compatibility-origin ordering, adjacent value categories, and digest
  input-shape differences do not collapse canonical meaning.
- Readiness inventory tests prove exact coverage for certified surfaces,
  compile-fail evidence, golden artifacts, property seed evidence, phase gates,
  cost evidence, fixture manifests, `worth-harness` expansion points,
  assumptions, non-assumptions, and debt.
- Topology checks show no `worth-foundational` source or test file over 400
  lines and no source/test directory over 10 direct Rust files.

## Verification

The final QA pass ran:

```powershell
cargo fmt -p worth-foundational
cargo test -p worth-foundational
cargo clippy -p worth-foundational --all-targets --no-deps -- -D warnings
git diff --check
```

All passed.

Result counts:

- `2` unit tests passed.
- `117` certification tests passed.
- `17` compile-time boundary test groups passed.
- `0` doc tests ran.
- Clippy completed with warnings denied.

Additional final gates passed:

- source/test line caps
- source/test direct-directory caps
- later-milestone implementation residue search

## Explicit Deferrals

Milestone 2 does not implement:

- final production cryptographic digest policy
- profile vocabulary or profile-driven materialization policy
- report, summary, artifact, or receipt taxonomy
- diagnostics and explanation ontology
- lineage and provenance ontology
- branch, merge, and commit vocabulary
- performance/layout vocabulary beyond canonicalization cost boundaries
- adopting-crate migrations or proof that real runtime lowering is correct
- a generic serializer, storage engine, scheduler, executor, or workflow
  harness

Those remain downstream roadmap work. Milestone 2 closes the canonical evidence
substrate those later surfaces must consume.
