use worth_foundational::FoundationalBranchReferenceMismatchAxis;
use worth_signal::facade::branch::{SignalBranchAdvanceDenial, SignalBranchBasisReadmissionDenial};

use super::observation::neutral_basis;
use super::world::{populated_world, CargoContext, CargoOutputs};

#[test]
fn cargo_routing_baseline_is_real_and_publicly_observable() {
    let (world, outputs) = populated_world();

    assert_eq!(outputs, CargoOutputs::baseline());
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
    let (world, _) = populated_world();
    let expected = world.main_basis.clone();
    let mut context = CargoContext::storm_front();
    let (outcome, outputs) = world
        .advance(
            &expected,
            &mut context,
            world.nodes.storm,
            super::world::STORM_SEVERITY,
        )
        .expect("the storm input advances through the public owner port");

    assert_eq!(outputs, CargoOutputs::storm_front());
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
        world.services.mutation_port().advance_exact(
            &expected,
            &mut CargoContext::storm_front(),
            &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            |_| Ok(())
        ),
        Err(SignalBranchAdvanceDenial::BasisMismatch { ref axes })
            if axes.contains(&FoundationalBranchReferenceMismatchAxis::ReferenceGeneration)
    ));
    assert!(matches!(
        world
            .services
            .basis_port()
            .readmit_exact(&world.reference(&next), expected.descriptor()),
        Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch { ref axes })
            if axes.contains(&FoundationalBranchReferenceMismatchAxis::ReferenceGeneration)
    ));
}
