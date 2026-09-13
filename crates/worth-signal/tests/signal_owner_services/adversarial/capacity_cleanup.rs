//! Scheduled resource-bound evidence.  The large cases stay ignored in the
//! ordinary court but are selected by the Scale roster with `--ignored`.

use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;

use worth_signal::facade::branch::{
    SignalBranchAdvanceDenial, SignalBranchRetentionAcquisitionDenial,
    SignalBranchRetentionReleaseOutcome, SignalOwnerCancellationSource,
};

use super::world::{AdversarialWorld, PROGRESS_BOUND};

#[derive(Default)]
struct ReleaseGate {
    released: Mutex<bool>,
    changed: Condvar,
}

impl ReleaseGate {
    fn wait(&self) {
        let mut released = self
            .released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !*released {
            released = self
                .changed
                .wait(released)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn release(&self) {
        let mut released = self
            .released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *released = true;
        self.changed.notify_all();
    }
}

struct ReleaseOnDrop(Arc<ReleaseGate>);

impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        self.0.release();
    }
}

#[test]
#[ignore = "Scale: exhaust the fixed 64-operation owner bound"]
fn operation_capacity_denies_the_65th_and_restores_after_release() {
    let world = AdversarialWorld::new();
    let mut bases = vec![world.child_basis.clone()];
    for ordinal in 0..63 {
        let name = format!("operation-capacity-{ordinal}");
        let basis = world
            .mutation
            .fork_exact(
                worth_signal::facade::branch::validate_signal_branch_name(name)
                    .expect("the generated capacity identity is valid"),
                &world.root_basis,
                &SignalOwnerCancellationSource::new().token(),
            )
            .expect("the first 64 live branches are admitted")
            .into_parts()
            .1;
        bases.push(basis);
    }

    let release = Arc::new(ReleaseGate::default());
    let _release_guard = ReleaseOnDrop(Arc::clone(&release));
    let (ready_tx, ready_rx) = mpsc::sync_channel(64);
    let mutation = world.mutation.clone();
    let mut workers = Vec::with_capacity(64);
    for basis in bases {
        let mutation = mutation.clone();
        let release = Arc::clone(&release);
        let ready_tx = ready_tx.clone();
        workers.push(thread::spawn(move || {
            let result = mutation
                .advance_exact(
                    &basis,
                    &mut (),
                    &SignalOwnerCancellationSource::new().token(),
                    move |_| {
                        ready_tx
                            .send(())
                            .expect("the capacity court receives every held operation");
                        release.wait();
                        Ok(())
                    },
                )
                .map(|_| ())
                .map_err(|denial| format!("{denial:?}"));
            assert_eq!(result, Ok(()));
        }));
    }
    for _ in 0..64 {
        ready_rx
            .recv_timeout(PROGRESS_BOUND)
            .expect("each admitted operation reaches its held callback");
    }

    let denied = mutation.advance_exact(
        &world.root_basis,
        &mut (),
        &SignalOwnerCancellationSource::new().token(),
        |_| panic!("capacity denial must precede the callback"),
    );
    assert!(matches!(
        denied,
        Err(SignalBranchAdvanceDenial::OperationCapacityExhausted {
            maximum_in_flight_operations: 64
        })
    ));
    release.release();
    for worker in workers {
        worker.join().expect("each held operation releases cleanly");
    }

    world
        .mutation
        .advance_exact(
            &world.root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("operation slots return exactly when held calls finish");
}

#[test]
#[ignore = "Scale: exhaust the fixed 4,096 owner retention bound"]
fn retention_capacity_denies_then_all_releases_restore_one_lease() {
    let world = AdversarialWorld::new();
    // The two exact bases held by this world already occupy two admitted
    // retention slots.  The external ledger therefore has 4,094 available
    // slots before it must report the configured 4,096-owner bound.
    let mut leases = Vec::with_capacity(4_094);
    for _ in 0..4_094 {
        leases.push(
            world
                .basis
                .retain_exact(&world.child_basis)
                .expect("the configured retention capacity admits 4,094 external leases"),
        );
    }
    assert!(matches!(
        world.basis.retain_exact(&world.child_basis),
        Err(SignalBranchRetentionAcquisitionDenial::CapacityExhausted {
            maximum_active_leases: 4_096
        })
    ));

    for lease in leases {
        assert!(matches!(
            world.basis.release_exact(lease),
            SignalBranchRetentionReleaseOutcome::Released(_)
        ));
    }
    let final_lease = world
        .basis
        .retain_exact(&world.child_basis)
        .expect("all terminal releases return the exact retention slots");
    assert!(matches!(
        world.basis.release_exact(final_lease),
        SignalBranchRetentionReleaseOutcome::Released(_)
    ));
}

#[test]
#[ignore = "Scale: exhaust the fixed 4,096 live-branch bound"]
fn live_branch_capacity_denies_then_retirement_restores_one_slot() {
    let world = AdversarialWorld::new();
    // A returned basis owns one bounded admitted lease. Keep only the basis
    // needed for retirement; the registry itself owns every live branch cell.
    let mut released = world.child_basis.clone();
    for ordinal in 0..4_094 {
        let name = format!("live-branch-{ordinal}");
        released = world
            .mutation
            .fork_exact(
                worth_signal::facade::branch::validate_signal_branch_name(name)
                    .expect("the generated branch identity is valid"),
                &world.root_basis,
                &SignalOwnerCancellationSource::new().token(),
            )
            .expect("the first 4,096 live branches are admitted")
            .into_parts()
            .1;
    }

    assert!(matches!(
        world.mutation.fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name("live-branch-overflow")
                .expect("the overflow identity is valid"),
            &world.root_basis,
            &SignalOwnerCancellationSource::new().token(),
        ),
        Err(worth_signal::facade::branch::SignalBranchForkOperationDenial::LiveBranchCapacityExhausted {
            maximum_live_branches: 4_096
        })
    ));

    let plan = match world.lifecycle.plan_retirement_exact(
        released,
        worth_signal::facade::branch::SignalBranchRetirementReason::Superseded,
    ) {
        worth_proof::TransitionOutcome::Success(plan) => plan,
        other => panic!("the cleanup branch remains retireable: {other:?}"),
    };
    assert!(matches!(
        world
            .lifecycle
            .retire_exact(plan, &SignalOwnerCancellationSource::new().token(),),
        worth_proof::TransitionOutcome::Success(_)
    ));

    world
        .mutation
        .fork_exact(
            worth_signal::facade::branch::validate_signal_branch_name("live-branch-recovered")
                .expect("the recovered identity is valid"),
            &world.root_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("retirement returns one live-branch slot");
}
