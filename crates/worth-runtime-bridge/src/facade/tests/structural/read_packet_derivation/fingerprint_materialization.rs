use super::*;
use crate::facade::BridgeDeliveryErrorKind;

#[test]
fn runtime_materializes_structural_fingerprint_from_truth_view_read() {
    let policy = crate::policy::BridgeExecutionPolicyBaseline::operational().request_policy();
    let lease = crate::snapshot::test_execution_lease_for_policy(
        policy,
        worth_execution::CancellationToken::new(),
    );
    let execution = worth_execution::ExecutionRequest::leased(&lease);

    let runtime = runtime(BridgeRuntimePolicy::default());
    let declaration = registered_structural(
        "structural:analysis-snapshot",
        StructuralFingerprintFamily::TopologyFingerprint,
        StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        )),
    );
    let contract = runtime
        .admit_structural_comparison(declaration)
        .expect("registered structural declaration should be admitted");

    let fingerprint = execution
        .run(
            worth_execution::ExecutionWorkCeiling::new(8_000_000),
            |_| {
                runtime.materialize_structural_fingerprint(
                    &contract,
                    SnapshotReadPacket::new(vec![
                        crate::snapshot::SnapshotReadRequest::for_coarse(
                            "entity-1",
                            crate::snapshot::SnapshotReadContract::scalar(
                                worth_foundational::facade::AspectKey::new("profile")
                                    .expect("valid snapshot aspect key"),
                                worth_foundational::facade::ScalarAspectType::String,
                            ),
                        ),
                    ]),
                    execution,
                )
            },
        )
        .expect("active leased scope")
        .0
        .expect("structural fingerprint should materialize");

    assert_eq!(
        fingerprint.family(),
        StructuralFingerprintFamily::TopologyFingerprint
    );
    assert_eq!(
        fingerprint.snapshot_identity(),
        &crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
    );
    assert_eq!(
        fingerprint.snapshot_identity_text(),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a").as_str()
    );
    assert!(fingerprint
        .digest()
        .starts_with("structural-fingerprint:sha256:"));
    assert_eq!(fingerprint.record_value_evidence().records().len(), 1);
    assert_eq!(fingerprint.equivalence_member_evidence().members().len(), 1);
    assert!(fingerprint.record_value_evidence().records()[0]
        .canonical_basis()
        .starts_with("structural-record-value-evidence|"));
    assert!(fingerprint.equivalence_member_evidence().members()[0]
        .canonical_basis()
        .starts_with("structural-equivalence-member|"));
    assert!(fingerprint
        .canonical_basis()
        .contains("record-values=structural-record-value-evidence-set|"));
    assert!(fingerprint
        .canonical_basis()
        .contains("equivalence-members=structural-equivalence-member-set|"));
}

struct RefusingSnapshotReader {
    identity: TruthSnapshotIdentity,
    resource: bool,
}

impl crate::snapshot::TruthSnapshotReader for RefusingSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.identity.clone()
    }

    fn read_packet(
        &self,
        _: &SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::snapshot::SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError>
    {
        if !self.resource {
            return Err(crate::snapshot::BridgeSnapshotReadError::new(
                "domain read refusal",
            ));
        }
        // This source's retained read cannot fit even the entire caller envelope.
        execution
            .in_scope(|lease| {
                let denial = worth_execution::ExecutionMemoryReservation::reserve_in_scope(
                    lease,
                    execution.memory_limit() + 1,
                )
                .unwrap_err();
                Err(crate::snapshot::BridgeSnapshotReadError::execution_denied(
                    denial.into(),
                ))
            })
            .map_err(crate::snapshot::BridgeSnapshotReadError::execution_scope_denied)?
    }
}

struct RefusingSnapshotSource(bool);

impl crate::adapter::SnapshotReadSource for RefusingSnapshotSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Box<dyn crate::snapshot::TruthSnapshotReader>,
        crate::adapter::RelationalBridgeSourceError,
    > {
        Ok(Box::new(RefusingSnapshotReader {
            identity: identity.clone(),
            resource: self.0,
        }))
    }
}

#[test]
fn structural_read_adapter_preserves_resource_and_domain_refusals() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

    for resource in [true, false] {
        let declaration = registered_structural(
            "structural:analysis-snapshot",
            StructuralFingerprintFamily::TopologyFingerprint,
            StructuralTruthViewBasis::explicit_snapshot(BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            )),
        );
        let runtime = crate::builder::RuntimeBridgeBuilder::new()
            .with_policy(BridgeRuntimePolicy::default())
            .with_committed_patch_source(StaticSource)
            .with_snapshot_read_source(RefusingSnapshotSource(resource))
            .with_source_adapter(StaticSourceAdapter)
            .with_truth_branch_head_source(StaticSource)
            .with_signal_sink(StaticSink)
            .register_structural(declaration.clone())
            .register_mapping(native_profile_mapping_registration())
            .build()
            .unwrap();
        let contract = runtime.admit_structural_comparison(declaration).unwrap();
        let error = runtime
            .materialize_structural_fingerprint(
                &contract,
                SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                    "entity-1",
                    crate::snapshot::SnapshotReadContract::scalar(
                        worth_foundational::facade::AspectKey::new("profile").unwrap(),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                )]),
                execution,
            )
            .unwrap_err();
        if resource {
            assert!(
                matches!(
                    error.kind(),
                    BridgeDeliveryErrorKind::ExecutionDenied(
                        crate::error::BridgeExecutionDenial::MemoryExhausted(_),
                    )
                ),
                "the public structural adapter must retain its resource cause: {error:?}"
            );
        } else {
            assert!(
                matches!(error.kind(), BridgeDeliveryErrorKind::SnapshotReadContractViolation(cause) if cause.kind() == crate::snapshot::BridgeSnapshotReadErrorKind::ExternalSnapshotReadFailure)
            );
            assert!(error.to_string().contains("domain read refusal"));
        }
    }
}
