use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn structural_identity_comparison_only_uses_fingerprint_truth() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "alpha");

    let comparison = runtime
        .inspect_what_happened()
        .compare_structural_identity(
            execution,
            InspectionScope::Current,
            crate::facade::transactions::RecordRef::Entity(entity),
            crate::facade::transactions::RecordRef::Entity(entity),
        )
        .expect("read request remains live");

    assert_eq!(
        comparison.verdict,
        StructuralIdentityComparisonVerdict::IncomparableMissingFingerprint
    );
}

#[test]
fn structural_identity_evidence_exposes_declared_fingerprint_and_lineage_for_entities_only() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "alpha");
    assert!(runtime.set_entity_structural_identity_for_test(
        entity,
        Some(StructuralFingerprint::new(Symbol(11), 101)),
        Some(LineageId(77)),
    ));

    let entity_evidence = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Current,
            crate::facade::transactions::RecordRef::Entity(entity),
        )
        .expect("read request remains live")
        .expect("entity evidence");

    assert_eq!(
        entity_evidence.structural_fingerprint,
        Some(StructuralFingerprint::new(Symbol(11), 101))
    );
    assert_eq!(entity_evidence.lineage_id, Some(LineageId(77)));
    assert!(entity_evidence.degradations.is_empty());

    let relation = create_relation(execution, &runtime, entity, entity, "self");
    let relation_evidence = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Current,
            crate::facade::transactions::RecordRef::Relation(relation),
        )
        .expect("read request remains live")
        .expect("relation evidence");

    assert!(relation_evidence.structural_fingerprint.is_none());
    assert!(relation_evidence.lineage_id.is_none());
    assert!(relation_evidence
        .degradations
        .contains(&crate::facade::inspection::InspectionDegradation::MissingStructuralFingerprint));
    assert!(relation_evidence
        .degradations
        .contains(&crate::facade::inspection::InspectionDegradation::MissingLineageIdentity));
}

#[test]
fn structural_identity_comparison_distinguishes_equal_mismatch_and_family_mismatch() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    let left = create_entity(execution, &runtime, "left");
    let right = create_entity(execution, &runtime, "right");
    let other_family = create_entity(execution, &runtime, "other-family");

    assert!(runtime.set_entity_structural_identity_for_test(
        left,
        Some(StructuralFingerprint::new(Symbol(21), 500)),
        Some(LineageId(1)),
    ));
    assert!(runtime.set_entity_structural_identity_for_test(
        right,
        Some(StructuralFingerprint::new(Symbol(21), 500)),
        Some(LineageId(2)),
    ));
    assert!(runtime.set_entity_structural_identity_for_test(
        other_family,
        Some(StructuralFingerprint::new(Symbol(22), 500)),
        Some(LineageId(3)),
    ));

    let equal = runtime
        .inspect_what_happened()
        .compare_structural_identity(
            execution,
            InspectionScope::Current,
            crate::facade::transactions::RecordRef::Entity(left),
            crate::facade::transactions::RecordRef::Entity(right),
        )
        .expect("read request remains live");
    assert_eq!(
        equal.verdict,
        StructuralIdentityComparisonVerdict::EqualByFingerprint
    );

    assert!(runtime.set_entity_structural_identity_for_test(
        right,
        Some(StructuralFingerprint::new(Symbol(21), 999)),
        Some(LineageId(2)),
    ));
    let mismatch = runtime
        .inspect_what_happened()
        .compare_structural_identity(
            execution,
            InspectionScope::Current,
            crate::facade::transactions::RecordRef::Entity(left),
            crate::facade::transactions::RecordRef::Entity(right),
        )
        .expect("read request remains live");
    assert_eq!(
        mismatch.verdict,
        StructuralIdentityComparisonVerdict::NotEqualByFingerprint
    );

    let family_mismatch = runtime
        .inspect_what_happened()
        .compare_structural_identity(
            execution,
            InspectionScope::Current,
            crate::facade::transactions::RecordRef::Entity(left),
            crate::facade::transactions::RecordRef::Entity(other_family),
        )
        .expect("read request remains live");
    assert_eq!(
        family_mismatch.verdict,
        StructuralIdentityComparisonVerdict::IncomparableFingerprintFamilyMismatch
    );
}

#[test]
fn structural_identity_query_is_family_scoped_and_entity_only() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    let left = create_entity(execution, &runtime, "left");
    let right = create_entity(execution, &runtime, "right");
    let ignored = create_entity(execution, &runtime, "ignored");
    let _relation = create_relation(execution, &runtime, left, right, "rel");

    assert!(runtime.set_entity_structural_identity_for_test(
        left,
        Some(StructuralFingerprint::new(Symbol(31), 1)),
        Some(LineageId(10)),
    ));
    assert!(runtime.set_entity_structural_identity_for_test(
        right,
        Some(StructuralFingerprint::new(Symbol(31), 2)),
        Some(LineageId(11)),
    ));
    assert!(runtime.set_entity_structural_identity_for_test(
        ignored,
        Some(StructuralFingerprint::new(Symbol(32), 3)),
        Some(LineageId(12)),
    ));

    let queried = runtime
        .inspect_what_happened()
        .query_structural_identity(
            execution,
            &StructuralIdentityQueryRequest {
                scope: InspectionScope::Current,
                partition_scope: None,
                fingerprint_family: Symbol(31),
            },
        )
        .expect("read request remains live");

    assert_eq!(queried.len(), 2);
    assert!(queried.iter().all(|evidence| evidence.record_class
        == crate::facade::inspection::InspectionRecordClass::Entity));
    assert!(queried.iter().all(|evidence| {
        evidence
            .structural_fingerprint
            .is_some_and(|fingerprint| fingerprint.family == Symbol(31))
    }));
}

#[test]
fn structural_identity_historical_scope_does_not_leak_reused_slot_sidecars() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = runtime_with_test_schema();
    let original = create_entity_outcome(execution, &runtime, "original");
    let original_entity = changed_entities(&original)[0];
    assert!(runtime.set_entity_structural_identity_for_test(
        original_entity,
        Some(StructuralFingerprint::new(Symbol(41), 111)),
        Some(LineageId(41)),
    ));
    let replacement_entity = runtime
        .simulate_entity_slot_reuse_for_test(
            original_entity,
            Some(StructuralFingerprint::new(Symbol(42), 222)),
            Some(LineageId(42)),
        )
        .expect("replacement entity");
    assert_eq!(original_entity.local_slot, replacement_entity.local_slot);

    let historical = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Version(original.version_id),
            crate::facade::transactions::RecordRef::Entity(original_entity),
        )
        .expect("read request remains live")
        .expect("historical structural identity");
    let current = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Current,
            crate::facade::transactions::RecordRef::Entity(replacement_entity),
        )
        .expect("read request remains live")
        .expect("current replacement structural identity");

    assert!(historical.structural_fingerprint.is_none());
    assert!(historical.lineage_id.is_none());
    assert!(historical
        .degradations
        .contains(&crate::facade::inspection::InspectionDegradation::MissingStructuralFingerprint));
    assert!(historical
        .degradations
        .contains(&crate::facade::inspection::InspectionDegradation::MissingLineageIdentity));
    assert_eq!(
        current.structural_fingerprint,
        Some(StructuralFingerprint::new(Symbol(42), 222))
    );
    assert_eq!(current.lineage_id, Some(LineageId(42)));
}

#[test]
fn structural_identity_recovery_preserves_current_evidence_and_queries() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = durable_runtime_with_test_schema();
    let family = runtime
        .services
        .symbols
        .with_write(|symbols| symbols.intern("inspection.structural.family"));
    let left = create_entity(execution, &runtime, "left");
    let right = create_entity(execution, &runtime, "right");
    assert!(runtime.set_entity_structural_identity_for_test(
        left,
        Some(StructuralFingerprint::new(family, 1001)),
        Some(LineageId(501)),
    ));
    assert!(runtime.set_entity_structural_identity_for_test(
        right,
        Some(StructuralFingerprint::new(family, 1002)),
        Some(LineageId(502)),
    ));
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let expected_left = runtime
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Current,
            crate::facade::transactions::RecordRef::Entity(left),
        )
        .expect("read request remains live")
        .expect("expected left evidence");
    let expected_query = runtime
        .inspect_what_happened()
        .query_structural_identity(
            execution,
            &StructuralIdentityQueryRequest {
                scope: InspectionScope::Current,
                partition_scope: None,
                fingerprint_family: family,
            },
        )
        .expect("read request remains live");

    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    let actual_left = recovered
        .inspect_what_happened()
        .structural_identity(
            execution,
            InspectionScope::Current,
            crate::facade::transactions::RecordRef::Entity(left),
        )
        .expect("read request remains live")
        .expect("actual left evidence");
    let actual_query = recovered
        .inspect_what_happened()
        .query_structural_identity(
            execution,
            &StructuralIdentityQueryRequest {
                scope: InspectionScope::Current,
                partition_scope: None,
                fingerprint_family: family,
            },
        )
        .expect("read request remains live");

    assert_eq!(expected_left, actual_left);
    assert_eq!(expected_query, actual_query);
}

#[test]
fn inspection_truth_bundle_recovery_parity_holds_for_current_and_historical_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "bundle");
    let entity = changed_entities(&created)[0];
    let _relation = create_relation(execution, &runtime, entity, entity, "self");
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();

    let expected = capture_inspection_truth_bundle(
        execution,
        &runtime,
        &BranchId("main".to_string()),
        entity,
        created.version_id,
    );
    let plan = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = durable_runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let actual = capture_inspection_truth_bundle(
        execution,
        &recovered,
        &BranchId("main".to_string()),
        entity,
        created.version_id,
    );

    assert_eq!(expected, actual);
}
