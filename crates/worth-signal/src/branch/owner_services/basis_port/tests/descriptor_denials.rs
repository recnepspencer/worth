use worth_foundational::FoundationalBranchReferenceMismatchAxis;

use crate::branch::{
    ManagedSignalBranchReferenceAdmissionDenial, SignalBranchBasisObservationDenial,
    SignalBranchBasisReadmissionDenial, SignalBranchRetentionAcquisitionDenial,
};

use super::world::{
    advance_exact, basis_port_world, issue_reference, observation_with_generation,
    target_changed_descriptor,
};

#[test]
fn foreign_managed_authority_denies_before_receiving_owner_registry_contact() {
    let issuer = basis_port_world();
    let receiver = basis_port_world();
    let reference = issue_reference(&issuer.port, &issuer.basis_b);
    let before = receiver
        .port
        .owner_service_cost_snapshot()
        .expect("the receiving owner reports its baseline");

    assert!(matches!(
        receiver.port.observe_current(&reference),
        Err(SignalBranchBasisObservationDenial::ManagedReferenceDenied {
            denial: ManagedSignalBranchReferenceAdmissionDenial::ForeignOwner,
        })
    ));
    assert!(matches!(
        receiver.port.compare_current_exact(&issuer.basis_b),
        Err(SignalBranchBasisReadmissionDenial::OwnerMismatch { .. })
    ));
    assert!(matches!(
        receiver.port.retain_exact(&issuer.basis_b),
        Err(SignalBranchRetentionAcquisitionDenial::ForeignBasis)
    ));
    let after = receiver
        .port
        .owner_service_cost_snapshot()
        .expect("the receiving owner reports its denial work");
    assert_eq!(
        after.owner_upgrade_attempts(),
        before.owner_upgrade_attempts() + 4,
        "three operation upgrades and the after-inspection upgrade are explicit work"
    );
    assert_eq!(
        after.branch_registry_lookups(),
        before.branch_registry_lookups()
    );
    assert_eq!(after.target_cell_contacts(), before.target_cell_contacts());
    assert_eq!(
        after.retention_registry_contacts(),
        before.retention_registry_contacts()
    );
}

#[test]
fn complete_descriptor_comparison_distinguishes_every_frozen_mismatch_axis() {
    let world = basis_port_world();
    let foreign = basis_port_world();
    let reference = issue_reference(&world.port, &world.basis_b);
    let descriptor = world.basis_b.descriptor();
    let target = descriptor
        .observation()
        .target()
        .as_basis()
        .expect("the production descriptor carries an exact target");

    assert!(matches!(
        world
            .port
            .readmit_exact(&reference, foreign.basis_b.descriptor()),
        Err(SignalBranchBasisReadmissionDenial::OwnerMismatch { .. })
    ));

    let definition_mismatch = target_changed_descriptor(
        descriptor,
        target.graph_instance_id(),
        target.definition_basis() + 1,
        target.snapshot_id(),
    );
    assert!(matches!(
        world.port.readmit_exact(&reference, &definition_mismatch),
        Err(SignalBranchBasisReadmissionDenial::DefinitionMismatch { .. })
    ));

    assert!(matches!(
        world
            .port
            .readmit_exact(&reference, world.basis_a.descriptor()),
        Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch { axes })
            if axes == vec![FoundationalBranchReferenceMismatchAxis::BranchIdentity]
    ));

    let snapshot_mismatch = target_changed_descriptor(
        descriptor,
        target.graph_instance_id(),
        target.definition_basis(),
        Some(target.snapshot_id().unwrap_or(0) + 991),
    );
    assert!(matches!(
        world.port.readmit_exact(&reference, &snapshot_mismatch),
        Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch { axes })
            if axes == vec![FoundationalBranchReferenceMismatchAxis::TargetBasis]
    ));

    let generation_mismatch =
        observation_with_generation(descriptor, descriptor.observation().generation().get() + 1);
    assert!(matches!(
        world.port.readmit_exact(&reference, &generation_mismatch),
        Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch { axes })
            if axes == vec![FoundationalBranchReferenceMismatchAxis::ReferenceGeneration]
    ));
}

#[test]
fn canonical_movement_stales_exact_basis_but_not_its_managed_reference() {
    let world = basis_port_world();
    let reference = issue_reference(&world.port, &world.basis_b);
    let stale = world.basis_b.descriptor().clone();
    let moved = advance_exact(&world.port, &world.basis_b);

    assert!(matches!(
        world.port.compare_current_exact(&world.basis_b),
        Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch { axes })
            if axes.contains(&FoundationalBranchReferenceMismatchAxis::ReferenceGeneration)
    ));
    assert!(matches!(
        world.port.readmit_exact(&reference, &stale),
        Err(SignalBranchBasisReadmissionDenial::ReferenceMismatch { axes })
            if axes.contains(&FoundationalBranchReferenceMismatchAxis::ReferenceGeneration)
    ));
    let current = world
        .port
        .observe_current(&reference)
        .expect("the managed reference follows movement in the same cell incarnation");
    assert_eq!(current.observation(), &moved);
}
