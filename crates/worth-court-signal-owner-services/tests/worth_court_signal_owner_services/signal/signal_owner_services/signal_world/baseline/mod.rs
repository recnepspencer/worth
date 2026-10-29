pub(crate) use {worth_foundational::FoundationalBranchReferenceMismatchAxis};
pub(crate) use {worth_signal::facade::branch::SignalBranchAdvanceDenial, worth_signal::facade::branch::SignalBranchBasisReadmissionDenial};

pub(crate) use {crate::signal_owner_services::ordinary_context::signal_world::observation::neutral_basis};
pub(crate) use {crate::signal_owner_services::ordinary_context::signal_world::world::populated_world, crate::signal_owner_services::ordinary_context::signal_world::world::CargoContext, crate::signal_owner_services::ordinary_context::signal_world::world::CargoOutputs};

#[test]
fn cargo_routing_baseline_is_real_and_publicly_observable() {
    let (world, outputs) = populated_world();

    assert_eq!(outputs, crate::signal_owner_services::ordinary_context::signal_world::world::CargoOutputs::baseline());
    let neutral = neutral_basis(&world.main_basis);
    assert_eq!(neutral.owner_branch_id, world.main_branch.id.0);
    assert!(!neutral.branch_identity.is_empty());
    assert!(!neutral.graph_instance_id.is_empty());
    assert_eq!(neutral.generation, 1);
    assert_eq!(
        neutral.lifecycle,
        world.main_basis.descriptor().lifecycle_posture()
    );
    assert!(world
        .services
        .basis_port()
        .compare_current_exact(&world.main_basis)
        .is_ok());

    let reference = world.reference(&world.main_basis);
    let observed = world
        .observe(&reference)
        .expect("the public observation returns the canonical branch basis");
    assert_eq!(neutral_basis(&observed), neutral);
}

#[test]
fn cargo_routing_mutation_changes_effects_and_stales_the_consumed_basis() {
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
    let mut context = crate::signal_owner_services::ordinary_context::signal_world::world::CargoContext::storm_front();
    let (outcome, outputs) = world
        .advance(
            &expected,
            &mut context,
            world.nodes.storm,
            crate::signal_owner_services::ordinary_context::signal_world::world::STORM_SEVERITY,
        )
        .expect("the storm input advances through the public owner port");

    assert_eq!(outputs, crate::signal_owner_services::ordinary_context::signal_world::world::CargoOutputs::storm_front());
    let next = outcome.into_basis();
    assert_eq!(
        neutral_basis(&next).generation,
        neutral_basis(&expected).generation + 1
    );
    assert!(world
        .services
        .basis_port()
        .compare_current_exact(&next)
        .is_ok());
    assert!(matches!(
        world.services.mutation_port().advance_exact(request_execution,
            &expected,
            &mut crate::signal_owner_services::ordinary_context::signal_world::world::CargoContext::storm_front(),
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            |_| Ok(())
        ),
        Err(worth_signal::facade::branch::SignalBranchAdvanceDenial::BasisMismatch { ref axes })
            if axes.contains(&worth_foundational::FoundationalBranchReferenceMismatchAxis::ReferenceGeneration)
    ));
    assert!(matches!(
        world
            .services
            .basis_port()
            .readmit_exact(&world.reference(&next), expected.descriptor()),
        Err(worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::ReferenceMismatch { ref axes })
            if axes.contains(&worth_foundational::FoundationalBranchReferenceMismatchAxis::ReferenceGeneration)
    ));
}
