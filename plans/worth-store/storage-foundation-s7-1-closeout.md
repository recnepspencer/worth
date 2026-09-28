# S.7.1 Closeout: Structural Cleanup And S.7 Continuation Readiness

S.7.1 closes on structural evidence rather than new blob capability.

The cleanup leaves the remaining S.7 work with a narrower continuation surface:

- `worth-store-blob-chunks` now teaches lifecycle order from `src/lib.rs` through `src/exports/mod.rs` instead of exposing a flat noun dump.
- `worth-store-physical-format` keeps its root as aggregation only, with format authority grouped behind stable artifact families.
- `worth-store-recovery-physics` split the former `s4_integrity_handoff_payload.rs` seam into declaration, envelope evidence, and sealed payload construction.
- `worth-store-test-support` now distinguishes `harness::production_facade` from `harness::test_authority`.
- Future S.8 layout/index intake remains a follow-on contract. S.7.1 does not mint an S.8 readiness object because S.7 itself is not closed yet.

## Structural Bundle

The closeout bundle is machine-checked by:

- `workspaces/worth-store/crates/worth-store-certification/tests/s7_1_structural_closeout.rs`

Those proofs lock these boundaries:

- file-count and line-cap checks for cleaned loci:
  `worth-store-blob-chunks/src`,
  `worth-store-physical-format/src`,
  `worth-store-recovery-physics/src/s4_integrity_handoff_payload`,
  `worth-store-test-support/src`,
  `worth-store-layout-indexes/src`
- aggregation-only policy for:
  `worth-store-blob-chunks/src/lib.rs`,
  `worth-store-blob-chunks/src/exports/mod.rs`,
  `worth-store-physical-format/src/lib.rs`,
  `worth-store-recovery-physics/src/s4_integrity_handoff_payload/mod.rs`,
  `worth-store-test-support/src/lib.rs`,
  `worth-store-layout-indexes/src/lib.rs`
- facade visibility policy for the cleaned public roots
- continuation posture: remaining S.7 work should use cleaned public facades and phase-shaped handoff modules instead of raw internals, certification rows, or helper constructors

## Proof-Flow Outcome

The cleaned proof grammar now reads as named transitions instead of copied fields:

- collect evidence
- classify the case
- verify the transition
- construct the receipt
- expose the next capability

This is mechanically visible in the blob lifecycle facade, the corruption and hostile-lane surfaces, and the split recovery handoff payload seam.
The blob hostile-lane proof now lives in `corruption::integration_tests` as a direct child of the
corruption state machine instead of a nested test helper module, so the closeout command targets
the real authority boundary it certifies.

S.8 readiness is intentionally not implemented here. The follow-on requirement is that S.8 must consume a typed capability produced by completed S.7 lifecycle authority, not a tuple of receipts, raw counters, certification rows, or harness helpers.

## Authority Boundary

Production authority remains in production crates:

- blob lifecycle, publication, corruption, recovery, and reclaim capability types stay under `worth-store-blob-chunks`
- physical format and recovery handoff law stay under their Store production crates
- certification remains the courtroom that scans and cross-examines the cleaned topology
- test support can assemble legal production flows or explicitly named synthetic evidence, but it cannot masquerade as remaining S.7 production authority

## Focused Verification

Focused closeout verification:

- `cargo test -p worth-store-certification --test s7_1_structural_closeout`
- `cargo test -p worth-store-blob-chunks integration_paths_classify_all_five_damage_cases_before_decode -- --nocapture`
- `cargo test -p worth-store-test-support --test harness_authority_compile_fail`
- `cargo check -p worth-store-layout-indexes -p worth-store-certification -p worth-store-blob-chunks -p worth-store-recovery-physics`

## Explicit Structural Exceptions

The closeout keeps three explicit exceptions rather than pretending S.7.1 rewrote unrelated roots:

1. `workspaces/worth-store/crates/worth-store-blob-chunks/src/exports`
Reason: one small aggregation file per lifecycle family keeps the public blob facade ordered by authority without re-exporting internals from larger mixed files.
Owner: S.8 layout/access-path intake follow-on.
Scope: public export grouping only.
Follow-on milestone: `S.8`.

2. `workspaces/worth-store/crates/worth-store-buffer-pool/src`
Reason: S.7.1 Phase 13 only cleaned blob-adjacent memory-boundary seams; the broader S.2 crate root topology remains a dedicated buffer-pool restructuring task.
Owner: S.2 / S.8 follow-on.
Scope: full buffer-pool root decomposition.
Follow-on milestone: `S.8`.

3. `workspaces/worth-store/crates/worth-store-physical-integrity/src`
Reason: S.7.1 Phase 13 only cleaned corruption-first admission and handoff seams needed by blob/corruption proof flow; the full S.3 crate topology remains broader than this cleanup gate.
Owner: S.3 / S.8 follow-on.
Scope: full physical-integrity root decomposition.
Follow-on milestone: `S.8`.
