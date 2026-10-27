//! Retaining one exact Signal component basis after its branch has moved on.
//!
//! This is the runnable owner workflow behind `BRANCH_BASES.md`. It uses only
//! `worth_signal::facade`, so everything here is available to an integrating
//! component.
//!
//! Run it with:
//!
//! ```text
//! cargo run -p worth-signal --example branch_bases
//! ```

use worth_signal::facade::branch::{
    AdmittedSignalBranchBasis, SignalBranchBasisReadmissionDenial,
    SignalBranchRetainedReadmissionDenial, SignalBranchRetentionOwnerPosture,
    SignalBranchRetentionReleaseOutcome, SignalBranchRetentionTerminalOutcome, SignalBranchTarget,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

type ExampleRuntime = SignalRuntime<(), (), (), (), ()>;

pub fn main() {
    // A `SignalRuntime` is a large value and Windows gives a debug binary's main
    // thread a 1 MiB stack, so this example runs its workflow on a thread with
    // room for one. That is a build accommodation, not part of the owner
    // contract below.
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(owner_workflow)
        .expect("the example workflow thread starts")
        .join()
        .expect("the example workflow thread completes");
}

fn owner_workflow() {
    let mut runtime = ExampleRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let (superseded, current) = branch_with_a_superseded_target(&mut runtime);

    residency_is_not_currentness(&runtime, &superseded, &current);
    explicit_release_returns_governed_evidence(&runtime, &superseded);
    dropping_an_obligation_releases_it(&runtime, &superseded);
    an_obligation_outlives_its_owner(runtime, superseded);
    internal_obligations_stay_out_of_the_external_ledger();

    println!("branch_bases: every owner claim in BRANCH_BASES.md held.");
}

/// Move a branch reference forward so its earlier state is stored but no longer
/// current. Capturing twice is the smallest honest way to get there: the first
/// captured state stays in the runtime's stored snapshots while the branch goes
/// on to select the second. `advance_signal_branch` moves the reference the
/// same way when you have real mutations to stage.
fn branch_with_a_superseded_target(
    runtime: &mut ExampleRuntime,
) -> (AdmittedSignalBranchBasis, AdmittedSignalBranchBasis) {
    let main_basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .expect("the current branch admits an owner basis");
    let (_branch, forked) = runtime
        .fork_signal_branch("retained-history", &main_basis)
        .expect("an exact basis admits a fork")
        .into_parts();

    let (first_snapshot, superseded) = runtime
        .capture_signal_branch_snapshot(&forked)
        .expect("the first capture succeeds")
        .into_parts();
    let (second_snapshot, current) = runtime
        .capture_signal_branch_snapshot(&superseded)
        .expect("the second capture succeeds")
        .into_parts();

    // Admitted bases and admitted snapshots each carry their own internal
    // branch obligation, which is what keeps retirement from reclaiming a state
    // somebody still holds. Drop the ones we are done with.
    drop(forked);
    drop(first_snapshot);
    drop(second_snapshot);

    assert_ne!(
        superseded.observation().target(),
        current.observation().target(),
        "the second capture must move the branch off the first exact target"
    );
    (superseded, current)
}

/// Residency and currentness are two different questions asked of one runtime.
///
/// Ordinary readmission asks "does this branch still select this state?" and
/// refuses once the reference moves. Retention asks "is this exact immutable
/// state still available?" and says yes. Currentness never denies retention;
/// the branch's current head only ever adds availability evidence.
fn residency_is_not_currentness(
    runtime: &ExampleRuntime,
    superseded: &AdmittedSignalBranchBasis,
    current: &AdmittedSignalBranchBasis,
) {
    // The currentness question: refused, because the branch has moved on.
    assert!(matches!(
        runtime.readmit_signal_branch_basis(superseded.descriptor().clone()),
        Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch { .. })
    ));

    // The residency question: granted, because the exact state is still stored.
    let lease = runtime
        .retain_signal_component_basis(superseded)
        .expect("an exact stored target stays retainable after its branch moves");
    assert_eq!(
        lease.retained_target(),
        basis_target(superseded),
        "the obligation names the exact target it was opened over"
    );
    assert_ne!(
        lease.retained_target(),
        basis_target(current),
        "an obligation is never a claim about whatever the branch holds now"
    );
    assert_eq!(
        lease.owner_posture(),
        SignalBranchRetentionOwnerPosture::Live
    );

    // Holding the obligation is what makes the historical target admissible
    // again. Acquire it before you drop the last admitted basis for a state you
    // will need: ordinary readmission answers the currentness question above,
    // so without a live obligation there is no route back.
    let readmitted = runtime
        .readmit_retained_signal_branch_basis(lease.descriptor().clone(), &lease)
        .expect("a live obligation readmits the exact target it retains");
    assert_eq!(readmitted.descriptor(), lease.descriptor());

    // The obligation authorizes one exact target, not any target.
    assert!(matches!(
        runtime.readmit_retained_signal_branch_basis(current.descriptor().clone(), &lease),
        Err(SignalBranchRetainedReadmissionDenial::DescriptorMismatch)
    ));

    drop(readmitted);
    drop(lease);
}

/// The runtime lane is the explicit release to prefer while you hold the issuing
/// runtime: it checks owner affinity before spending the obligation.
fn explicit_release_returns_governed_evidence(
    runtime: &ExampleRuntime,
    superseded: &AdmittedSignalBranchBasis,
) {
    let before = runtime
        .signal_component_retention_terminal_counts()
        .explicit_releases();
    let first = runtime
        .retain_signal_component_basis(superseded)
        .expect("an exact stored target stays retainable");
    let second = runtime
        .retain_signal_component_basis(superseded)
        .expect("the same exact target may carry more than one obligation");

    let receipt = match runtime.release_signal_component_basis(first) {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => receipt,
        // A denial hands the still-live obligation back rather than dissolving
        // it. Letting that binding fall out of scope here would drop-release
        // the very obligation you were just told you still hold.
        SignalBranchRetentionReleaseOutcome::Denied { lease, denial } => {
            panic!("this runtime issued the obligation: {denial:?} {lease:?}")
        }
    };
    assert_eq!(receipt.released_target(), basis_target(superseded));
    assert_eq!(
        receipt.outcome(),
        SignalBranchRetentionTerminalOutcome::Released
    );
    // One obligation over this exact target remains, and it is the only
    // obligation anywhere on this branch.
    assert_eq!(receipt.remaining_target_leases(), 1);
    assert_eq!(receipt.remaining_branch_leases(), 1);

    let receipt = match runtime.release_signal_component_basis(second) {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => receipt,
        other => panic!("this runtime issued the obligation: {other:?}"),
    };
    assert_eq!(receipt.remaining_target_leases(), 0);
    assert_eq!(receipt.remaining_branch_leases(), 0);

    let counts = runtime.signal_component_retention_terminal_counts();
    assert_eq!(counts.explicit_releases(), before + 2);
    assert_eq!(counts.unknown_lease_defenses(), 0);
}

/// Dropping an obligation is not a leak. It takes the same exactly-once
/// terminal path as an explicit release, and only the receipt is forgone.
fn dropping_an_obligation_releases_it(
    runtime: &ExampleRuntime,
    superseded: &AdmittedSignalBranchBasis,
) {
    let before = runtime.signal_component_retention_terminal_counts();
    let lease = runtime
        .retain_signal_component_basis(superseded)
        .expect("an exact stored target stays retainable");
    drop(lease);

    let after = runtime.signal_component_retention_terminal_counts();
    assert_eq!(after.explicit_releases(), before.explicit_releases());
    assert_eq!(after.dropped_releases(), before.dropped_releases() + 1);
    assert_eq!(after.terminal_releases(), before.terminal_releases() + 1);
    assert_eq!(after.unknown_lease_defenses(), 0);
}

/// An obligation may outlive the runtime that issued it. The owner ledger keeps
/// the release observable; it does not keep the retained state available.
fn an_obligation_outlives_its_owner(
    runtime: ExampleRuntime,
    superseded: AdmittedSignalBranchBasis,
) {
    let descriptor = superseded.descriptor().clone();
    let orphan = runtime
        .retain_signal_component_basis(&superseded)
        .expect("an exact stored target stays retainable");
    let witness = runtime
        .retain_signal_component_basis(&superseded)
        .expect("a second obligation over the same exact target is legitimate");

    // Read the owner ledger while the owner is still there to read it.
    let before = runtime.signal_component_retention_terminal_counts();
    drop(superseded);
    drop(runtime);

    assert_eq!(
        orphan.owner_posture(),
        SignalBranchRetentionOwnerPosture::Lost
    );
    // The obligation still describes what it retained, but no live owner can
    // hand that state back: the ledger survives the runtime, the data does not.
    assert_eq!(orphan.descriptor(), &descriptor);

    // No runtime is left to offer this back to.
    // `SignalBranchRetentionLease::release` needs none, so an orphaned
    // obligation can still be ended with a governed receipt.
    let receipt = orphan.release();
    assert_eq!(
        receipt.outcome(),
        SignalBranchRetentionTerminalOutcome::OwnerUnavailable,
        "owner loss is recorded distinctly from an ordinary release"
    );
    assert_eq!(
        receipt.remaining_target_leases(),
        1,
        "the witness still holds one"
    );

    // The ledger outlives the runtime, so the loss stays observable.
    //
    // That single release moved two independent axes at once: by cause it is an
    // explicit release, and by owner posture it is an owner-loss release. The
    // posture counter overlays the cause buckets rather than forming a third
    // one, so `terminal_releases()` advances by one, not two.
    let counts = witness.owner_terminal_counts();
    assert_eq!(counts.explicit_releases(), before.explicit_releases() + 1);
    assert_eq!(counts.dropped_releases(), before.dropped_releases());
    assert_eq!(counts.terminal_releases(), before.terminal_releases() + 1);
    assert_eq!(
        counts.owner_loss_releases(),
        before.owner_loss_releases() + 1
    );
    assert_eq!(counts.unknown_lease_defenses(), 0);
    drop(witness);
}
/// `BRANCH_BASES.md`'s Real Example asserts absolute counter values rather than
/// deltas, on a fresh runtime. Those exact numbers are only true if the
/// obligation an admitted basis and an admitted snapshot each carry internally
/// never reaches the external terminal ledger. The phases above deliberately
/// assert deltas, so they cannot see that difference; this one replays the
/// guide's walkthrough exactly — including holding the fork handle and both
/// admitted snapshots alive across the release, the drop, and the loss of the
/// runtime, as the guide's own bindings do — and asserts the published numbers.
fn internal_obligations_stay_out_of_the_external_ledger() {
    let mut runtime = ExampleRuntime::builder(SignalGraph::new())
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

    one_explicit_release_and_one_drop_read_as_published(&runtime, &superseded);
    owner_loss_overlays_the_published_cause_buckets(runtime, &superseded);
}

/// The guide's first counter block: one explicit release and one drop, read as
/// absolute totals off a runtime that has never seen any other obligation.
fn one_explicit_release_and_one_drop_read_as_published(
    runtime: &ExampleRuntime,
    superseded: &AdmittedSignalBranchBasis,
) {
    let first = runtime
        .retain_signal_component_basis(superseded)
        .expect("an exact stored target stays retainable");
    let second = runtime
        .retain_signal_component_basis(superseded)
        .expect("a second obligation over the same exact target is legitimate");

    let receipt = match runtime.release_signal_component_basis(first) {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => receipt,
        other => panic!("this runtime issued the obligation: {other:?}"),
    };
    assert_eq!(
        receipt.outcome(),
        SignalBranchRetentionTerminalOutcome::Released
    );
    assert_eq!(receipt.remaining_target_leases(), 1);
    assert_eq!(receipt.remaining_branch_leases(), 1);

    drop(second);
    let counts = runtime.signal_component_retention_terminal_counts();
    assert_eq!(counts.explicit_releases(), 1);
    assert_eq!(counts.dropped_releases(), 1);
    assert_eq!(counts.terminal_releases(), 2);
    assert_eq!(counts.owner_loss_releases(), 0);
    assert_eq!(counts.unknown_lease_defenses(), 0);
}

/// The guide's second counter block: one release of an orphaned obligation
/// raises a cause bucket and the posture overlay together, so the published
/// totals move by one, not two.
fn owner_loss_overlays_the_published_cause_buckets(
    runtime: ExampleRuntime,
    superseded: &AdmittedSignalBranchBasis,
) {
    let orphan = runtime
        .retain_signal_component_basis(superseded)
        .expect("an exact stored target stays retainable");
    let witness = runtime
        .retain_signal_component_basis(superseded)
        .expect("a second obligation over the same exact target is legitimate");
    drop(runtime);

    assert_eq!(
        orphan.owner_posture(),
        SignalBranchRetentionOwnerPosture::Lost
    );
    let receipt = orphan.release();
    assert_eq!(
        receipt.outcome(),
        SignalBranchRetentionTerminalOutcome::OwnerUnavailable,
        "owner loss is recorded distinctly from an ordinary release"
    );

    let counts = witness.owner_terminal_counts();
    assert_eq!(counts.explicit_releases(), 2);
    assert_eq!(counts.dropped_releases(), 1);
    assert_eq!(counts.terminal_releases(), 3);
    assert_eq!(counts.owner_loss_releases(), 1);
    assert_eq!(counts.unknown_lease_defenses(), 0);
}

fn basis_target(basis: &AdmittedSignalBranchBasis) -> &SignalBranchTarget {
    basis
        .observation()
        .target()
        .as_basis()
        .expect("an owner observation carries a basis target")
}
