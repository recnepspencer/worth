use super::*;

#[test]
fn runtime_admits_registered_source_declaration() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = registered_source(
        "source:analysis-snapshot",
        BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ],
    );

    let contract = runtime
        .admit_source(declaration)
        .expect("registered source declaration should be admitted");

    assert_eq!(
        contract.declaration().declaration_identity().as_str(),
        "source:analysis-snapshot"
    );
}

#[test]
fn runtime_validates_registered_source_declaration() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = registered_source(
        "source:analysis-history",
        BridgeTruthViewSelector::historical_commit(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        ),
        vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::HistoricalRead,
            BridgeSourceCapability::BranchRead,
            BridgeSourceCapability::ReplayContinuityRead,
        ],
    );

    let validated = runtime
        .validate_source_declaration(declaration.clone())
        .expect("registered source declaration should validate");
    let admitted = runtime
        .admit_source(declaration.clone())
        .expect("registered source declaration should admit");

    assert_eq!(validated.declaration(), &declaration);
    assert_eq!(
        validated.contract_identity(),
        admitted.contract_identity().as_str()
    );
    assert_eq!(
        validated.canonical_basis(),
        format!(
            "validated-source-declaration|contract={}|declaration={}",
            admitted.digest(),
            declaration.digest(),
        )
    );
}

#[test]
fn runtime_rejects_unregistered_source_declaration() {
    let runtime = runtime(BridgeRuntimePolicy::default());
    let error = runtime
        .admit_source(registered_source(
            "source:missing",
            BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            ),
            vec![BridgeSourceCapability::SnapshotRead],
        ))
        .expect_err("unregistered source declaration should fail");

    assert_eq!(
        error.kind(),
        crate::error::BridgeDeliveryErrorKind::SourceContractMismatch
    );
    let retained = runtime
        .diagnostics()
        .source_failure_for_declaration_identity("source:missing")
        .expect("unregistered source rejection should retain canonical source failure");
    assert_eq!(
        retained.failure_class(),
        crate::source::SourceFailureClass::SourceContractMismatch
    );
    assert_eq!(
        retained.delivery_error_kind(),
        crate::error::BridgeDeliveryErrorKind::SourceContractMismatch
    );
    assert_eq!(
        runtime
            .diagnostics()
            .explain_source_failure_record(&retained)
            .failure_identity(),
        retained.failure_identity().as_str()
    );
}

#[test]
fn runtime_materializes_registered_source_packet() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let contract = runtime
        .admit_source(registered_source(
            "source:analysis-history",
            BridgeTruthViewSelector::historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ],
        ))
        .expect("registered historical source should be admitted");

    let observation = runtime
        .materialize_source_packet(&contract, SnapshotReadPacket::new(vec![]), execution)
        .expect("registered historical source should materialize");

    assert!(
        crate::truth_identity_fixtures::truth_snapshot_fixture_matches(
            observation.snapshot_identity(),
            "snapshot-a"
        )
    );
    assert!(observation
        .authority_basis()
        .commit_identity()
        .is_some_and(
            |identity| crate::truth_identity_fixtures::truth_commit_fixture_matches(
                identity, "commit-a"
            )
        ));
}

#[test]
fn runtime_records_source_materialization_rejection_when_adapter_cannot_open_snapshot() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime =
        runtime_with_source_adapter(BridgeRuntimePolicy::default(), RejectingSourceAdapter);
    let contract = runtime
        .admit_source(registered_source(
            "source:analysis-history",
            BridgeTruthViewSelector::historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ],
        ))
        .expect("registered historical source should be admitted");

    let error = match runtime.materialize_source_packet(
        &contract,
        SnapshotReadPacket::new(vec![]),
        execution,
    ) {
        Ok(_) => panic!("rejecting source adapter should fail materialization"),
        Err(error) => error,
    };

    assert!(
        matches!(error.kind(), crate::error::BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(ref cause) if cause.kind() == crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure)
    );
    let failure = runtime
        .diagnostics()
        .last_source_failure_record()
        .expect("source failure should be retained");
    assert_eq!(
        failure.failure_class(),
        crate::source::SourceFailureClass::SourceMaterializationRejected
    );
    assert!(
        matches!(failure.delivery_error_kind(), crate::error::BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(ref cause) if cause.kind() == crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure)
    );
}

#[test]
fn runtime_records_adapter_capability_drift_when_adapter_binds_wrong_snapshot() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime = runtime_with_source_adapter(BridgeRuntimePolicy::default(), DriftSourceAdapter);
    let contract = runtime
        .admit_source(registered_source(
            "source:analysis-history",
            BridgeTruthViewSelector::historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ],
        ))
        .expect("registered historical source should be admitted");

    let error = match runtime.materialize_source_packet(
        &contract,
        SnapshotReadPacket::new(vec![]),
        execution,
    ) {
        Ok(_) => panic!("drift source adapter should fail materialization"),
        Err(error) => error,
    };

    assert_eq!(
        error.kind(),
        crate::error::BridgeDeliveryErrorKind::SnapshotIdentityMismatch
    );
    let failure = runtime
        .diagnostics()
        .last_source_failure_record()
        .expect("source failure should be retained");
    assert_eq!(
        failure.failure_class(),
        crate::source::SourceFailureClass::AdapterCapabilityDrift
    );
    assert_eq!(
        failure.delivery_error_kind(),
        crate::error::BridgeDeliveryErrorKind::SnapshotIdentityMismatch
    );
}

#[test]
fn runtime_rejects_source_packet_set_reordering_from_adapter() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let runtime =
        runtime_with_source_adapter(BridgeRuntimePolicy::default(), ReorderingSourceAdapter);
    let contract = runtime
        .admit_source(registered_source(
            "source:analysis-history",
            BridgeTruthViewSelector::historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ],
        ))
        .expect("registered historical source should be admitted");

    let error = match runtime.materialize_source_packet_batch(
        &contract,
        vec![
            SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                "entity-1",
                crate::snapshot::SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid snapshot aspect key"),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
            )]),
            SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                "entity-2",
                crate::snapshot::SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid snapshot aspect key"),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
            )]),
        ],
        execution,
    ) {
        Ok(_) => panic!("reordered source packet set should be rejected"),
        Err(error) => error,
    };

    assert_eq!(
        error.kind(),
        crate::error::BridgeDeliveryErrorKind::SourceContractMismatch
    );
    let failure = runtime
        .diagnostics()
        .last_source_failure_record()
        .expect("source failure should be retained");
    assert_eq!(
        failure.failure_class(),
        crate::source::SourceFailureClass::AdapterCapabilityDrift
    );
    assert_eq!(
        failure.delivery_error_kind(),
        crate::error::BridgeDeliveryErrorKind::SourceContractMismatch
    );
}
