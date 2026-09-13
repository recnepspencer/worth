# Signal Branch Bases

## What This Feature Is

Signal branch bases let you name one exact, live branch state without exposing
mutable graph internals. A basis says which runtime, branch, Signal definition,
snapshot posture, and reference generation you observed. Use it when work must
fork, restore, advance, merge, or retain a particular Signal branch without silently
falling back to whatever branch is current later.

A **component obligation** (a retention lease) is the second half of that story.
It lets one component keep an exact past state available while the branch itself
moves on without it.

## Why You Use It

- Fork background or preview work from the exact state you inspected.
- Reject a stale request after another transaction, snapshot, restore, or merge
  moves the branch reference.
- Carry a descriptive basis through JSON and ask the owning runtime to readmit
  it before it can operate.
- Keep one exact component state available to a downstream owner after the
  branch reference has moved past it.

## Stable Entry Points

Import owner artifacts from `worth_signal::facade::branch` and call these
methods on `SignalRuntime`:

- `observe_signal_branch_basis`
- `readmit_signal_branch_basis`
- `fork_signal_branch`
- `restore_signal_branch`
- `validate_signal_basis_compatibility`
- `retain_signal_component_basis`
- `release_signal_component_basis`
- `readmit_retained_signal_branch_basis`
- `signal_component_retention_terminal_counts`
- `advance_signal_branch`
- `merge` for guided planning and `merge_branch` for the full-branch shortcut
- `capture_signal_branch_snapshot`
- `reconstruct_signal_branch_snapshot` for pristine-runtime construction only
- `plan_signal_branch_retirement` and `retire_signal_branch`

One terminal operation lives on the obligation itself rather than on the
runtime:

- `SignalBranchRetentionLease::release` consumes the obligation and returns the
  same governed receipt without needing a runtime to call it through. It is the
  explicit path that remains once the issuing runtime is gone.

The lease's inspection accessors are described under
[Inspection And Debugging](#inspection-and-debugging).

`SignalBranchBasisDescriptor` is the transport form. It is descriptive data,
not permission to operate. `AdmittedSignalBranchBasis` is the runtime-issued
form accepted by governed operations.

`SignalSnapshotV1` is likewise portable data, not restore authority.
`capture_signal_branch_snapshot` returns an owner-bound
`AdmittedSignalBranchSnapshot`; ordinary `restore_signal_branch` accepts only
that admitted form. A fresh, pristine runtime may import portable snapshot
data through the distinct `reconstruct_signal_branch_snapshot` construction
lane, which validates its empty live basis and issues a new owner-bound
snapshot.

An otherwise eligible branch cannot import into a runtime with other live
branches, stored snapshots, or prior snapshot identity use: that is
`NonPristineRuntime`, before movement. Successful import preserves the portable
snapshot identity and advances the receiving allocator so later captures cannot
reuse it.

An admitted snapshot carries its own bounded branch-retention obligation.
Snapshot clones share that one obligation, and branch retirement remains
denied until the final admitted snapshot holder is dropped. Consuming the
admitted form into portable `SignalSnapshotV1` data releases that authority
and its retention. Owners that coordinate snapshot disposal with retirement
may use the snapshot-release planning lanes: planning borrows each concrete
admitted snapshot, validates its runtime and branch identity, and leaves every
snapshot usable on denial. The owner must release the declared snapshots
before executing the returned retirement plan; execution revalidates that no
undeclared or cloned authority still retains the branch.

Heavy branch snapshot states are also governed by a runtime-wide stored-
snapshot budget (`maximum_stored_branch_snapshots` on the runtime builder).
Capacity is checked before capture moves a branch reference. Exhaustion is a
typed `SnapshotCapacityExhausted` denial on the canonical capture lane, and
branch retirement reclaims that branch's stored states for subsequent use.

The older transaction-head tuple and strong basis artifact are engine details,
not production facade contracts.

## Core Mental Model

The `SignalRuntime` owns live branch truth. Its branch manager is the one
operational catalog for branch handles and reference generations. Graph
diagnostics may display that catalog, but they do not provide a second source
of currentness.

A **basis** is an exact observation plus owner proof. Cloning an admitted basis
shares the same immutable admission; it does not inspect the graph, reacquire a
snapshot, or run evaluation. That shared admission also carries one internal
retention obligation: every `Arc` clone shares it, and the obligation is
released only after the last clone is dropped.

A **descriptor** is a serializable copy of the observation. Serialization
deliberately drops operational authority. The receiving runtime checks owner,
definition, lifecycle, snapshot availability, and exact reference generation
before issuing a new admitted basis.

Observation and readmission each acquire exactly one bounded admission. If the
retention registry is full, observation reports
`SignalBranchBasisObservationDenial::RetentionUnavailable` and readmission
reports `SignalBranchBasisReadmissionDenial::UnavailableRetention`; neither
misclassifies the branch as unknown.

### Residency Is Not Currentness

Retention and readmission ask the runtime two different questions about the
same state, and the answers are independent:

```text
readmit_signal_branch_basis(descriptor)
    "Does this branch still select this state?"
    -> ReferenceMismatch once any operation moves the reference.

retain_signal_component_basis(&basis)
    "Is this exact immutable state still available?"
    -> Ok. Currentness is never a reason to refuse.
```

An obligation over a historical target is therefore ordinary, not exceptional.
`SignalBranchRetentionAcquisitionDenial` has no stale-basis variant on purpose:
every refusal it can give is a fact about the owner, the branch lifecycle, the
Signal definition, or the continued availability of that exact target.

Be precise about what this does and does not say. Currentness never *denies*
retention. The branch reference is still allowed to *help*: a target is
available if the runtime still holds it as a stored snapshot state **or** if it
is the branch's current head. Consulting the reference can therefore only add
availability evidence, never subtract it.

A lease is consequently **not** evidence that its branch reference is current,
and it is not a private copy of the state. It is an obligation the owner
honors: that exact immutable state stays available, and branch retirement stays
denied, until the obligation ends.

### An Obligation Names One Exact Target

`SignalBranchRetentionLease` is deliberately not `Clone`. The obligation is the
value, so it cannot be duplicated and cannot be released twice. It authorizes
exactly the target it was opened over: `readmit_retained_signal_branch_basis`
refuses any other descriptor with `DescriptorMismatch`, and refuses a runtime
that did not issue it with `ForeignRetention`.

The same exact target may carry several independent obligations. Release
receipts keep them distinct: `remaining_target_leases` counts obligations still
held over that one exact target, while `remaining_branch_leases` counts
obligations still held over any target of the same branch.

### Acquire Before You Lose It

`readmit_retained_signal_branch_basis` is the only route back to an admitted
basis for a state the branch no longer selects, and it requires a live
obligation. Ordinary readmission answers the currentness question and refuses.

So retain **before** you drop the last admitted basis for a state you will
need. Once no admitted basis and no obligation remain, that exact target is no
longer reachable, and branch retirement is free to reclaim it.

### Every Terminal Path Ends the Obligation Exactly Once

An obligation reaches a terminal state exactly once. Three surfaces get it
there, and two of them are explicit:

- `SignalRuntime::release_signal_component_basis` offers the obligation back to
  a runtime. This is the path to prefer while you still hold the issuing
  runtime, because it checks owner affinity first: a runtime that did not issue
  the obligation answers `Denied` and hands the live lease back instead of
  spending it. On acceptance it returns a
  `SignalBranchRetentionReleaseReceipt`: the released target, the terminal
  outcome, and both remaining counts.
- `SignalBranchRetentionLease::release` consumes the obligation directly and
  returns the same receipt. It needs no runtime, so it is the explicit path
  that remains once the issuing runtime is gone and there is no runtime left
  to offer the obligation to. Because there is no runtime to compare against,
  it performs no affinity check; reach for the runtime lane when you have a
  runtime to reach for.
- Dropping the lease takes the same exactly-once terminal path, recorded as a
  dropped release instead of fabricating a receipt nobody asked for.

Forgetting to release is therefore not a leak. Every path discharges identical
accounting and every path unblocks retirement; only the two explicit paths
yield governed evidence.

An obligation may also outlive the runtime that issued it. Its
`owner_posture()` becomes `Lost`, releasing it reports
`SignalBranchRetentionTerminalOutcome::OwnerUnavailable` rather than `Released`,
and `owner_terminal_counts()` stays readable so the loss remains observable.
The owner ledger survives the runtime; the retained state does not. A lost
owner cannot hand the target back, so `readmit_retained_signal_branch_basis`
reports `UnavailableRetainedTarget`.

### Reading The Terminal Counters

`signal_component_retention_terminal_counts` reports two independent axes, not
four disjoint buckets. Reading them as a flat tally will not add up.

- **Cause** is a partition of every obligation the ledger accounted as
  terminally released. `explicit_releases` and `dropped_releases` are its two
  buckets, and `terminal_releases()` is exactly their sum.
- **Owner posture** is an overlay on those same events.
  `owner_loss_releases` counts how many of them found no live owner, so it is a
  subset of `terminal_releases()` rather than a third bucket. One release of an
  orphaned obligation increments a cause bucket *and* the overlay: an explicit
  release after owner loss raises both `explicit_releases` and
  `owner_loss_releases` by one, and a dropped one raises both
  `dropped_releases` and `owner_loss_releases`.
- `unknown_lease_defenses` is on neither axis. It counts terminal attempts the
  ledger did not recognise, where nothing was released, so it is never part of
  `terminal_releases()`.

Branch retirement is denied while external obligations remain. Retirement
planning consumes the branch's unique admitted basis into a linear plan;
sibling clones deny planning until only one holder remains, and execution
releases that final internal obligation before reclamation.

## How It Executes

1. Observe a known branch through its owning runtime.
2. Borrow the admitted basis for fork, restore, merge, compatibility, or mutation.
3. Signal compares the complete live observation before effects.
4. A successful reference-moving operation returns a new admitted basis.
5. If the basis crosses a transport boundary, send only its descriptor and
   readmit it at the destination owner.
6. Retain an exact basis when another component must keep that state
   available. This is legitimate before or after the branch reference moves
   past it, because retention decides availability rather than currentness.
7. End the obligation: offer it to its runtime through
   `release_signal_component_basis` for an affinity-checked receipt, consume it
   directly through `SignalBranchRetentionLease::release` for the same receipt
   without a runtime, or drop it for the same terminal accounting without one.
8. To retire a branch, pass its sole admitted basis by value once no external
   obligation remains. A successful plan owns that basis until retirement
   executes.

Owner, definition, lifecycle, snapshot, and generation failures remain
distinct so callers can choose retry, refresh, or rejection deliberately.
Fork validates its owner identity before allocating a catalog entry. Fork,
restore, advance, and canonical merge distinguish basis denials before effects
from failures that occur only after the owner performed the operation.

## Small Example

```rust,no_run
use worth_signal::facade::branch::SignalBranchBasisReadmissionDenial;
use worth_signal::facade::{SignalGraph, SignalRuntime};

let mut runtime = SignalRuntime::<(), (), (), (), ()>::builder(SignalGraph::new())
    .with_kernel_defaults()
    .build();
let main_basis = runtime
    .observe_signal_branch_basis(runtime.current_branch())
    .expect("the current branch admits an owner basis");
let (_preview, forked) = runtime
    .fork_signal_branch("retained-history", &main_basis)
    .expect("an exact basis admits a fork")
    .into_parts();

// Capturing twice moves the branch reference. The first captured state stays
// stored; the branch now selects the second. `advance_signal_branch` moves the
// reference the same way when you have real mutations to stage.
let (_first, superseded) = runtime
    .capture_signal_branch_snapshot(&forked)
    .expect("the first capture succeeds")
    .into_parts();
let (_second, current) = runtime
    .capture_signal_branch_snapshot(&superseded)
    .expect("the second capture succeeds")
    .into_parts();

// The currentness question: refused, because the branch has moved on.
assert!(matches!(
    runtime.readmit_signal_branch_basis(superseded.descriptor().clone()),
    Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch { .. })
));

// The residency question: granted, because that exact state is still stored.
let lease = runtime
    .retain_signal_component_basis(&superseded)
    .expect("an exact stored target stays retainable after its branch moves");
assert_ne!(
    lease.retained_target(),
    current
        .observation()
        .target()
        .as_basis()
        .expect("an owner observation carries a basis target"),
    "an obligation is never a claim about whatever the branch holds now"
);

// Holding the obligation is what makes the historical target admissible again.
let readmitted = runtime
    .readmit_retained_signal_branch_basis(lease.descriptor().clone(), &lease)
    .expect("a live obligation readmits the exact target it retains");
assert_eq!(readmitted.descriptor(), lease.descriptor());
```

Two questions, one runtime, separate answers: the branch reference moved, and
the exact state it moved off is still there.

## Real Example

```rust,no_run
use worth_signal::facade::branch::{
    SignalBranchRetentionOwnerPosture, SignalBranchRetentionReleaseOutcome,
    SignalBranchRetentionTerminalOutcome,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

let mut runtime = SignalRuntime::<(), (), (), (), ()>::builder(SignalGraph::new())
    .with_kernel_defaults()
    .build();
let main_basis = runtime
    .observe_signal_branch_basis(runtime.current_branch())
    .expect("the current branch admits an owner basis");
let (_preview, forked) = runtime
    .fork_signal_branch("retained-history", &main_basis)
    .expect("an exact basis admits a fork")
    .into_parts();
let (_first, superseded) = runtime
    .capture_signal_branch_snapshot(&forked)
    .expect("the first capture succeeds")
    .into_parts();
let (_second, _current) = runtime
    .capture_signal_branch_snapshot(&superseded)
    .expect("the second capture succeeds")
    .into_parts();

// The same exact target may carry more than one independent obligation.
let first = runtime
    .retain_signal_component_basis(&superseded)
    .expect("an exact stored target stays retainable");
let second = runtime
    .retain_signal_component_basis(&superseded)
    .expect("a second obligation over the same exact target is legitimate");

// The runtime lane checks owner affinity before spending the obligation.
let receipt = match runtime.release_signal_component_basis(first) {
    SignalBranchRetentionReleaseOutcome::Released(receipt) => receipt,
    // A denial hands the still-live obligation back rather than dissolving it.
    // Letting this binding fall out of scope would drop-release the very
    // obligation you were just told you still hold.
    SignalBranchRetentionReleaseOutcome::Denied { lease, denial } => {
        panic!("this runtime issued the obligation: {denial:?} {lease:?}")
    }
};
assert_eq!(
    receipt.outcome(),
    SignalBranchRetentionTerminalOutcome::Released
);
// One obligation still covers this exact target, and it is the only obligation
// anywhere on this branch.
assert_eq!(receipt.remaining_target_leases(), 1);
assert_eq!(receipt.remaining_branch_leases(), 1);

// Dropping the second is the same terminal path; only the receipt is forgone.
drop(second);
// Cause is a partition; `terminal_releases()` is exactly its two buckets summed.
let counts = runtime.signal_component_retention_terminal_counts();
assert_eq!(counts.explicit_releases(), 1);
assert_eq!(counts.dropped_releases(), 1);
assert_eq!(counts.terminal_releases(), 2);
assert_eq!(counts.owner_loss_releases(), 0);
assert_eq!(counts.unknown_lease_defenses(), 0);

// An obligation may outlive its runtime. The ledger survives; the state does not.
let orphan = runtime
    .retain_signal_component_basis(&superseded)
    .expect("an exact stored target stays retainable");
let witness = runtime
    .retain_signal_component_basis(&superseded)
    .expect("a second obligation over the same exact target is legitimate");
drop(runtime);

assert_eq!(
    orphan.owner_posture(),
    SignalBranchRetentionOwnerPosture::Lost
);
// No runtime is left to offer this back to, so consume the obligation itself
// and still get a governed receipt for it.
let receipt = orphan.release();
assert_eq!(
    receipt.outcome(),
    SignalBranchRetentionTerminalOutcome::OwnerUnavailable,
    "owner loss is recorded distinctly from an ordinary release"
);

// That one release moved two independent axes: it is an explicit release by
// cause, and an owner-loss release by posture. The overlay is not a third
// bucket, so it is not added into `terminal_releases()`.
let counts = witness.owner_terminal_counts();
assert_eq!(counts.explicit_releases(), 2);
assert_eq!(counts.dropped_releases(), 1);
assert_eq!(counts.terminal_releases(), 3);
assert_eq!(counts.owner_loss_releases(), 1);
```

The runtime is authoritative throughout. Retaining the superseded basis never
pretends it is still current; it only keeps that exact state available, and the
obligation ends exactly once however it ends.

The two snippets above are compiled against the real facade on every
`cargo test`, so their types, method names, and outcome variants cannot drift
away from the runtime. They are not executed: a `SignalRuntime` does not fit
the main-thread stack a doctest binary gets.

The workflow they narrate, and the exact totals they assert, are executed by
[`examples/branch_bases.rs`](./examples/branch_bases.rs), which also covers the
retained readmission lane. The crate manifest declares that example a test
target, so it runs as part of an ordinary package test run and of a
workspace-wide one:

```text
cargo test -p worth-signal
```

To watch the same workflow instead of only seeing it pass:

```text
cargo run -p worth-signal --example branch_bases
```

## How It Relates To Other Features

- Transactions remain Signal's mutation engine. `advance_signal_branch`
  supplies exact branch admission and returns the resulting basis together
  with the transaction receipt.
- Existing Signal-local merge planning and execution consume exact admitted
  source and target bases. Successful execution returns the merge result with
  the newly admitted target basis; raw branch handles remain private engine
  inputs.
- `capture_signal_branch_snapshot` consumes an exact admitted expectation and
  returns both an owner-bound snapshot and the post-capture basis.
  `restore_signal_branch` validates both exact owner basis and snapshot owner,
  then returns the post-restore basis. Portable snapshot data must use the
  separate pristine-runtime reconstruction lane before ordinary restore can
  accept it.
- Branch retirement is the counterweight to retention. It is denied while any
  external obligation remains, and it reclaims every exact stored target that
  no obligation retains.
- Runtime Bridge may carry and reuse Signal-issued bases, but it does not mint
  them or become Signal authority.
- Foundational defines the runtime-neutral branch reference grammar. Signal
  owns liveness, readmission, retention, and all effects.
- Relational branch bases are independent owner artifacts. There is no
  combined Relational-plus-Signal authority in this milestone.

## Inspection And Debugging

Inspect the admitted basis's `observation()` when diagnosing a rejection. Its
branch identity, target basis, and generation show which axes were expected.
Readmission reports foreign owner, definition drift, retired or unknown branch,
unavailable snapshot, and reference mismatch separately.

For an obligation, ask it directly:

- `retained_target()` and `descriptor()` give the exact target as of
  acquisition, never the branch's current one.
- `owner_posture()` distinguishes `Live` from `Lost`.
- `owner_terminal_counts()` reads the issuing owner's ledger, and stays
  readable after that runtime is gone.

For the owner-side view, `signal_component_retention_terminal_counts` reports
cause and owner posture as independent axes; see
[Reading The Terminal Counters](#reading-the-terminal-counters) before treating
its four numbers as a flat tally. When a release receipt surprises you, read
`remaining_target_leases` and `remaining_branch_leases` as the different
questions they are.

For lifecycle failures, inspect the typed retention or retirement denial.
Do not infer liveness from graph diagnostic output; ask the runtime owner.

## Anti-Patterns

- Do not pass a `SignalBranchBasisDescriptor` to a governed operation. Readmit
  it first.
- Do not cache a raw branch ID and reconstruct expected currentness yourself.
- Do not use graph diagnostics as a second branch catalog.
- Do not treat `Clone` on an admitted basis as a new observation.
- Do not pass deserialized `SignalSnapshotV1` data directly to ordinary
  restore; only an owner-issued admitted snapshot opens that door.
- Do not read a lease as a claim that its branch reference is still current. It
  says that one exact immutable state is still available.
- Do not read a lease as durability. Once its owner is gone the ledger stays
  readable, but the retained state is not recoverable.
- Do not let a `SignalBranchRetentionReleaseOutcome::Denied { lease, .. }`
  binding fall out of scope. That denial handed you a live obligation, and
  dropping it releases the retention you were just told you still hold.
- Do not drop the last admitted basis for a state you will need before
  retaining it; ordinary readmission will refuse to give it back.
- Do not treat an unreleased lease as a leak to be defended against with
  `std::mem::forget` or a second bookkeeping layer. Drop is terminal.
- Do not sum `owner_loss_releases` alongside `explicit_releases` and
  `dropped_releases` as though the three were disjoint. It overlays the same
  events those two buckets already partition.
- Do not create a second Signal graph for Query-installed operations hosted by
  Runtime Bridge.

## Current Limits

- Branch state, descriptors awaiting readmission, and retention accounting are
  memory-resident. Restart durability is deferred to Worth Store integration.
- Automatic rebase, distributed reference movement, and offline synchronization
  are not supported.
- Signal and Relational operations do not form a cross-owner atomic commit.
- Cross-runtime semantic merge, undo/redo, and persistent branch recovery
  remain future work.

## Owner Component Port

The public methods described earlier in this guide are current
`SignalRuntime` compatibility methods. Phase 3 also exports the
`ManagedSignalBranchReference` vocabulary, but the concrete basis, mutation,
and lifecycle port methods are not public yet. Their frozen later contract is
[`OWNER_SERVICES.md`](./OWNER_SERVICES.md); that contract is not an availability
claim.

When those services stabilize, an integrating owner may consume only the
owner-issued managed reference, public admitted basis, descriptor,
fork outcome, readmission/compatibility denials, retention lease outcomes,
linear retirement plan, and the admitted basis returned by capture, restore,
or advance. Fork, capture, restore, and advance return typed mismatch,
retention, and no-movement outcomes rather than flattened owner strings;
owner preflight acquires the retention needed to issue the successor basis, so
basis construction after a performed canonical operation is an internal
invariant rather than a caller-visible fallible phase. It should treat a typed
no-movement outcome as leaving the old reference current; once Signal returns
the new basis, that basis is the post-operation reference. Retention transfers
through the explicit external lease object and never through a descriptor or
raw branch ID. The managed reference identifies one owner and one branch-cell
incarnation across branch-state movement; it does not carry exact-state or
retention authority.

The later ports do not expose Signal graph internals, legacy head tuples,
private snapshot storage, or authority minting. They become the component
boundary for composite-owner work only after facade stabilization; they do not
provide cross-owner atomicity.

Milestone 9.17.2 may carry a `SignalBranchBasisDescriptor` as descriptive
comparison data alongside either a `ManagedSignalBranchReference` for current
readmission or the exact `SignalBranchRetentionLease` that already preserves a
historical target. It asks the Signal owner to validate the applicable pair,
retain the resulting `AdmittedSignalBranchBasis`, compare
two admitted bases through `validate_signal_basis_compatibility`, and consume
the typed owner outcome from fork, capture, restore, advance, retention, or
retirement. Merge remains an inherited `SignalRuntime` compatibility operation
outside the three future owner ports; preserving it does not add a mutation-port
method or a second owner-state lane. The integrating owner may not construct a
basis, infer one from a snapshot ID or
digest, call the private graph/branch manager, or publish a product reference
as though Signal issued it.

Signal's current compatibility methods are synchronous and do not expose a
generic external cancellation token. The later owner ports accept the concrete
`SignalOwnerCancellationToken` at their pre-movement cutoff. Once a call
returns a successor admitted basis, Signal movement is performed and cannot be
relabeled as cancelled; a typed denial that says no movement preserves the
predecessor. Milestone 9.17.2 must use this boundary in its own
no-half-publication protocol rather than attempting raw Signal rollback.

The integrating owner must acquire `retain_signal_component_basis` before it
promises component residency, and it must acquire while it still holds an
admitted basis for that exact state. Retention is available whether or not the
branch reference has already moved past the target, and it never claims that
reference is current. While the obligation is live, the owner may ask for the
exact retained basis back through `readmit_retained_signal_branch_basis`;
ordinary `readmit_signal_branch_basis` answers the currentness question and
will refuse. The later current readmission route requires the managed reference;
retained-exact readmission instead relies on the concrete owner-bound lease and
does not add a currentness or managed-reference prerequisite. A descriptor by
itself remains descriptive and opens neither route.

The obligation ends exactly once, through any of three surfaces, and an
integrating owner is therefore not required to build leak defenses around it.
While the owner still holds the issuing runtime it should prefer
`release_signal_component_basis`, which checks owner affinity: a release
offered to a runtime that did not issue the obligation is denied with the live
lease handed back, so the integrator must rebind that returned lease rather
than discard it. Where no runtime is in hand, `SignalBranchRetentionLease::release`
consumes the obligation directly and returns the same governed receipt.
Dropping the lease is equally terminal and simply forgoes the receipt.

An obligation that outlives its Signal runtime reports a `Lost` owner posture,
and releasing it reports an `OwnerUnavailable` terminal outcome. Since the
issuing runtime no longer exists to be called,
`SignalBranchRetentionLease::release` is the path by which an integrator can
still obtain that governed receipt. The surviving ledger is observability, not
a promise that the retained state can still be produced.

All Signal branch catalog state, admitted bases, stored branch snapshots, and
retention accounting remain memory-resident. Restart durability is deferred to
Worth Store integration; the port does not expose a serializable authority
token or persistence promise.

## Related Docs

- [Signal documentation index](./DOCS.md)
- [Snapshots, branches, and history](./docs/guides/snapshots-branches-and-history.md)
- [Foundational branch references](../worth-foundational/docs/branching-merging-and-commit-vocabulary/branch-references.md)
- [Authority and workflow contracts](../worth-proof/docs/features/authority-and-workflow-contracts.md)
