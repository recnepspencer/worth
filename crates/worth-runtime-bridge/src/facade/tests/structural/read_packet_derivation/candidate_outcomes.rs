use super::*;

#[test]
fn runtime_derives_structural_candidates_from_read_packets() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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

    let planned = runtime
        .plan_structural_match_packet_set_from_read_packets(
            &contract,
            SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                "entity-1",
                crate::snapshot::SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid snapshot aspect key"),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
            )]),
            vec![SnapshotReadPacket::new(vec![
                crate::snapshot::SnapshotReadRequest::for_coarse(
                    "entity-1",
                    crate::snapshot::SnapshotReadContract::scalar(
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid snapshot aspect key"),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                ),
            ])],
            execution,
        )
        .expect("structural candidates should derive from read packets");
    let reduced = runtime
        .reduce_structural_match_set(&planned)
        .expect("derived structural packet set should reduce");

    assert_eq!(planned.candidate_count(), 1);
    assert_eq!(
        planned.candidates()[0].candidate_kind(),
        StructuralMatchCandidateKind::ExactAdvisoryMatch
    );
    assert_eq!(
        reduced.outcome_class(),
        StructuralMatchOutcomeClass::ExactAdvisoryMatch
    );
}
#[test]
fn runtime_derives_identity_authority_conflict_from_same_snapshot_same_structure() {
    let serial_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);

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

    let planned = runtime
        .plan_structural_match_packet_set_from_read_packets(
            &contract,
            SnapshotReadPacket::new(vec![crate::snapshot::SnapshotReadRequest::for_coarse(
                "entity-1",
                crate::snapshot::SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid snapshot aspect key"),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
            )]),
            vec![SnapshotReadPacket::new(vec![
                crate::snapshot::SnapshotReadRequest::for_coarse(
                    "entity-2",
                    crate::snapshot::SnapshotReadContract::scalar(
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid snapshot aspect key"),
                        worth_foundational::facade::ScalarAspectType::String,
                    ),
                ),
            ])],
            execution,
        )
        .expect("structural candidates should derive from read packets");
    let reduced = runtime
        .reduce_structural_match_set(&planned)
        .expect("derived structural packet set should reduce");

    assert_eq!(planned.candidate_count(), 1);
    assert_eq!(
        planned.candidates()[0].candidate_kind(),
        StructuralMatchCandidateKind::IdentityAuthorityConflict
    );
    assert_eq!(
        reduced.outcome_class(),
        StructuralMatchOutcomeClass::RejectedIdentityAuthorityConflict
    );
}
