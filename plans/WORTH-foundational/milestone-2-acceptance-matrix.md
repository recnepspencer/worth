# WORTH Foundational Milestone 2 Acceptance Matrix

This matrix is a pre-closeout audit aid for `plans/worth-foundational/milestone-2.md`.
It intentionally ignores later roadmap milestones except where Milestone 2 names
them as deferred debt or future expansion points.

## Phase Gates

| Phase | Status before final QA | Primary evidence |
| --- | --- | --- |
| Phase 1: canonical basis grammar | Implemented | `src/canonicalization/basis/{grammar,construction,readiness}` and `tests/certification/canonicalization/basis/basis_grammar.rs` |
| Phase 2: Milestone 1 basis builders | Covered by final QA | `src/canonicalization/*_preparation.rs`, `src/canonicalization/contract_preparation/`, and `tests/certification/canonicalization/digest_preparation/` |
| Phase 3: equivalence and mismatch basis | Implemented | `src/canonicalization/equivalence/`, `src/canonicalization/mismatch/`, and `tests/certification/canonicalization/equivalence/comparison_readiness.rs` |
| Phase 4: export and golden fixture bundles | Implemented | `src/canonicalization/export/`, `tests/certification/canonicalization/export/`, and `tests/certification/canonicalization/golden_artifacts/` |
| Phase 5: digest slots and derived digest values | Implemented | `src/canonicalization/digest_slots/` and `tests/certification/canonicalization/digest_slots/digest_derivation.rs` |
| Phase 6: production-test readiness | Covered by final QA | `src/canonicalization/production_readiness/`, `tests/certification/canonicalization/production_readiness/`, and `tests/certification/canonicalization/proof_carriage/` |

## Milestone 2 Acceptance Evidence

| Requirement | Status before final QA | Evidence or audit target |
| --- | --- | --- |
| Versioned canonical basis grammar | Covered | `basis/grammar`, `basis/construction`, `basis/readiness` |
| Typed domains, loci, entry kinds, values, and entry ids | Covered | `basis/grammar` and basis compile-fail fixtures |
| Concrete value carriers avoid arbitrary string/blob collapse | Covered | `CanonicalBasisValue` and golden value-family tests |
| Deterministic sequence and bundle ordering | Covered | `basis_grammar.rs` |
| Duplicate, domain-incoherent, and malformed basis rejection | Covered | `basis_grammar.rs` |
| Cost accounting for entry count, ordering, nested sequence, and compatibility lowering | Covered by final QA | basis grammar tests plus exact production-readiness cost evidence |
| `CanonicalBasisReadyArtifact` proof carriage | Covered | `proof_carriage/readiness_artifacts.rs` |
| Basis builders for Milestone 1 surfaces | Covered by final QA | `digest_preparation/*` plus exact golden/readiness evidence for contract, mask, state, patch, compatibility, identity, and locator surfaces |
| Native and compatibility-origin parity | Covered by final QA | `state_basis.rs`, compatibility tests, and property-seed evidence |
| Storage-equal but meaning-distinct variants | Covered | value-family, identity, locator, and digest-slot tests |
| Blind-consumer interpretation from basis entries | Covered in targeted areas; final QA should verify all named surfaces are represented | digest-preparation, equivalence, export tests |
| Distinct checked outcome lanes where exposed | Covered by final QA | compatibility and canonicalization tests |
| Explicit equivalence basis before comparison | Covered | `comparison_readiness.rs` and compile-fail boundaries |
| Structured mismatch basis and smallest-locus reporting | Covered | `comparison_readiness.rs`, golden mismatch tests, export first-mismatch tests |
| Unsupported comparison is structured, not ordinary inequality | Covered | `comparison_readiness.rs` |
| Export-ready bundle and manifest proof | Covered | `export_ready_fixtures.rs`, proof-carriage tests |
| Golden fixtures for value, contract, mask, state, patch, identity, locator, compatibility, equivalence, mismatch, export, and digest surfaces | Covered by final QA | exact `CanonicalGoldenArtifactEvidence` rows and fixture manifest mappings |
| Boundary-bridged export freshness downgrade and readmission | Covered | export readmission tests and compile-fail fixture |
| Digest algorithm slots, metadata, and input-shape typing | Covered | `digest_slots/*` and `digest_derivation.rs` |
| Digest values cannot replace equivalence basis evidence | Covered | digest derivation tests and compile-fail fixture |
| Raw bytes, debug strings, category-erased blobs, plain slots, and plain payloads rejected | Covered by final QA | `tests/ui/canonicalization/**` |
| Production-test readiness artifact | Covered | `production_readiness` implementation and certification tests |
| Readiness inventory: certified surfaces, pressures, compile-fail boundaries, golden artifacts, property seeds, worth-harness expansion points, assumptions, non-assumptions, and debt | Covered by final QA | exact readiness inventories, concrete golden fixture rows, concrete property-seed evidence, and readiness tests |
| Topology is responsibility-shaped rather than a flat dump | Covered by final QA | split canonicalization basis, equivalence, export comparison, digest material, contract preparation, line caps, and direct-directory caps |

## Explicitly Out Of Milestone 2 Scope

These test-requirements scenarios remain important, but Milestone 2 must not
pretend to close them because they require later vocabulary:

- profile-controlled diagnostic materialization over authoritative state
- report/artifact/receipt category separation over a simulated authority boundary
- branch candidate plus merge verdict plus commit receipt plus digest basis
- reduced-richness branch/merge materialization over committed authority
- provenance plus receipt plus digest basis over one canonical boundary artifact
- diagnostics, lineage, provenance, receipts, and performance/layout vocabulary

Milestone 2 may reserve domains, fixture debt, and worth-harness expansion points
for these surfaces. It must not claim their production-shaped behavior is proven.

## Final QA Checklist

- Re-run `cargo test -p worth-foundational`.
- Re-run `cargo clippy -p worth-foundational --all-targets --no-deps -- -D warnings`.
- Re-run `git diff --check`.
- Re-run source/test line-cap and direct-directory-cap checks.
- Keep every row above grounded in concrete test files before writing
  `milestone-2-closeout.md`.
- Ensure closeout names any remaining later-surface debt explicitly instead of
  implying production runtime readiness.
