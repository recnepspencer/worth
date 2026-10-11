use super::*;

#[test]
fn cross_branch_immutable_target_substitution_denies_distinctly() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = crate::tests::support::runtime_with_test_schema();
    crate::tests::support::create_entity_outcome(execution, &runtime, "phase6-main-basis");
    runtime
        .history_authority()
        .fork_branch_from(BranchId("feature".to_owned()), &BranchId("main".to_owned()))
        .unwrap();
    crate::tests::support::create_entity_outcome_on_branch(
        execution,
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
