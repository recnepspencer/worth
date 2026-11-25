//! Public Native branch custody and readmission through real owner entries.
use worth_relational::facade::{
    branch::{
        relational_branch_observation, RelationalBranchBasisDenial,
        RelationalBranchBasisDescriptor, RelationalBranchBasisMismatchAxis,
        RelationalBranchBasisPosture, RelationalBranchReferenceObservation,
        RelationalBranchVersion,
    },
    history::BranchId,
    mvcc::RelationalTransactionIntent,
    runtime::{RelationalRuntime, RelationalRuntimeApi},
};
use worth_testkit_relational::{
    durable_publication::{batch_create, create_entity, create_entity_outcome, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[derive(serde::Serialize)]
struct DescriptorTransportPayload {
    descriptor_version: u16,
    runtime_instance_id: u64,
    branch_id: BranchId,
    reference: RelationalBranchReferenceObservation,
    truth_version: RelationalBranchVersion,
    root_identity: u64,
    schema_commitment: worth_foundational::CanonicalDigestId,
    visibility_commitment: worth_foundational::CanonicalDigestId,
    posture: RelationalBranchBasisPosture,
}

enum DescriptorTransportMutation {
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
        schema_commitment: worth_foundational::CanonicalDigestId::new(
            descriptor.schema_commitment(),
        ),
        visibility_commitment: worth_foundational::CanonicalDigestId::new(
            descriptor.visibility_commitment(),
        ),
        posture: descriptor.posture(),
    };
    match mutation {
        DescriptorTransportMutation::Branch(value) => payload.branch_id = value,
        DescriptorTransportMutation::Reference(value) => payload.reference = value,
        DescriptorTransportMutation::TruthVersion(value) => payload.truth_version = value,
        DescriptorTransportMutation::RootIdentity(value) => payload.root_identity = value,
        DescriptorTransportMutation::Schema(value) => {
            payload.schema_commitment = worth_foundational::CanonicalDigestId::new(value)
        }
        DescriptorTransportMutation::Visibility(value) => {
            payload.visibility_commitment = worth_foundational::CanonicalDigestId::new(value)
        }
    }
    let encoded = rmp_serde::to_vec_named(&payload).expect("hostile descriptor payload encodes");
    rmp_serde::from_slice(&encoded).expect("transport accepts descriptive hostile payload")
}

fn runtime() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .build()
        .expect("valid native runtime configuration")
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
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    create_entity_outcome(execution, &runtime, "retained-basis-before");
    let identity = runtime.main_branch_identity();
    let (descriptor, retained) = runtime.observe_branch(&identity).unwrap();
    create_entity_outcome(execution, &runtime, "retained-basis-after");
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
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    create_entity_outcome(execution, &runtime, "snapshot-before-movement");
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let observation = basis.observation();
    let before = runtime
        .snapshots()
        .snapshot_for_observation(&observation)
        .unwrap();
    create_entity_outcome(execution, &runtime, "snapshot-after-movement");
    let after = runtime
        .snapshots()
        .snapshot_for_observation(&observation)
        .unwrap();

    assert_eq!(before.branch_id(), after.branch_id());
    assert_eq!(before.version_id(), after.version_id());
    assert!(observation.selected_root_identity() > 0);
}

#[test]
fn external_pin_retains_once_and_release_consumes_the_obligation() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    create_entity(execution, &runtime, "external-pin-before");
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    let lease = runtime.retain_component_basis(&basis).unwrap();
    drop(basis);
    create_entity(execution, &runtime, "external-pin-after");

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
fn superseded_unretained_descriptor_denies_without_reconstructing_authority() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    create_entity(execution, &runtime, "stale-before");
    let identity = runtime.main_branch_identity();
    let (descriptor, basis) = runtime.observe_branch(&identity).unwrap();
    drop(basis);
    create_entity(execution, &runtime, "stale-after");

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
fn malformed_truth_and_root_identity_substitutions_deny_distinctly() {
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    create_entity_outcome(execution, &runtime, "phase6-axis-basis");
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
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    create_entity_outcome(execution, &runtime, "phase6-commitment-basis");
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
    let host_request = serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = memory_runtime();
    create_entity_outcome(execution, &runtime, "phase6-main-basis");
    let port = runtime.fork_port();
    let (_, source) = port.observe_fork_source(&BranchId("main".into())).unwrap();
    let forked = port
        .fork_branch(execution, BranchId("feature".into()), source)
        .unwrap();
    let (_, basis) = runtime.observe_branch(forked.target_identity()).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            batch_create("phase6-feature-basis"),
            execution.allocation_policy(),
        )
        .unwrap();
    transaction.commit(execution, &runtime).unwrap();
    drop(basis);

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
