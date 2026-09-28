# Worth UI 3.9 Phase 8 Boundary Brief

**Status:** Closed. Exact-reference preservation, live successor switching,
candidate-only rollback, removal retirement, and Query-owned close evidence are
proven through the public application lifecycle and real Query Consumer Kit.

## Outcome

Phase 8 makes an already-admitted installed Query view executable as one
visible-range slice borrowed from the active application plan. The plan owns
the view row and exact installed reference; the Query binding owns consumed
projection authority, native observations, and live resources. A frame joins
those two sealed authorities by exact reference and never receives a plan,
registry, projection bag, or Query resource from the caller.

This phase proves the view-reference, handle, range, evidence, and lifecycle
substrate. It does not claim Query collection materialization, cursor
construction, pagination, patch production, or Milestone 3.13/6 live-product
semantics.

## Adversarial constraint

Matching labels, digests, native values, or visible output are never Query
authority. A foreign Query installation with identical definitions and values
must fail before row execution. Likewise, a caller-held
`WorthUiVirtualizedDataPlan`, a UI-authored Query posture enum, or a test-only
full-scan target cannot certify the production boundary.

## Real authorities entering the slice

- The prepared application Query binding plan owns the exact installed-domain
  authority and registered view definitions.
- The sealed lowering authority resolves each Query row to one
  `WorthUiInstalledQueryBindingReference`; the regional executable retains that
  reference and its binding identity by shared immutable ownership.
- `WorthUiRuntimeQueryBinding` alone consumes the lifecycle-specific
  `WorthUiQuerySnapshotProjectionOutcome` or
  `WorthUiQueryLiveProjectionOutcome`, retains the latest exact settlement for
  each registered view, and exposes only a compact read-only
  execution/evidence reference. The snapshot admission path cannot accept a
  live envelope.
- Snapshot and live installed views remain distinct lifecycle types until they
  enter a non-executable registration envelope. Snapshot views own the one-shot
  read/projection path; live views own Query `open` and yield a managed resource.
- The active execution-plan bundle owns the virtualized row index, query-free
  posture, handles, and range executor.
- Query owns installed live handles and close receipts. Worth UI may retain and
  route one opaque binding-owned live-resource wrapper, but it cannot reproduce
  subscription, recovery, result-state, or disposal semantics.
- The Phase 4 publication shell owns the atomic application, active-plan, and
  Query-binding successor cutover.

## Architectural corrections required before the slice is complete

1. `WorthUiPlanNodeInput` equivalence must compare the exact installed
   reference, not only its definition. The regional executable must retain the
   exact reference; otherwise a foreign runtime can reuse a row.
2. Virtualized lowering must iterate the regional `QueryViewBinding` family
   index. Scanning reconstructed flat topology and lane-support links is a
   competing authority and makes unrelated graph breadth part of cold cost.
3. The active sealed bundle must own an explicit `Executable` or `QueryFree`
   virtualized posture. Product execution cannot accept a caller-built plan.
4. `WorthUiRuntimeQueryBinding::admit` must retain the exact successful
   settlement. A later no-ingress framework turn must resolve it without
   replaying Query or materializing diagnostics.
5. Frame counters must describe work actually performed: direct row
   resolution, requested visible rows/columns/cells, evidence-reference joins,
   and forbidden work remaining zero. They must not claim Query patch or
   collection execution.
6. Test-only full-collection, offset-pagination, and wrong-family executor
   variants must disappear. Empty/overflowing ranges are rejected by the real
   range constructor; missing ranges and wrong handles are rejected by the real
   active executor; source audits prove no widening/full-scan branch exists.
7. Application replacement must prepare and publish the successor Query
   binding with the successor app and plan. The current predecessor binding
   cannot remain installed after cutover.
8. Exact matching references must retain their settlement/live resource across
   successor publication. Rebound or removed references must yield an opaque,
   single-use retirement handoff whose close operation remains Query-owned.
9. The lifetime matrix assigns first-path rebind/removal proof to Phase 8.
   Phase 8 owns and proves those first public rows; Phase 14 re-proves them
   under bounded regional churn and locality pressure.
10. Live-resource and consumed-projection admission must be one affine
    transaction. A denial returns the resource or routes it through Query-owned
    abandonment cleanup; it cannot strand a half-admitted settlement. Query
    must also prove that the projection belongs to that exact managed-resource
    generation; equal definition and installed authority do not establish the
    pair.
11. Phase 8 owns candidate-only resource rollback because it first creates the
    resource. Successor preparation completes before predecessor release, and
    a failed preparation or Phase-4 publication-shell decision preserves the
    predecessor while Query closes or reaps only the candidate resource. Phase
    15 re-proves this inside the final cross-family transaction.

## Representation and cost

Each virtualized row stores only:

- its active view-binding handle;
- the exact shared installed binding reference;
- the exact shared UI binding identity; and
- row-local immutable execution metadata needed for visible-range admission.

The active plan builds a direct handle index from the regional Query-family
view. It does not duplicate the Query registry or native observations. The
runtime binding retains the latest settlement in a map keyed by view identity;
lookup rechecks the exact installed reference before returning a compact
evidence reference that shares Query-owned native observations.

Frame cost is `O(1)` row/evidence resolution plus `O(requested visible window)`
declared host work. It is independent of unrelated graph rows and other Query
view rows. No frame path parses strings, constructs cursors, scans a collection,
clones a full settlement, or allocates a diagnostic report.

## Public DX target

```rust
let summary = session.inspect_virtualized_plan(
    WorthUiVirtualizedPlanSummaryRequest::first_view(),
)?;
let target = summary
    .target(WorthUiVisibleRange::rows(120, 40)?)
    .ok_or(NoRegisteredView)?;

let execution = session
    .execute_framework_turn(|_| {})
    .into_execution()?;
let completed = execution.execute_virtualized_data_frame(target)?;

assert_eq!(completed.receipt().visible_range().row_count(), 40);
assert_eq!(completed.receipt().evidence().observations()[0].extent(), expected);
```

The summary is budgeted and read-only. It can expose definition, lifecycle,
shape, handle, native observations, and opaque evidence coordinates, but cannot
recover installed-domain or consumed-projection authority.

For live views, the installed view opens a Query-owned resource through the
Query workspace. The active binding admits that opaque resource. Successful
replacement either preserves it for an exact reference or returns a single-use
retirement token for the caller to close through the same Query workspace.
Dropping an unclosed handoff must enter Query's managed-resource abandonment
lane; Worth UI may not implement an independent fallback disposer.

The lifecycle distinction is structural:

```rust
let snapshot = installed.measurement_view("inspector.measurements")?;
let read = snapshot.read(&mut workspace)?;
let projection = snapshot.project(&read, domain::project_facts().display_field(path))?;

let live = installed.live_measurement_view("inspector.measurements")?;
let resource = live.open_using(domain::current(), &mut workspace)?;
let live_read = resource.read(&mut workspace)?;
let live_projection = resource.project(
    &live_read,
    domain::project_facts().display_field(path),
)?;
```

The binding consumes `resource` and `live_projection` together. A stopped
admission returns the resource-bearing stop so the caller can retry or close it;
successful admission stores both before the framework turn can publish a
settlement.

## Slice plan

### Slice A: exact active visible-range execution

1. Fix exact installed-reference equality and retain shared binding facts in
   regional executables.
2. Add a binding-owned compact execution/evidence reference backed by the exact
   admitted settlement and native observations.
3. Replace provisional virtualized rows/counters/targets with the regional
   Query-family indexed active posture.
4. Seal that posture into the active bundle and expose budgeted discovery plus
   framework-turn execution through the active application session and minimal
   host output.
5. Remove caller-plan execution and test-only executor variants; replace their
   tests with production-boundary hostile cases and independent scale proofs.

### Slice B: atomic Query succession and real lifecycle proof

1. Prepare a successor runtime binding from the successor application authority
   before activation; preserve settlements/resources only for exact references.
2. Stage the successor binding inside the Phase 4 publication shell and include
   the application, active plan, and Query binding in the same final infallible
   commit. No bind-after-plan-swap step is permitted. Every denial before that
   commit leaves the predecessor binding untouched; retirement handoff begins
   only from the successful commit receipt.
3. Add the binding-owned installed live-resource wrapper and single-use Query
   close handoff needed to prove preservation, rebind, removal, and exact-once
   disposal without a UI-local resource manager.
4. Make resource-plus-projection admission atomic and make every stop retain the
   resource. Prove candidate preparation/publication denial cleans up the
   candidate through Query while leaving the predecessor usable.
5. Extend the existing `application_contracts::query_consumer_kit_lifecycle`
   module: real Consumer Kit installation, real projection, public launch,
   active visible-range execution, public replacement/removal, live-resource
   succession, and Query-owned close receipt.
6. Amend the lifetime matrix to make Phase 8 the first-path owner and keep
   Phase 14 responsible for bounded-churn reproof.

## Proof plan

1. Fast runtime tests prove exact-reference equivalence, foreign-installation
   denial, stale/foreign handles, invalid/missing ranges, query-free posture,
   active-plan ownership, and independent graph/Query scale bounds.
2. Binding tests prove exact native `AspectValue`/`StructAspectValue` reaches the
   plan edge through shared Query-owned evidence with no text/JSON/widening.
3. Source/topology assertions prove there is no caller-plan executor,
   test-only hostile target, Query posture mirror, full-scan branch, offset
   substitute, or per-frame reconstructed registry.
4. The existing application-contract target proves the real public Query and
   replacement lifecycle once; no new test binary, compiler session, fixture
   workspace, nested Cargo invocation, or fast-lane external I/O is added.
5. The hostile QA loop treats passing counters as insufficient unless the real
   production mechanism and authority edge are independently observed.

## Later-phase handoff

Phase 14 reuses these first-path lifecycle transitions to prove bounded regional
replacement and sustained-churn locality. Phase 15 re-proves candidate rollback
inside the complete application/plan/Query/host/inspection publication
transaction. Milestones 3.13 and 6 own collection/cursor/patch semantics; they
must extend this sealed reference boundary rather than replace it with a
parallel Query lane.
