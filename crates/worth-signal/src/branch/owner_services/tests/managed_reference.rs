use std::panic::{catch_unwind, AssertUnwindSafe};

use worth_proof::TransitionOutcome;

use crate::branch::{
    ManagedSignalBranchReference, ManagedSignalBranchReferenceAdmissionDenial,
    SignalBranchAdvanceDenial, SignalBranchBasisObservationDenial,
    SignalBranchBasisReadmissionDenial, SignalBranchRetirementReason,
    SignalBranchSnapshotCaptureDenial,
};

use super::runtime_root::runtime_with_two_branches;

#[test]
fn owner_issued_reference_reenters_one_live_cell_without_retaining_exact_state() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ManagedSignalBranchReference>();

    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let expected = basis.observation().clone();
    let (port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = port.upgrade_owner().expect("sealed owner remains live");
    assert_eq!(owner.admitted_retention_count(branch.id), 1);

    let reference = owner
        .issue_managed_branch_reference(&basis)
        .expect("a real admitted basis issues its branch reference");
    let cloned = reference.clone();
    drop(basis);
    assert_eq!(
        owner.admitted_retention_count(branch.id),
        0,
        "a managed reference opens no exact-basis retention obligation"
    );

    let before_observation = owner.cost_snapshot();
    let observed = owner
        .observe_managed_branch_reference(&cloned)
        .expect("managed admission reaches the canonical cell");
    let after_observation = owner.cost_snapshot();
    assert_eq!(observed, expected);
    assert_eq!(
        after_observation.branch_registry_lookups(),
        before_observation.branch_registry_lookups() + 1,
        "managed observation performs one checked registry lookup"
    );
    assert_eq!(
        after_observation.target_cell_contacts(),
        before_observation.target_cell_contacts() + 1,
        "managed observation consumes that checked cell without a raw-id retry"
    );
    assert_eq!(
        format!("{reference:?}"),
        "ManagedSignalBranchReference { .. }"
    );
}

#[test]
fn equal_looking_branch_numbers_from_another_owner_are_denied_by_affinity() {
    let (mut runtime_a, _, branch_a, basis_a) = runtime_with_two_branches();
    let (mut runtime_b, _, branch_b, _basis_b) = runtime_with_two_branches();
    assert_eq!(
        branch_a.id, branch_b.id,
        "the hostile twin reuses the raw number"
    );
    let (port_a, _, _) = runtime_a.owner_port_slots().expect("owner A seals");
    let (port_b, _, _) = runtime_b.owner_port_slots().expect("owner B seals");
    let owner_a = port_a.upgrade_owner().expect("owner A remains live");
    let owner_b = port_b.upgrade_owner().expect("owner B remains live");
    let reference = owner_a
        .issue_managed_branch_reference(&basis_a)
        .expect("owner A issues from its real admitted basis");

    assert!(matches!(
        owner_b.admit_managed_branch_reference(&reference),
        Err(ManagedSignalBranchReferenceAdmissionDenial::ForeignOwner)
    ));
    assert!(matches!(
        owner_b.observe_managed_branch_reference(&reference),
        Err(SignalBranchBasisObservationDenial::ManagedReferenceDenied {
            denial: ManagedSignalBranchReferenceAdmissionDenial::ForeignOwner
        })
    ));
    assert!(owner_a.admit_managed_branch_reference(&reference).is_ok());
}

#[test]
fn managed_reference_denial_serializes_inside_the_frozen_readmission_outcome() {
    let denial = SignalBranchBasisReadmissionDenial::ManagedReferenceDenied {
        denial: ManagedSignalBranchReferenceAdmissionDenial::BranchIncarnationReplaced,
    };
    let encoded = serde_json::to_string(&denial).expect("descriptive denial serializes");
    let decoded: SignalBranchBasisReadmissionDenial =
        serde_json::from_str(&encoded).expect("descriptive denial round-trips");
    assert_eq!(decoded, denial);
}

#[test]
fn foreign_affinity_wins_even_after_the_issuing_owner_closes() {
    let (mut runtime_a, _, branch_a, basis_a) = runtime_with_two_branches();
    let (mut runtime_b, _, branch_b, _basis_b) = runtime_with_two_branches();
    assert_eq!(
        branch_a.id, branch_b.id,
        "the hostile twin reuses the raw number"
    );
    let (port_a, _, _) = runtime_a.owner_port_slots().expect("owner A seals");
    let (port_b, _, _) = runtime_b.owner_port_slots().expect("owner B seals");
    let owner_a = port_a.upgrade_owner().expect("owner A remains live");
    let owner_b = port_b.upgrade_owner().expect("owner B remains live");
    let reference = owner_a
        .issue_managed_branch_reference(&basis_a)
        .expect("owner A issues from its real admitted basis");
    drop(basis_a);
    drop(runtime_a);
    let issuer_before = owner_a.cost_snapshot();
    let receiver_before = owner_b.cost_snapshot();

    assert!(matches!(
        owner_b.admit_managed_branch_reference(&reference),
        Err(ManagedSignalBranchReferenceAdmissionDenial::ForeignOwner)
    ));
    assert_eq!(owner_a.cost_snapshot(), issuer_before);
    assert_eq!(owner_b.cost_snapshot(), receiver_before);
}

#[test]
fn canonical_movement_stales_exact_basis_without_staling_managed_reference() {
    let (mut runtime, _, _, basis) = runtime_with_two_branches();
    let old_observation = basis.observation().clone();
    let (port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = port.upgrade_owner().expect("sealed owner remains live");
    let reference = owner
        .issue_managed_branch_reference(&basis)
        .expect("a real admitted basis issues its branch reference");
    let (admission, cell) = owner
        .admit_managed_branch_reference(&reference)
        .expect("the live reference reaches its checked cell");
    let mut runtime_context = ();
    let moved = cell
        .advance_exact::<(), (), _>(
            &admission,
            &basis,
            &mut runtime_context,
            &super::super::SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the exact basis moves its canonical cell");
    let (moved_observation, _) = moved.into_parts();
    drop(admission);

    let observed = owner
        .observe_managed_branch_reference(&reference)
        .expect("the stable reference reaches the moved cell incarnation");
    assert_eq!(observed, moved_observation);
    assert!(observed.compare(&old_observation).is_err());

    let retry_admission = owner.admit().expect("stale retry admits the live owner");
    let movements_before_retry = cell.cost_snapshot().movements();
    let mut callback_ran = false;
    let retry = cell.advance_exact::<(), (), _>(
        &retry_admission,
        &basis,
        &mut runtime_context,
        &super::super::SignalOwnerCancellationSource::new().token(),
        |_| {
            callback_ran = true;
            Ok(())
        },
    );
    assert!(matches!(
        retry,
        Err(crate::branch::SignalBranchAdvanceDenial::BasisMismatch { .. })
    ));
    assert!(!callback_ran, "stale exact authority cannot run mutation");
    assert_eq!(cell.cost_snapshot().movements(), movements_before_retry);
}

#[test]
fn retirement_invalidates_the_reference_for_the_consumed_branch_incarnation() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let plan = match runtime.plan_signal_branch_retirement(
        branch.clone(),
        basis,
        SignalBranchRetirementReason::Rejected,
    ) {
        TransitionOutcome::Success(plan) => plan,
        other => panic!("real owner should issue retirement authority: {other:?}"),
    };
    let (port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = port.upgrade_owner().expect("sealed owner remains live");
    let reference = owner
        .issue_managed_branch_reference(plan.admitted_basis())
        .expect("the live branch issues a managed reference");
    let admission = owner.admit().expect("retirement admits");
    owner
        .begin_retirement(&admission, branch.id)
        .expect("the registry owns the referenced incarnation")
        .execute_exact(
            plan,
            &super::super::SignalOwnerCancellationSource::new().token(),
        )
        .expect("cell admission remains valid")
        .expect("retirement performs");
    drop(admission);

    assert!(matches!(
        owner.admit_managed_branch_reference(&reference),
        Err(ManagedSignalBranchReferenceAdmissionDenial::BranchLifecycleEnded)
    ));
}

#[test]
fn managed_reference_treats_matching_owner_close_as_terminal() {
    let (mut runtime, _, _, basis) = runtime_with_two_branches();
    let (port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = port.upgrade_owner().expect("sealed owner remains live");
    let reference = owner
        .issue_managed_branch_reference(&basis)
        .expect("the owner issues the reference");
    drop(basis);
    drop(runtime);

    assert!(matches!(
        owner.admit_managed_branch_reference(&reference),
        Err(ManagedSignalBranchReferenceAdmissionDenial::OwnerUnavailable(_))
    ));
}

#[test]
fn transaction_panic_quarantines_managed_readmission_without_unknown_branch() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = port.upgrade_owner().expect("sealed owner remains live");
    let reference = owner
        .issue_managed_branch_reference(&basis)
        .expect("the live basis issues a managed reference");
    owner
        .observe_managed_reference_for_readmission(&reference)
        .expect("the healthy checked-cell readmission step observes");
    let admission = owner.admit().expect("panic operation admits");
    let cell = owner
        .lookup_cell(&admission, branch.id)
        .expect("the referenced cell is installed");
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = cell.advance_exact::<(), (), _>(
            &admission,
            &basis,
            &mut (),
            &super::super::SignalOwnerCancellationSource::new().token(),
            |_| panic!("inject contained transaction panic"),
        );
    }));
    assert!(panic.is_err());
    drop(admission);

    assert!(matches!(
        owner.observe_managed_reference_for_readmission(&reference),
        Err(SignalBranchBasisReadmissionDenial::QuarantinedBranch { branch_id })
            if branch_id == branch.id
    ));
    let retry_admission = owner.admit().expect("quarantined owner still admits");
    let mut callback_ran = false;
    assert!(matches!(
        cell.advance_exact::<(), (), _>(
            &retry_admission,
            &basis,
            &mut (),
            &super::super::SignalOwnerCancellationSource::new().token(),
            |_| {
                callback_ran = true;
                Ok(())
            },
        ),
        Err(SignalBranchAdvanceDenial::QuarantinedBranch { branch_id })
            if branch_id == branch.id
    ));
    assert!(!callback_ran);
}

#[test]
fn carried_managed_and_snapshot_admissions_preserve_precise_reentry_denials() {
    let (mut runtime, _, branch, basis) = runtime_with_two_branches();
    let (port, _, _) = runtime.owner_port_slots().expect("runtime seals");
    let owner = port.upgrade_owner().expect("owner remains live");
    let first = owner.admit().expect("first admission is idle");
    let prior = owner
        .admit()
        .expect("the prior admission exists before the first cell hold");
    let cell = owner
        .lookup_cell(&first, branch.id)
        .expect("the exact branch cell is live");

    cell.with_state(&first, |_, _| {
        assert!(matches!(
            owner.issue_managed_branch_reference_with_admission(&prior, &basis),
            Err(ManagedSignalBranchReferenceAdmissionDenial::OwnerReentry)
        ));
        assert!(matches!(
            owner.issue_managed_branch_reference_with_admission(&first, &basis),
            Err(ManagedSignalBranchReferenceAdmissionDenial::OwnerCellMisuse)
        ));
        assert!(matches!(
            owner.metadata.reserve_snapshot(&prior, &cell),
            Err(SignalBranchSnapshotCaptureDenial::OwnerReentry)
        ));
        assert!(matches!(
            owner.metadata.reserve_snapshot(&first, &cell),
            Err(SignalBranchSnapshotCaptureDenial::OwnerCellMisuse { branch_id })
                if branch_id == branch.id
        ));
    })
    .expect("the disputed reentry runs inside a real cell hold");

    let healthy = owner
        .metadata
        .reserve_snapshot(&prior, &cell)
        .expect("reentry denial neither reserves nor leaks snapshot capacity");
    drop(healthy);
}
