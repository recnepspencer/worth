use super::*;
use crate::branch::{
    relational_branch_observation, RelationalBranchBasisMismatchAxis, RelationalBranchVersion,
};
use crate::history::data::BranchId;
use crate::runtime::{RelationalRuntime, RelationalRuntimeConfig};

#[derive(serde::Serialize)]
struct DescriptorTransportPayload {
    descriptor_version: u16,
    runtime_instance_id: u64,
    branch_id: BranchId,
    reference: RelationalBranchReferenceObservation,
    truth_version: RelationalBranchVersion,
    root_identity: u64,
    schema_commitment: [u8; 32],
    visibility_commitment: [u8; 32],
    posture: RelationalBranchBasisPosture,
}

enum DescriptorTransportMutation {
    DescriptorVersion(u16),
    Posture(RelationalBranchBasisPosture),
    Branch(BranchId),
    Reference(RelationalBranchReferenceObservation),
    TruthVersion(RelationalBranchVersion),
    RootIdentity(u64),
    Schema([u8; 32]),
    Visibility([u8; 32]),
}

fn hostile_transport_descriptor(
    descriptor: &RelationalBranchBasisDescriptor,
    mutation: DescriptorTransportMutation,
) -> RelationalBranchBasisDescriptor {
    let mut payload = DescriptorTransportPayload {
        descriptor_version: descriptor.descriptor_version(),
        runtime_instance_id: descriptor.runtime_instance_id(),
        branch_id: descriptor.branch_id().clone(),
        reference: descriptor.reference().clone(),
        truth_version: descriptor.truth_version(),
        root_identity: descriptor.root_identity(),
        schema_commitment: descriptor.schema_commitment(),
        visibility_commitment: descriptor.visibility_commitment(),
        posture: descriptor.posture(),
    };
    match mutation {
        DescriptorTransportMutation::DescriptorVersion(value) => payload.descriptor_version = value,
        DescriptorTransportMutation::Posture(value) => payload.posture = value,
        DescriptorTransportMutation::Branch(value) => payload.branch_id = value,
        DescriptorTransportMutation::Reference(value) => payload.reference = value,
        DescriptorTransportMutation::TruthVersion(value) => payload.truth_version = value,
        DescriptorTransportMutation::RootIdentity(value) => payload.root_identity = value,
        DescriptorTransportMutation::Schema(value) => payload.schema_commitment = value,
        DescriptorTransportMutation::Visibility(value) => payload.visibility_commitment = value,
    }
    let encoded = rmp_serde::to_vec_named(&payload).expect("hostile descriptor payload encodes");
    rmp_serde::from_slice(&encoded).expect("transport accepts descriptive hostile payload")
}

fn runtime() -> RelationalRuntime {
    RelationalRuntime::new(RelationalRuntimeConfig::default())
}

#[test]
fn observation_and_clones_share_one_repeatable_empty_basis() {
    let runtime = runtime();
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    let observation = basis.observation();
    let cloned = basis.clone();

    assert_eq!(descriptor.root_identity(), 0);
    assert_eq!(observation.selected_root_identity(), 0);
    assert_eq!(observation.descriptor(), cloned.descriptor());
    assert_eq!(
        basis.retention_reason(),
        crate::history::retention::RelationalBasisRetentionReason::Observation
    );
    assert_eq!(
        basis.admission_identity(),
        cloned.admission_identity(),
        "clones preserve the owner-issued admission identity"
    );
}

#[test]
fn repeated_observation_shares_one_registry_entry_and_final_drop_removes_it() {
    let runtime = runtime();
    let identity = runtime.main_branch_identity();
    assert_eq!(
        runtime
            .branch_basis_cost_counters()
            .retained_basis_registry_entries,
        0
    );

    let (_, first) = runtime.observe_branch(&identity).unwrap();
    let (_, second) = runtime.observe_branch(&identity).unwrap();
    assert_eq!(
        first.admission_identity(),
        second.admission_identity(),
        "the weak exact-basis registry reuses the live owner admission"
    );
    assert_eq!(
        runtime
            .branch_basis_cost_counters()
            .retained_basis_registry_entries,
        1
    );
    drop(first);
    assert_eq!(
        runtime
            .branch_basis_cost_counters()
            .retained_basis_registry_entries,
        1
    );
    drop(second);
    assert_eq!(
        runtime
            .branch_basis_cost_counters()
            .retained_basis_registry_entries,
        0
    );
}

#[test]
fn retained_basis_readmits_after_owner_publication_moves_the_reference() {
    let runtime = crate::tests::support::runtime_with_test_schema();
    crate::tests::support::create_entity_outcome(&runtime, "retained-basis-before");
    let identity = runtime.main_branch_identity();
    let (descriptor, retained) = runtime.observe_branch(&identity).unwrap();
    crate::tests::support::create_entity_outcome(&runtime, "retained-basis-after");
    let (current_descriptor, current) = runtime.observe_branch(&identity).unwrap();

    let readmitted = runtime.readmit_branch_basis(&descriptor).unwrap();
    assert_eq!(readmitted.descriptor(), retained.descriptor());
    assert_eq!(
        readmitted.observation().reference().generation(),
        descriptor.reference().generation()
    );
    assert_ne!(
        current_descriptor.root_identity(),
        descriptor.root_identity()
    );
    assert_ne!(current.descriptor(), retained.descriptor());
}

#[test]
fn observation_opens_the_same_snapshot_after_reference_movement() {
    let runtime = crate::tests::support::runtime_with_test_schema();
    crate::tests::support::create_entity_outcome(&runtime, "snapshot-before-movement");
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let observation = basis.observation();
    let before = runtime
        .snapshots()
        .snapshot_for_observation(&observation)
        .unwrap();
    crate::tests::support::create_entity_outcome(&runtime, "snapshot-after-movement");
    let after = runtime
        .snapshots()
        .snapshot_for_observation(&observation)
        .unwrap();

    assert_eq!(before.branch_id, after.branch_id);
    assert_eq!(before.version_id, after.version_id);
    assert!(observation.selected_root_identity() > 0);
}

#[test]
fn external_pin_retains_once_and_release_consumes_the_obligation() {
    let runtime = crate::tests::support::runtime_with_test_schema();
    crate::tests::support::create_entity(&runtime, "external-pin-before");
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    let lease = runtime.retain_component_basis(&basis).unwrap();
    drop(basis);
    crate::tests::support::create_entity(&runtime, "external-pin-after");

    let readmitted = runtime.readmit_branch_basis(&descriptor).unwrap();
    drop(readmitted);
    let release = runtime.release_component_basis(lease).unwrap();
    assert_eq!(release.descriptor(), &descriptor);
    assert!(matches!(
        runtime.readmit_branch_basis(&descriptor),
        Err(RelationalBranchBasisDenial::UnavailableRetainedTarget)
    ));
}

#[test]
fn reset_retention_owner_reports_unavailable_readmission_and_terminal_release() {
    let mut runtime = crate::tests::support::runtime_with_test_schema();
    crate::tests::support::create_entity(&runtime, "owner-reset-before");
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    let lease = runtime.retain_component_basis(&basis).unwrap();
    drop(basis);

    runtime.set_retention_capacity_for_test(128, 128);

    assert!(matches!(
        runtime.readmit_retained_branch_basis(&descriptor, &lease),
        Err(RelationalBranchBasisDenial::UnavailableRetainedTarget)
    ));
    let receipt = runtime.release_component_basis(lease).unwrap();
    assert_eq!(
        receipt.outcome(),
        crate::history::retention::RelationalBranchRetentionTerminalOutcome::OwnerUnavailable
    );
}

#[test]
fn superseded_unretained_descriptor_denies_without_reconstructing_authority() {
    let runtime = crate::tests::support::runtime_with_test_schema();
    crate::tests::support::create_entity(&runtime, "stale-before");
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    drop(basis);
    crate::tests::support::create_entity(&runtime, "stale-after");

    let before = runtime.branch_basis_cost_counters();
    assert!(matches!(
        runtime.readmit_branch_basis(&descriptor),
        Err(RelationalBranchBasisDenial::UnavailableRetainedTarget)
    ));
    let after = runtime.branch_basis_cost_counters();
    assert_eq!(after.readmission_denials, before.readmission_denials + 1);
    assert_eq!(
        after.stale_readmission_denials,
        before.stale_readmission_denials
    );
}

#[test]
fn foreign_runtime_descriptor_denies_before_lookup() {
    let source = runtime();
    let source_identity = source.main_branch_identity();
    let (descriptor, _) = source.observe_branch(&source_identity).unwrap();
    let foreign = runtime();

    assert!(matches!(
        foreign.readmit_branch_basis(&descriptor),
        Err(RelationalBranchBasisDenial::ForeignRuntime { .. })
    ));
}

#[test]
fn archived_and_unsupported_descriptors_deny_distinctly() {
    let runtime = runtime();
    let identity = runtime.main_branch_identity();
    let (descriptor, _) = runtime.observe_branch(&identity).unwrap();

    let archived = hostile_transport_descriptor(
        &descriptor,
        DescriptorTransportMutation::Posture(RelationalBranchBasisPosture::Archived),
    );
    assert!(matches!(
        runtime.readmit_branch_basis(&archived),
        Err(RelationalBranchBasisDenial::ArchivedBranch(_))
    ));

    let unsupported = hostile_transport_descriptor(
        &descriptor,
        DescriptorTransportMutation::DescriptorVersion(99),
    );
    assert!(matches!(
        runtime.readmit_branch_basis(&unsupported),
        Err(RelationalBranchBasisDenial::UnsupportedDescriptorVersion { actual: 99, .. })
    ));

    assert_eq!(runtime.history.branch_count(), 1);
    assert_eq!(identity.branch_id(), &BranchId("main".to_owned()));
}

#[test]
fn malformed_truth_and_root_identity_substitutions_deny_distinctly() {
    let runtime = crate::tests::support::runtime_with_test_schema();
    crate::tests::support::create_entity_outcome(&runtime, "phase6-axis-basis");
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    drop(basis);

    let malformed = hostile_transport_descriptor(
        &descriptor,
        DescriptorTransportMutation::Branch(BranchId("other".to_owned())),
    );
    assert!(matches!(
        runtime.readmit_branch_basis(&malformed),
        Err(RelationalBranchBasisDenial::MalformedDescriptor)
    ));

    let wrong_truth_version = hostile_transport_descriptor(
        &descriptor,
        DescriptorTransportMutation::TruthVersion(RelationalBranchVersion::new(
            descriptor.truth_version().as_u64() + 1,
        )),
    );
    assert!(matches!(
        runtime.readmit_branch_basis(&wrong_truth_version),
        Err(RelationalBranchBasisDenial::WrongBranchLocalTruthVersion)
    ));

    let wrong_root = hostile_transport_descriptor(
        &descriptor,
        DescriptorTransportMutation::RootIdentity(descriptor.root_identity() + 1),
    );
    assert!(matches!(
        runtime.readmit_branch_basis(&wrong_root),
        Err(RelationalBranchBasisDenial::MixedAxis(
            RelationalBranchBasisMismatchAxis::RootIdentity
        ))
    ));
}

#[test]
fn schema_and_visibility_commitment_substitutions_deny_distinctly() {
    let runtime = crate::tests::support::runtime_with_test_schema();
    crate::tests::support::create_entity_outcome(&runtime, "phase6-commitment-basis");
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    drop(basis);

    let mut schema = descriptor.schema_commitment();
    schema[0] ^= 0xff;
    let wrong_schema =
        hostile_transport_descriptor(&descriptor, DescriptorTransportMutation::Schema(schema));
    assert!(matches!(
        runtime.readmit_branch_basis(&wrong_schema),
        Err(RelationalBranchBasisDenial::MixedAxis(
            RelationalBranchBasisMismatchAxis::SchemaRoot
        ))
    ));

    let mut visibility = descriptor.visibility_commitment();
    visibility[0] ^= 0xff;
    let wrong_visibility = hostile_transport_descriptor(
        &descriptor,
        DescriptorTransportMutation::Visibility(visibility),
    );
    assert!(matches!(
        runtime.readmit_branch_basis(&wrong_visibility),
        Err(RelationalBranchBasisDenial::MixedAxis(
            RelationalBranchBasisMismatchAxis::Visibility
        ))
    ));
}

#[test]
fn cross_branch_immutable_target_substitution_denies_distinctly() {
    let runtime = crate::tests::support::runtime_with_test_schema();
    crate::tests::support::create_entity_outcome(&runtime, "phase6-main-basis");
    runtime
        .history_authority()
        .fork_branch_from(BranchId("feature".to_owned()), &BranchId("main".to_owned()))
        .unwrap();
    crate::tests::support::create_entity_outcome_on_branch(
        &runtime,
        "phase6-feature-basis",
        BranchId("feature".to_owned()),
    );

    let main_identity = runtime.main_branch_identity();
    let (main_descriptor, main_basis) = runtime.observe_branch(&main_identity).unwrap();
    let feature_identity = runtime
        .branch_identity(&BranchId("feature".to_owned()))
        .unwrap();
    let (feature_descriptor, feature_basis) = runtime.observe_branch(&feature_identity).unwrap();
    drop((main_basis, feature_basis));

    let substituted_reference = relational_branch_observation(
        runtime.runtime_instance_id(),
        "main",
        feature_descriptor.reference().target().clone(),
        main_descriptor.reference().generation(),
    )
    .unwrap();
    let main_descriptor = hostile_transport_descriptor(
        &main_descriptor,
        DescriptorTransportMutation::Reference(substituted_reference),
    );

    assert!(matches!(
        runtime.readmit_branch_basis(&main_descriptor),
        Err(RelationalBranchBasisDenial::WrongImmutableTarget)
    ));
}
