use std::sync::Arc;

use crate::branch::{
    validate_signal_branch_name, AdmittedSignalBranchBasis, SignalBranchForkOperationDenial,
};
use crate::state::SignalBranchId;

use super::super::{
    SignalBranchCellAdmissionDenial, SignalBranchRegistry, SignalBranchRegistryDenial, SignalOwner,
    SignalOwnerLifecycleState, SignalOwnerOperationAdmission, SignalOwnerServiceCounters,
};
use super::runtime_root::runtime_with_two_branches;

struct ForkReservationState<'a> {
    owner: &'a SignalOwner<(), (), ()>,
    admission: &'a SignalOwnerOperationAdmission<'a>,
    source_id: SignalBranchId,
    reservations: usize,
    reservation_counter: u64,
    children: Vec<SignalBranchId>,
}

impl<'a> ForkReservationState<'a> {
    fn capture(
        owner: &'a SignalOwner<(), (), ()>,
        admission: &'a SignalOwnerOperationAdmission<'a>,
        source_id: SignalBranchId,
    ) -> Self {
        Self {
            owner,
            admission,
            source_id,
            reservations: owner.reservation_count(),
            reservation_counter: owner.cost_snapshot().branch_registry_reservations(),
            children: owner
                .metadata
                .branch_children(admission, source_id)
                .expect("source lineage is observable"),
        }
    }

    fn assert_unchanged(&self, context: &str) {
        assert_eq!(
            self.owner.cost_snapshot().branch_registry_reservations(),
            self.reservation_counter
        );
        self.assert_capacity_and_lineage(self.reservations, &self.children, context);
    }

    fn assert_capacity_and_lineage(
        &self,
        expected_reservations: usize,
        expected_children: &[SignalBranchId],
        context: &str,
    ) {
        assert_eq!(self.owner.reservation_count(), expected_reservations);
        assert_eq!(
            self.owner
                .metadata
                .branch_children(self.admission, self.source_id)
                .expect(context),
            expected_children
        );
    }

    fn assert_healthy_cycle(
        &self,
        source_basis: &AdmittedSignalBranchBasis,
        expected_next_id: SignalBranchId,
    ) {
        let healthy = self
            .owner
            .reserve_fork_destination(
                self.admission,
                source_basis,
                validate_signal_branch_name("released-admission-healthy-fork")
                    .expect("the healthy identity is valid"),
            )
            .expect("the same admission reserves after both holds release");
        assert_eq!(healthy.branch().id, expected_next_id);
        assert_eq!(self.owner.reservation_count(), self.reservations + 1);
        assert_eq!(
            self.owner.cost_snapshot().branch_registry_reservations(),
            self.reservation_counter + 1
        );
        let mut expected_children = self.children.clone();
        expected_children.push(expected_next_id);
        expected_children.sort_unstable();
        assert_eq!(
            self.owner
                .metadata
                .branch_children(self.admission, self.source_id)
                .expect("the healthy reservation installs lineage"),
            expected_children
        );
        drop(healthy);
        self.assert_capacity_and_lineage(
            self.reservations,
            &self.children,
            "healthy reservation cleanup restores lineage",
        );
    }
}

#[test]
fn registry_and_cell_reject_foreign_and_expired_admissions_before_contact() {
    let counters = Arc::new(SignalOwnerServiceCounters::default());
    let lifecycle = SignalOwnerLifecycleState::new(51, Arc::clone(&counters));
    let owner_admission = lifecycle.admit(51).expect("owner admits work");
    let registry = SignalBranchRegistry::new(&lifecycle, 1, 1);
    let cell = registry
        .reserve(&owner_admission, SignalBranchId(1))
        .expect("owner reserves identity")
        .install(0_u64)
        .expect("owner installs state");

    let foreign_lifecycle =
        SignalOwnerLifecycleState::new(52, Arc::new(SignalOwnerServiceCounters::default()));
    let foreign_admission = foreign_lifecycle
        .admit(52)
        .expect("foreign owner admits itself");
    let next_incarnation =
        SignalOwnerLifecycleState::new(51, Arc::new(SignalOwnerServiceCounters::default()));
    let expired_admission = next_incarnation
        .admit(51)
        .expect("new lifecycle admits itself");

    assert_eq!(
        registry
            .lookup(&foreign_admission, SignalBranchId(1))
            .unwrap_err(),
        SignalBranchRegistryDenial::ForeignOwner
    );
    assert_eq!(
        registry
            .lookup(&expired_admission, SignalBranchId(1))
            .unwrap_err(),
        SignalBranchRegistryDenial::ExpiredAdmission
    );
    assert_eq!(
        cell.with_state(&foreign_admission, |_, _| ()).unwrap_err(),
        SignalBranchCellAdmissionDenial::ForeignOwner
    );
    assert_eq!(
        cell.with_state(&expired_admission, |_, _| ()).unwrap_err(),
        SignalBranchCellAdmissionDenial::ExpiredLifecycle
    );
    assert_eq!(
        cell.acquire_fork_source_custody(&foreign_admission).err(),
        Some(SignalBranchCellAdmissionDenial::ForeignOwner)
    );
    assert_eq!(
        cell.acquire_fork_source_custody(&expired_admission).err(),
        Some(SignalBranchCellAdmissionDenial::ExpiredLifecycle)
    );
    let snapshot = counters.snapshot();
    assert_eq!(snapshot.branch_registry_lookups(), 0);
    assert_eq!(snapshot.target_cell_contacts(), 0);
    assert_eq!(snapshot.target_cell_waits(), 0);
    let cell_snapshot = cell.cost_snapshot();
    assert_eq!(cell_snapshot.contacts(), 0);
    assert_eq!(cell_snapshot.waits(), 0);
}

#[test]
fn fork_reservation_rejects_held_admission_before_identity_or_capacity_movement() {
    let (mut runtime, first_branch, source_branch, source_basis) = runtime_with_two_branches();
    let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = mutation.upgrade_owner().expect("sealed owner upgrades");
    let admission = owner.admit().expect("owner admits fork reservation work");
    let source_cell = owner
        .lookup_cell(&admission, source_branch.id)
        .expect("the real source cell is live");
    let reservation_state = ForkReservationState::capture(&owner, &admission, source_branch.id);

    let held_cell_denial = source_cell
        .with_state(&admission, |_, _| {
            owner.reserve_fork_destination(
                &admission,
                &source_basis,
                validate_signal_branch_name("held-cell-denied-fork")
                    .expect("the denied identity is valid"),
            )
        })
        .expect("the source cell hold is real");
    assert!(matches!(
        held_cell_denial,
        Err(SignalBranchForkOperationDenial::OwnerCellMisuse { branch_id })
            if branch_id == source_branch.id
    ));

    let metadata_hold = admission
        .hold_owner_metadata()
        .expect("the admission holds the real owner-metadata slot");
    let held_metadata_denial = owner.reserve_fork_destination(
        &admission,
        &source_basis,
        validate_signal_branch_name("held-metadata-denied-fork")
            .expect("the second denied identity is valid"),
    );
    drop(metadata_hold);
    assert!(matches!(
        held_metadata_denial,
        Err(SignalBranchForkOperationDenial::OwnerCellMisuse { branch_id })
            if branch_id == source_branch.id
    ));

    reservation_state.assert_unchanged("denied reservations leave lineage unchanged");

    let expected_next_id = SignalBranchId(
        first_branch
            .id
            .0
            .max(source_branch.id.0)
            .checked_add(1)
            .expect("the small real fixture has another branch identity"),
    );
    reservation_state.assert_healthy_cycle(&source_basis, expected_next_id);
}

#[test]
fn prior_and_fresh_admissions_reject_same_thread_same_owner_cell_reentry() {
    let (mut runtime, branch_a, branch_b, _) = runtime_with_two_branches();
    let (basis, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = basis.upgrade_owner().expect("owner remains live");
    let first = owner.admit().expect("first admission is idle");
    let second = owner
        .admit()
        .expect("second admission exists before the first acquires a cell");
    let cell_a = owner
        .lookup_cell(&first, branch_a.id)
        .expect("branch A is live");
    let cell_b = owner
        .lookup_cell(&second, branch_b.id)
        .expect("branch B is live");

    cell_a
        .with_state(&first, |_, _| {
            assert_eq!(
                cell_b.with_state(&second, |_, _| ()).unwrap_err(),
                SignalBranchCellAdmissionDenial::ExecutingThreadReentry
            );
            assert_eq!(
                cell_a.with_state(&second, |_, _| ()).unwrap_err(),
                SignalBranchCellAdmissionDenial::ExecutingThreadReentry
            );
            assert_eq!(
                owner.admit().unwrap_err(),
                super::super::SignalOwnerAdmissionDenial::OwnerReentry
            );
        })
        .expect("first admission holds branch A");

    cell_b
        .with_state(&second, |state, _| {
            assert_eq!(state.branch_id(), branch_b.id)
        })
        .expect("other branch remains healthy after the rejected callback reentry");
}
