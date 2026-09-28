# Milestone 3.10.1 Phase 7 Implementation Plan

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

## Objective

Prove the closed architecture through one real production lifecycle and independent
cost oracles.

The primary journey is:

`real .wui bytes -> OS watcher -> DSL-owned compilation -> sealed runtime candidate
-> product app lifecycle -> mounted execution -> host adapter -> compact inspection`

Phase 7 adds proof, not a second lifecycle. It must not add an integration target,
fixture workspace, Cargo session, parser hook, synthetic watcher event, or
certification-only production constructor.

## Boundary Review

### Existing authority

- `WorthUiFilesystemSourceWatcher` already owns recursive OS notification,
  settlement, and immutable filesystem snapshots.
- `worth-ui-dsl` owns source parsing, authored legality, diagnostics, and the sealed
  semantic package. Runtime lowering consumes that package and already carries
  construction/reload counters.
- `WorthUiActiveApplicationSession::execute_mounted_frame` is the sole ordinary
  mounted entry. Raw framework-turn and lane execution remain available only through
  `worth-ui-test-support`.
- `FilesystemApplicationLifecycleScenario` already constructs real file- and
  Rust-authored applications through production preparation and activation.
- `query_replacement_lifecycle::mixed_real_lifecycle` already composes a real watcher,
  Query, egui, invalid edits, Query removal/reintroduction, churn, and cleanup, but the
  388-line owner cannot absorb Phase 7.
- `cross_lane_bundle_execution`, `mounted_headless_recorder`,
  `mounted_egui_adapter`, `mounted_cost_evidence`, and
  `executor_allocator_observation` already own focused adapter and cost claims.
- Runtime steady-frame receipts already reject source parsing, registry discovery,
  unrelated traversal, and diagnostic materialization. Production receipt counters
  are evidence, not the independent oracle.
- `scripts/ci/run_worth_ui_test_lane.py` is the comparison authority for the opening
  baseline. The compile-contract owner already executes 23 fail and 12 pass targets
  in exactly two Cargo sessions.

### Missing closure

- No single named Phase 7 evidence bundle maps the required twelve-step hostile
  lifecycle to production observations.
- The real watcher lifecycle does not yet pair its production counters with an
  independently authored semantic expectation and explicit hot-frame source
  hostility.
- Existing allocation tests focus raw lane calls. Phase 7 must also observe unchanged
  and changed public mounted-frame work without using counters as the only oracle.
- File acquisition, Rust canonicalization, local replacement, syntax denial, runtime
  denial, steady frame, changed frame, inspection, and build-lane costs are not yet
  recorded as distinct evidence categories.
- Build target/session ceilings and comparable closing timing need one mechanically
  audited closing record.

### Destination authority

- One new named module tree under the existing
  `worth-ui-certification::application_contracts` integration target owns the Phase 7
  journey. Existing scenario modules remain focused and are reused through their
  public certification contracts rather than copied.
- The primary lifecycle module imports ordinary lifecycle operations from
  `worth_ui::facade::{app, source, inspection}` and authored meaning from
  `worth_ui_dsl`. It does not import runtime implementation or mounting phase modules.
- Separate adapter/cost submodules may consume existing sealed certification
  extension traits and runtime facades; those imports do not become product
  authority.
- Production counters may be propagated through existing receipts when missing, but
  no counter can decide semantic equivalence, watcher truth, adapter success, or
  allocation expectations.
- The Phase 7 closing record uses the exact opening-baseline lane runner, target
  posture, machine, and five lane names.

## Public DX Contract

The primary lifecycle remains an ordinary facade journey:

```rust
use worth_ui::facade::app::{
    UiMountedFrameOutcome, UiMountedFrameRequest, UiPresentationDeadline,
};
use worth_ui::facade::source::{
    WorthUiFilesystemSourceProvider, WorthUiFilesystemSourceWatcher,
};

let mut watcher =
    WorthUiFilesystemSourceWatcher::start(WorthUiFilesystemSourceProvider::new(root))?;
let submission = watcher
    .take_initial_snapshot()?
    .lower_to_candidate_submission(capabilities)?;
let mut session = app.with_candidate_submission(submission).freeze()?.launch()?;

match session.execute_mounted_frame(
    UiMountedFrameRequest::all_bound_surfaces(),
    UiPresentationDeadline::at_tick(1),
    0,
    |_| {},
)? {
    UiMountedFrameOutcome::Published(receipt)
    | UiMountedFrameOutcome::Unchanged(receipt)
    | UiMountedFrameOutcome::Reconciled(receipt) => observe(receipt),
    other => recover(other),
}
```

The product journey does not import raw framework turns, lane executors, mounted
preparation types, or runtime source internals.

## Implementation Batches

### Batch 1 - Phase 7 evidence authority

1. Add this plan and `milestone-3.10.1-phase-7-proof-ledger.csv`.
2. Add an exact Phase 7 evidence manifest naming every required lifecycle seam,
   observer, cost category, real caller, and existing test owner.
3. Extend the integrated topology audit to reject missing evidence, duplicate claim
   ownership, new integration targets, nested fixture workspaces, or a compile-session
   ceiling above two.
4. Add hostile manifest fixtures for a manufactured watcher event, production-counter
   self-oracle, omitted cost category, and extra Cargo target.

### Batch 2 - Real file-to-mounted hostile journey

1. Add `application_contracts/phase7_real_lifecycle/mod.rs` and small named child
   modules under the existing application target.
2. Start from real files and a production OS watcher.
3. Lower through DSL-owned compilation and compare selected authored meaning against
   an independent semantic expectation.
4. Prepare, activate, and execute a complete cross-lane mounted request through the
   product lifecycle.
5. Perform a semantically equivalent Rust-authored replacement and a valid local file
   replacement.
6. Observe compact provenance, transition, mounted publication, and cleanup evidence.

### Batch 3 - Denial and interruption preservation

1. Feed an invalid syntax edit through the real watcher and prove DSL denial preserves
   the complete active/mounted predecessor.
2. Feed valid syntax requiring an unsupported capability and prove DSL success,
   runtime denial, and predecessor preservation.
3. Exercise unchanged reuse, rejected-before-effects, bounded in-flight completion,
   indeterminate presentation, and reconciliation using existing production protocol
   owners.
4. Interrupt source handoff, replacement preparation, mounted presentation, and
   publication seams; compare the full predecessor tuple after every stop.

### Batch 4 - Hot-frame and scale hostility

1. After activation, replace on-disk source with poisoned/invalid bytes and execute
   repeated unchanged and changed mounted frames without asking the watcher to settle.
2. Prove independently that the active generation, mounted generation, and adapter
   consequences remain executable while all steady-frame source/registry counters are
   zero.
3. Vary unrelated source bytes/declarations, graph width, mounted breadth, and changed
   scope independently.
4. Assert that local replacement and frame work follow admitted changed scope and the
   existing named replacement granules, not unrelated breadth.

### Batch 5 - Adapter and allocation evidence

1. Compare one sealed mounted meaning against an independent headless transcript.
2. Execute at least one real egui context frame and observe native consequences
   separately from lifecycle publication.
3. Use the thread-scoped allocation observer around unchanged and changed public
   mounted execution.
4. Reconcile observed allocations with production receipts while retaining an
   independent zero/bounded expectation.
5. Keep Query-free and Query-bound journeys distinct and prove both cleanly shut down.

### Batch 6 - Cost and build closeout

1. Record separate evidence for file acquisition/lowering, Rust canonicalization,
   valid replacement, syntax denial, runtime denial, unchanged frame, changed frame,
   inspection materialization, and verification lanes.
2. Run the five opening-baseline lane names through
   `scripts/ci/run_worth_ui_test_lane.py` with the same warm/cold posture.
3. Write `milestone-3.10.1-phase-7-closing-evidence.json` with exact commands, exit
   codes, durations, target/session counts, structural counters, and adjudication of
   every regression.
4. Run the canonical compile executor and prove 23 fail plus 12 pass targets still use
   exactly two Cargo sessions.
5. Run full tests, topology, boundary-check, agent-context, formatting, line-cap,
   composition, test-evidence review, and strict Clippy before closing the ledger.

## Proof Strategy

- Real OS files and watcher notifications prove the filesystem claim.
- An authored expectation independent of production lowering proves selected semantic
  equivalence.
- Exact generation/publication comparisons prove predecessor preservation.
- Headless transcript and egui output are independent adapter observations.
- Thread-scoped allocation measurement is independent of runtime counters.
- Runtime counters prove structural exclusion and named work only after independent
  behavior has established the result.
- Scale metamorphics expose work that incorrectly follows unrelated breadth.
- Topology and compile inventories prove target/session budgets mechanically.
- Comparable lane measurements prove the architectural cleanup did not hide build or
  test cost.

## Causal Reopen Rules

- Any source acquisition, DSL handoff, or replacement change reopens file/Rust parity,
  invalid-edit preservation, runtime denial, and source-cost evidence.
- Any mounted entry, outcome, presentation, publication, or reconciliation change
  reopens the primary journey, predecessor tuple, adapter parity, and allocation
  evidence.
- Any counter schema or receipt propagation change reopens hot-frame hostility,
  scale evidence, and independent allocation reconciliation.
- Any scenario target, fixture workspace, compile inventory, or runner change reopens
  integration-target and two-session budgets.
- Any lane runner or measurement methodology change invalidates direct comparison
  with the opening baseline and requires explicit adjudication.

## Non-Goals

- Phase 8 documentation and later-milestone insertion closeout.
- New authored language, Query, snapshot, rebind, service, intent, interaction,
  observation, or appearance semantics.
- A second end-to-end harness, integration target, fixture workspace, or Cargo runner.
- Public cost constructors, parser poisoning hooks, synthetic watcher injection, or
  production test branches.
- Silently weakening an existing cost contract to make measurements pass.
