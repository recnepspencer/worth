use std::sync::mpsc::{self, SyncSender};
use std::thread;
use std::time::Duration;

use crate::branch::SignalBranchBasisObservationDenial;

use super::world::{basis_port_world, issue_reference};

const PROGRESS_BOUND: Duration = Duration::from_secs(2);
const PARK_FAIL_SAFE: Duration = Duration::from_secs(6);

struct ParkRelease(Option<SyncSender<()>>);

impl ParkRelease {
    fn release(&mut self) {
        if let Some(release) = self.0.take() {
            let _ = release.send(());
        }
    }
}

impl Drop for ParkRelease {
    fn drop(&mut self) {
        self.release();
    }
}

#[test]
fn parked_branch_does_not_block_unrelated_real_port_observation() {
    let world = basis_port_world();
    let reference_a = issue_reference(&world.port, &world.basis_a);
    let reference_b = issue_reference(&world.port, &world.basis_b);
    let owner = world
        .port
        .upgrade_owner()
        .expect("the sealed owner remains live");
    let setup = owner.admit().expect("cell setup admits");
    let cell_a = owner
        .lookup_cell(&setup, world.branch_a.id)
        .expect("branch A owns one canonical cell");
    drop(setup);
    let (entered_tx, entered_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let parked_owner = owner.clone();
    let parked_cell = cell_a.clone();
    let worker = thread::spawn(move || {
        let admission = parked_owner.admit().expect("parked A work admits");
        parked_cell
            .with_state(&admission, |_, _| {
                entered_tx.send(()).expect("A reports its cell hold");
                release_rx
                    .recv_timeout(PARK_FAIL_SAFE)
                    .expect("A's drop-safe controller releases the park");
            })
            .expect("the real A cell remains healthy")
    });
    let mut release = ParkRelease(Some(release_tx));
    entered_rx
        .recv_timeout(PROGRESS_BOUND)
        .expect("branch A reaches its cell-local park");
    let a_before = cell_a.cost_snapshot();

    let observed_b = world
        .port
        .observe_current(&reference_b)
        .expect("unrelated B completes through the real basis port while A is parked");
    assert_eq!(observed_b.observation(), world.basis_b.observation());
    assert_eq!(
        cell_a.cost_snapshot(),
        a_before,
        "B performs zero contact, wait, or movement on A"
    );

    release.release();
    worker.join().expect("the parked A worker exits cleanly");
    assert_eq!(
        world
            .port
            .observe_current(&reference_a)
            .expect("A has a healthy follow-up after release")
            .observation(),
        world.basis_a.observation()
    );
}

#[test]
fn same_owner_cell_callback_reentry_denies_before_second_contact() {
    let world = basis_port_world();
    let reference = issue_reference(&world.port, &world.basis_b);
    let owner = world
        .port
        .upgrade_owner()
        .expect("the sealed owner remains live");
    let admission = owner.admit().expect("the outer operation admits");
    let cell = owner
        .lookup_cell(&admission, world.branch_b.id)
        .expect("branch B owns one canonical cell");
    let before = cell.cost_snapshot();

    cell.with_state(&admission, |_, _| {
        assert!(matches!(
            world.port.observe_current(&reference),
            Err(SignalBranchBasisObservationDenial::ManagedReferenceDenied {
                denial: crate::branch::ManagedSignalBranchReferenceAdmissionDenial::OwnerReentry,
            })
        ));
    })
    .expect("the outer real cell operation remains healthy");
    let after = cell.cost_snapshot();
    assert_eq!(after.contacts(), before.contacts() + 1);
    assert_eq!(after.waits(), before.waits());
    assert_eq!(after.movements(), before.movements());
    assert!(
        world.port.observe_current(&reference).is_ok(),
        "reentry denial leaves an immediate healthy follow-up"
    );
}
