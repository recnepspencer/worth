# Brief: cleanup step 1, succession characterization tests

This is test-only work. Do not change production behavior.

## Context

Worktree: `C:/forge_workspace/worth-ui-3.17`, branch `worth-ui-3.17`, HEAD `2056f4bd4b`.

Milestone 3.17 feature work is paused for a structural cleanup. Generation
succession is currently hand-wired across four writers, and it is being replaced
with a typed, compiler-total pipeline.

Read these first:
- The survey: `C:/Users/Esther/AppData/Local/Temp/claude/C--forge-workspace/47648b27-b3d8-4d01-b158-b01fe567d87e/scratchpad/succession_design.md`, especially §1 (writers W1-W4) and §4 step 1.
- The rulings: `.../scratchpad/cleanup_decisions.md`.

This step pins today's behavior exactly, so every later refactor step can prove
it changes nothing. It also decides whether the detached-completion hazard in
survey §1.5 is a live bug.

## Read before writing

- `AGENTS.md`, and every file in `docs/coding-guidelines/`. `testing_laws.md` and
  `qa_review_guide.md` are mandatory.
- `skills/qa-tests/SKILL.md`.
- The existing homes you will extend:
  - `workspaces/worth-ui/crates/worth-ui-runtime/src/facade/entry/active_application_session/expression_generation_following_tests.rs`
  - `condition_operability_succession_tests.rs`, `pointer_affordance_succession_tests.rs`
    and `expression_mounted_succession_tests.rs`, all in the same directory.
  - `workspaces/worth-ui/crates/worth-ui-runtime/src/runtime/tests/lifecycle/lifecycle_path_parity.rs`
  - Their fixtures.

## Deliverable

### Characterization tests

Write one counter-exact characterization test per writer:
- W1: evidence-only rebind.
- W2 attached: authored mounted-content rebind, completed while attached.
- W2 detached: detach, then complete.
- W3 Mounted cutover.
- W3 Unmounted cutover.
- W4: native mounted establishment.

Each test asserts, after commit:
- the active generation identity;
- that the pointer affordance snapshot equals a fresh observation of the
  committed state, and that it is present or absent as expected;
- the retained appearance owner snapshot's identity, or its presence;
- the overlay binding revision, where the writer has one;
- the exact values of `operability_reobservations`, the expression owner's
  `operand_probes`, `index_hits` and evaluation count, and the appearance
  invalidation batch count.

Rules for the counter assertions:
- Use the existing counter accessors. If a counter has no test accessor, add a
  `#[cfg(test)]` accessor beside the counter. No production accessor.
- Exact values only. Never write `>=` or `> 0`.
- Where an existing test already pins part of this, extend it rather than
  duplicating it.

### Detached-drift test

1. Start an authored mounted-content rebind and detach it.
2. While it is detached, apply an application fact update that changes a
   condition read by an intent whose pointer affordance is in the snapshot.
3. Complete the rebind.
4. Assert that the committed pointer snapshot and the standing operability facts
   equal a fresh observation of the committed state.

If this test fails today, the hazard is real. In that case:
- mark it `#[ignore = "detached completion commits pre-drift successor owners; fixed by the typed reattach (cleanup step 6)"]`;
- report the exact failing values.

If it passes, report why: which later path repairs the snapshot. Cite file:line
you read.

If a scenario can't be reached through today's crate-internal API, say so and
cite the code that blocks it. Do not add production seams to reach it.

## Constraints

- Keep each file's EOL style. Check it before editing. The working tree is
  mostly CRLF. Never normalize a file.
- Rust files stay at or under 400 lines, one responsibility each. Split a test
  file rather than growing one past the cap.
- Cargo:
  - Run from `workspaces/worth-ui`, offline, scoped with `-p worth-ui-runtime`.
  - Use `CARGO_TARGET_DIR=C:/forge_workspace/ui-targets-3161/target-test` for
    tests and `.../target-clippy` for clippy.
  - Run one cargo build at a time; the CPU is shared.
  - Run long test runs in the background.
- Run: `cargo fmt` (check only, on the touched files), `cargo clippy -p
  worth-ui-runtime --tests -- -D warnings`, `cargo test -p worth-ui-runtime`, and
  `python scripts/quality/scrutinize_rust_functions.py --dirty .`.
- Do not commit. Do not use `git stash`.
- Every claim about existing code in your report cites a file:line you read, or
  is marked UNVERIFIED.

## Report

Write your report to `.../scratchpad/step1_report.md`. It must contain:
- the tests added, with file:line;
- every pinned counter value per writer, as a table;
- the detached-drift verdict;
- any scenario you could not reach;
- the gate results.
