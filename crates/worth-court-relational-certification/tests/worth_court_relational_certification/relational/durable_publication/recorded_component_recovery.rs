use super::*;
use worth_relational::facade::durability::RelationalRecordedComponentAdmissionCause as Cause;

#[test]
fn recorded_component_recovery_admits_original_empty_and_historical_tail_subjects() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let empty = current(execution, &runtime);
    assert!(matches!(
        empty.target(),
        RecordedRelationalComponentTarget::Empty
    ));
    create_entity(execution, &runtime, "original-product-predecessor");
    let predecessor = current(execution, &runtime);
    create_entity(execution, &runtime, "unpublished-native-successor");
    let successor = current(execution, &runtime);
    let mut recovered = durable_runtime(&medium);
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let (_, authority) = recovered
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    for recorded in [&empty, &predecessor, &successor] {
        let basis = authority
            .admit_recorded_component(execution, &recovered, recorded)
            .unwrap()
            .into_basis();
        assert_recorded_basis(&recovered, &basis, recorded);
        assert_ne!(
            basis.descriptor().runtime_instance_id(),
            recorded.descriptor().runtime_instance_id()
        );
        let lease = recovered.retain_component_basis(&basis).unwrap();
        let retained = recovered
            .readmit_retained_branch_basis(basis.descriptor(), &lease)
            .unwrap();
        assert_eq!(retained, basis);
        let snapshot = recovered
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        recovered.snapshots().release_snapshot(&snapshot).unwrap();
        recovered.release_component_basis(lease).unwrap();
    }
    let predecessor_basis = authority
        .admit_recorded_component(execution, &recovered, &predecessor)
        .unwrap()
        .into_basis();
    assert!(
        recovered
            .owner_component_services()
            .basis_port()
            .compare_current_exact(&predecessor_basis)
            .is_err(),
        "historical admission does not substitute the latest root"
    );
    let foreign_medium = TestDurableMedium::new();
    let foreign = durable_runtime(&foreign_medium);
    assert!(matches!(
        authority
            .admit_recorded_component(execution, &foreign, &successor)
            .unwrap_err()
            .cause(),
        Cause::ForeignRecoveryRuntime
    ));
    let mut changed = serde_json::to_value(&successor).unwrap();
    let original_byte = changed["target"]["Canonical"]["artifact_digest"][0]
        .as_u64()
        .unwrap();
    changed["target"]["Canonical"]["artifact_digest"][0] = serde_json::json!(original_byte ^ 1);
    let changed: RecordedRelationalComponentReference = serde_json::from_value(changed).unwrap();
    assert_ne!(changed, successor);
    assert!(matches!(
        authority
            .admit_recorded_component(execution, &recovered, &changed)
            .unwrap_err()
            .cause(),
        Cause::RecordedTargetMismatch
    ));
}

#[test]
fn recorded_component_recovery_never_qualifies_memory_as_a_canonical_durable_source() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let source = memory_runtime();
    create_entity(execution, &source, "memory-canonical");
    let recorded = current(execution, &source);
    let plan = source
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut recovered = memory_runtime();
    let (_, authority) = recovered
        .durability_recovery()
        .recover_with_authority(execution, plan)
        .unwrap();
    assert!(matches!(
        authority
            .admit_recorded_component(execution, &recovered, &recorded)
            .unwrap_err()
            .cause(),
        Cause::MemoryOnly
    ));
}
