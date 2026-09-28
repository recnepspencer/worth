# WORTH Query Milestone 9.7 Phase 18 Closeout

Status: Closed

Phase 18 closes the Milestone 9.7 end-cap by deriving milestone posture from
phase-local closure proofs. It does not re-audit pinning, journal identity,
concurrent certification, or public-bridge consumption. Those proofs remain
owned by their phases.

## QA Findings And Corrections

1. The initial support-profile contract hard-coded `Closed` with placeholder
   phase evidence digests. That violated the Phase 18 rule that milestone
   posture must be aggregated from phase-local closure proofs, not support-row
   optimism. The correction makes the support-profile publication contract
   `Partial` until real phase-local evidence is supplied through
   `WORTHQueryMilestoneNineSevenDerivedClosure::derive_from_phase_closures`.

2. The initial phase-closure constructor was public, which allowed external
   callers to synthesize a `Closed` phase row without owning the phase proof.
   The correction narrows construction to the crate and keeps public
   construction through phase-specific adapters such as
   `from_shared_read_pinning`, `from_journal_replay_boundary`,
   `from_concurrent_hostile_matrix`, and `from_public_bridge_reader_lane`.

3. The closeout documentation did not distinguish support-profile publication
   from the actual derived closure artifact. The correction records that the
   support row publishes the required evidence contract while the closeout
   posture closes only from the supplied phase-local artifacts.

## Implemented Surfaces

- `WORTHQueryMilestoneNineSevenPhaseClosure`
- `WORTHQueryMilestoneNineSevenDerivedClosure`
- runtime support-matrix row:
  `milestone-9.7-derived-closure-posture`
- required-suite docs in `test-requirements.md`
- detailed suite row in `test-requirements-milestones-9_4-9_7.md`
- milestone closeout doc in `milestone-9.7-closeout.md`

## Required Phase Evidence

- Phase 13: `phase-13-shared-read-pinning`
- Phase 15: `phase-15-journal-replay`
- Phase 16: `phase-16-concurrent-hostile-matrix`
- Phase 17: `phase-17-public-bridge-reader-lane`

The derived milestone closure reports `Closed` only when all four required
phase rows are present, `Closed`, and evidence-bearing. Missing evidence or an
`Open`/`Partial` phase row prevents closure.

## Verification

Passed:

- `cargo fmt -p worth-query`
- `cargo check -p worth-query --all-targets`
- `cargo test -p worth-query --lib milestone_nine_seven`
- `cargo test -p worth-query --test public_bridge_reader_lane_honesty`
- `cargo test -p worth-query --test phase_boundaries_projection_consumption_compile_fail`
- `git diff --check` on the Phase 18 code and docs

## Defended Exclusions

- Store-backed execution parity belongs to Milestone 10.
- Durable restart and artifact reload belong to Milestone 11.

These are not Milestone 9.7 gaps. They are later milestone-class owners.
