# Milestone 3.11 Phase 5B Implementation Plan

> Historical QA policy (2026-08-22): proof, closure, migration, acceptance,
> and phase ledgers described below are frozen historical records. They are not
> active implementation or release gates, are not updated or reopened, and a
> ledger-only failure does not block current work. Current evidence follows
> [the QA review guide](../../docs/coding-guidelines/qa_review_guide.md) and
> [testing laws](../../docs/coding-guidelines/testing_laws.md): specifications state QA
> considerations in prose, tests and repository checks run against the current
> commit, and code review decides whether the evidence is adequate. This note
> does not retire product-domain ledgers that are part of runtime behavior.

Status: closed

## Destination

Phase 5B makes every declared visual resource bound an enforcing production
contract. A host-backed capture must reserve its maximum retained pixel and
structural footprint before a host request can exist. A derived region capture
must atomically transfer the consumed parent resource rather than pretend that
one linear resource is two snapshots. Completion must prove its actual retained
footprint fits the reservation, and disposal or shutdown must release and
report both pixel and structural bytes.

This batch does not add a diagnostic manager, alternate capture entry, new test
target, or scheduled correctness lane.

## Authority Review

- `worth-ui-inspection/query/visual_snapshot/disclosure.rs` owns immutable
  caller policy values. It will add a concrete visible/hit region capacity and
  distinct per-receipt/per-session structural budgets.
- `worth-ui-runtime/inspection/visual_snapshot/grant.rs` projects every policy
  bound into the concrete session-minted grant scope.
- `worth-ui-runtime/facade/entry/visual_snapshot.rs` owns admission order. It
  must acquire mounted truth, calculate checked reservations, and reject before
  host effects.
- `worth-ui-runtime/inspection/visual_snapshot/registry.rs` owns active
  reservations, retained resources, successor transfer, and cleanup totals.
- `worth-ui-runtime/inspection/visual_snapshot/spatial/` owns the exact
  representation-size formula for visible and hit-test indexes.
- `worth-ui-runtime/inspection/visual_snapshot/overlay.rs` owns immutable
  published/cleared overlay cost projections; the overlay registry still owns
  lifecycle.
- ordinary execution remains owned by the ordinary lane and must expose eleven
  zero visual counters across repeated unchanged frames.

## Typed Policy and Denials

Add `UiVisualInspectionRegionCapacity` with independently named visible and
hit-test ceilings. The production default is 65,536 for each.

Split the structural byte policy into:

- maximum retained structure per receipt: 64 MiB by default;
- maximum retained structure per session: 256 MiB by default.

The public denial enum must distinguish:

- visible-region capacity;
- hit-test-region capacity;
- per-receipt structural capacity;
- per-session structural capacity;
- retained-pixel capacity; and
- snapshot-count capacity.

Accounting overflow may retain the generic capacity posture. No boolean policy,
generic marker, or string denial is permitted.

## Host Capture Admission

The host route proceeds in this order:

1. prove grant session and exact disclosure;
2. consume the typed target into its route;
3. acquire the exact mounted snapshot basis;
4. check visible and hit-test basis counts;
5. calculate a checked structural reservation;
6. check the per-receipt bound;
7. reserve pixel, structure, count, and surface capacity in the session
   registry;
8. mint the capture identity and pin the typed capture progression; and
9. allow polling to construct the host request.

The structural reservation is the maximum of:

- pending retained structure: mounted-frame pin + cloned trace basis + mounted
  visual-region basis; and
- completed retained structure: mounted-frame pin + cloned trace basis +
  exact visible/hit interval-index representation at the admitted record
  counts.

All arithmetic is checked. A denial drops the acquired mounted lease and opens
no host request or registry entry.

## Completion and Derived Successor

Host completion computes actual retained structure from the exact sealed
indexes, trace basis, and mounted-frame pin. Registry completion asserts actual
pixels and structure do not exceed their reservations, then converts the
active reservation into one retained resource.

A derived region target already consumes its parent receipt. It therefore:

- creates no second active registration;
- retains the parent registry resource while pending;
- on success, atomically rekeys that resource to the child identity and
  replaces its pixel/structural accounting;
- on cancellation or failure, drops the consumed parent and releases exactly
  one resource.

This preserves a snapshot count of one and avoids transient double-accounting.

## Overlay and Shutdown Cost

`UiPublishedVisualOverlay` carries an overlay-lane cost receipt proving four
emitted border regions, one retained lease, and the mounted structural bytes
retained by that lease. `UiClearedVisualOverlayReceipt` carries the successor
clear cost with zero emitted overlay regions and zero retained overlay leases.

Visual capture shutdown adds disposed structural bytes beside disposed pixel
bytes. The Platform Pulse version-2 shutdown observation and executable
adjudication project and verify that field. No older protocol is reinterpreted.

## Proof Worlds

Within the existing runtime unit target:

- generated exact spatial worlds cover 1, 1,024, and 65,536 records;
- estimator and built-index retained bytes agree;
- maximum overlap still becomes typed incomplete at the 4,096 candidate
  default.

Within the existing consolidated application target:

- a real filesystem/egui four-way world proves visible and hit count denials
  before screenshot commands;
- bounded real worlds distinguish per-receipt structure, per-session structure,
  pixel, snapshot, and overlay denials;
- derived crop proves atomic parent-to-child resource transfer under a
  one-snapshot policy;
- overlay publish and clear assert exact cost receipts;
- repeated unchanged ordinary frames each expose `[0; 11]`.

The existing executable-world target proves shutdown structural cleanup in the
same permanent product journey.

## Ordered Implementation

1. Add typed policy/grant fields and distinct denials.
2. Add exact structural estimation and resource reservation values.
3. Reorder host admission and implement registry structure accounting.
4. Implement derived successor transfer.
5. Add overlay cost and shutdown structural projection.
6. Add focused production-boundary and deterministic cost proofs.
7. Run `qa-loop`, `qa-tests`, and `code-quality-qa`.

Phase 5C remained locked until all seven steps passed on the same source.

## Closure Evidence

- The Platform Pulse executable-world target passes 9 tests with the product
  selecting WGPU and the independent observer capturing the exact process and
  HWND through WGC. The canonical journey uses 6 native captures, zero
  retries, and reports zero retained visual resources at shutdown.
- The consolidated application target passes 56 visual contracts. Runtime
  spatial evidence passes 7 exact cost contracts at 1, 1,024, and 65,536
  records, and repeated unchanged ordinary frames each remain `[0; 11]`.
- The canonical compile matrix passes 35 fail and 13 pass targets in exactly 2
  Cargo sessions. All 5 Phase 5 contract/ledger audits pass while every
  milestone verdict row remains `OPEN`.
- Affected all-target/all-feature clippy passes with warnings denied. All 357
  dirty or new WORTH UI Rust files are within the 400-line cap; dirty-function
  scrutiny found no Phase 5B production advisory defect.
- `cargo fmt --check`, `boundary-check`, `agent-context check`, and
  `git diff --check` pass on the closing source.

Phase 5C is now the sole unlocked batch.
