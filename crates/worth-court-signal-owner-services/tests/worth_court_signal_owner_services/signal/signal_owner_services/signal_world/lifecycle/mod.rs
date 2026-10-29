pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_signal::facade::branch::SignalBranchRetirementReason};
pub(crate) use {worth_signal::facade::branch::ManagedSignalBranchReferenceAdmissionDenial, worth_signal::facade::branch::SignalBranchAdvanceDenial, worth_signal::facade::branch::SignalBranchBasisObservationDenial, worth_signal::facade::branch::SignalBranchRetirementDenial, worth_signal::facade::branch::SignalOwnerCancellationSource, worth_signal::facade::branch::SignalOwnerLifecycleObservation};

pub(crate) use {crate::signal_owner_services::ordinary_context::signal_world::observation::neutral_basis};
pub(crate) use {crate::signal_owner_services::ordinary_context::signal_world::world::populated_world, crate::signal_owner_services::ordinary_context::signal_world::world::CargoContext, crate::signal_owner_services::ordinary_context::signal_world::world::CargoOutputs};

#[test]
fn same_branch_stale_and_retired_postures_are_exact_and_recoverable() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (world, _) = populated_world();
    let expected = world.main_basis.clone();
    let mut storm_context = crate::signal_owner_services::ordinary_context::signal_world::world::CargoContext::storm_front();
    let (first, outputs) = world
        .advance(
            &expected,
            &mut storm_context,
            world.nodes.storm,
            crate::signal_owner_services::ordinary_context::signal_world::world::STORM_SEVERITY,
        )
        .expect("the first same-branch operation performs the real evaluation");
    assert_eq!(outputs, crate::signal_owner_services::ordinary_context::signal_world::world::CargoOutputs::storm_front());
    assert_eq!(
        neutral_basis(first.advanced_basis()).generation,
        neutral_basis(&expected).generation + 1
    );

    let mut stale_context = crate::signal_owner_services::ordinary_context::signal_world::world::CargoContext::berth_maintenance();
    let stale = world.services.mutation_port().advance_exact(
        request_execution,
        &expected,
        &mut stale_context,
        &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
        |_| panic!("a stale basis must be denied before the caller callback"),
    );
    assert!(matches!(
        stale,
        Err(worth_signal::facade::branch::SignalBranchAdvanceDenial::BasisMismatch { .. })
    ));

    let reference = world.reference(first.advanced_basis());
    let observed = world
        .observe(&reference)
        .expect("the winning operation leaves a healthy canonical cell");
    assert_eq!(
        neutral_basis(&observed),
        neutral_basis(first.advanced_basis())
    );
}

#[test]
fn unrelated_branch_mutations_make_progress_and_keep_effects_separate() {
    let (world, _) = populated_world();
    let source = world.main_basis.clone();
    let (storm_branch, storm_basis) = world.fork(&source, "storm-branch");
    let (maintenance_branch, maintenance_basis) = world.fork(&source, "maintenance-branch");
    assert_ne!(storm_branch.id, maintenance_branch.id);

    let storm_reference = world.reference(&storm_basis);
    let maintenance_reference = world.reference(&maintenance_basis);

    let mut storm_context = crate::signal_owner_services::ordinary_context::signal_world::world::CargoContext::storm_front();
    let (storm, storm_outputs) = world
        .advance(
            &storm_basis,
            &mut storm_context,
            world.nodes.storm,
            crate::signal_owner_services::ordinary_context::signal_world::world::STORM_SEVERITY,
        )
        .expect("the storm branch advances through the public owner port");
    let mut maintenance_context = crate::signal_owner_services::ordinary_context::signal_world::world::CargoContext::berth_maintenance();
    let (maintenance, maintenance_outputs) = world
        .advance(
            &maintenance_basis,
            &mut maintenance_context,
            world.nodes.berth,
            crate::signal_owner_services::ordinary_context::signal_world::world::BERTH_AVAILABILITY,
        )
        .expect("the maintenance branch advances independently");
    assert_eq!(storm_outputs, crate::signal_owner_services::ordinary_context::signal_world::world::CargoOutputs::storm_front());
    assert_eq!(maintenance_outputs, crate::signal_owner_services::ordinary_context::signal_world::world::CargoOutputs::berth_maintenance());

    let storm_after = world
        .observe(&storm_reference)
        .expect("storm branch remains publicly observable");
    let maintenance_after = world
        .observe(&maintenance_reference)
        .expect("maintenance branch remains publicly observable");
    assert_eq!(storm_after.branch_id(), storm_branch.id);
    assert_eq!(maintenance_after.branch_id(), maintenance_branch.id);
    assert_eq!(
        neutral_basis(&storm_after).generation,
        neutral_basis(&storm_basis).generation + 1
    );
    assert_eq!(
        neutral_basis(&maintenance_after).generation,
        neutral_basis(&maintenance_basis).generation + 1
    );
    assert_ne!(
        neutral_basis(&storm_after).branch_identity,
        neutral_basis(&maintenance_after).branch_identity
    );
    assert_eq!(
        neutral_basis(storm.advanced_basis()).generation,
        neutral_basis(&storm_after).generation
    );
    assert_eq!(
        neutral_basis(maintenance.advanced_basis()).generation,
        neutral_basis(&maintenance_after).generation
    );

    let mut storm_followup_context = crate::signal_owner_services::ordinary_context::signal_world::world::CargoContext::baseline();
    let (storm_followup, storm_followup_outputs) = world
        .advance(
            storm.advanced_basis(),
            &mut storm_followup_context,
            world.nodes.storm,
            crate::signal_owner_services::ordinary_context::signal_world::world::STORM_SEVERITY,
        )
        .expect("the storm branch remains independently writable after maintenance");
    assert_eq!(storm_followup_outputs, crate::signal_owner_services::ordinary_context::signal_world::world::CargoOutputs::baseline());
    let storm_followup_reference = world.reference(storm_followup.advanced_basis());
    let storm_followup_after = world
        .observe(&storm_followup_reference)
        .expect("the follow-up observation remains on the storm branch");
    assert_eq!(
        neutral_basis(&storm_followup_after).generation,
        neutral_basis(storm_followup.advanced_basis()).generation
    );
}

#[test]
fn retired_child_denies_new_work_while_the_owner_and_unrelated_branch_remain_healthy() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let (mut world, _) = populated_world();
    let (child, child_basis) = world.fork(&world.main_basis, "retirement-child");
    let child_reference = world.reference(&child_basis);
    let main_reference = world.reference(&world.main_basis);
    let lifecycle = world.services.lifecycle_port();
    let plan = match lifecycle
        .plan_retirement_exact(child_basis, worth_signal::facade::branch::SignalBranchRetirementReason::Superseded)
    {
        worth_proof::TransitionOutcome::Success(plan) => plan,
        other => panic!("the child retirement plan must be owner-issued: {other:?}"),
    };
    let receipt = match lifecycle.retire_exact(plan, &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token())
    {
        worth_proof::TransitionOutcome::Success(receipt) => receipt,
        other => panic!("the child retirement must perform: {other:?}"),
    };
    assert_eq!(receipt.retired_branch().id, child.id);

    assert!(matches!(
        world.observe(&child_reference),
        Err(worth_signal::facade::branch::SignalBranchBasisObservationDenial::ManagedReferenceDenied {
            denial: worth_signal::facade::branch::ManagedSignalBranchReferenceAdmissionDenial::BranchLifecycleEnded,
        })
    ));
    let main_observed = world
        .observe(&main_reference)
        .expect("retiring a child cannot strand its unrelated parent");
    assert_eq!(main_observed.branch_id(), world.main_branch.id);
    assert!(matches!(
        lifecycle.plan_retirement_exact(
            world.main_basis.clone(),
            worth_signal::facade::branch::SignalBranchRetirementReason::Rejected,
        ),
        worth_proof::TransitionOutcome::Denied(worth_signal::facade::branch::SignalBranchRetirementDenial::CurrentBranch { branch_id })
            if branch_id == world.main_branch.id
    ));

    world.close_owner();
    assert_eq!(
        lifecycle.owner_lifecycle_observation(),
        worth_signal::facade::branch::SignalOwnerLifecycleObservation::Closed
    );
    assert!(matches!(
        world.observe(&main_reference),
        Err(worth_signal::facade::branch::SignalBranchBasisObservationDenial::OwnerUnavailable(_))
    ));
    assert!(matches!(
        world.services.mutation_port().advance_exact(
            request_execution,
            &world.main_basis,
            &mut crate::signal_owner_services::ordinary_context::signal_world::world::CargoContext::baseline(),
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            |_| Ok(())
        ),
        Err(worth_signal::facade::branch::SignalBranchAdvanceDenial::OwnerUnavailable(_))
    ));
    assert!(lifecycle.owner_service_cost_snapshot().is_err());
}
