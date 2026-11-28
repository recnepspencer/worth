use worth_harness::facade::{ExecutionProfile, HarnessAdapter, ScenarioPlan};

use super::certification_bundle::SourceHarnessCertificationBundle;
use super::{execute_source_request, SourceHarnessExecution, SourceHarnessTarget};
use crate::error::BridgeDeliveryErrorKind;
use crate::facade::{BridgeSourceCapability, BridgeSourceCapabilitySet, BridgeTruthViewSelector};
use crate::harness::adapter::{BridgeHarnessAdapter, BridgeHarnessSession};
use crate::harness::fixtures::{BridgeHarnessFixture, NamedTruth};
use crate::source::{SourceDeclaration, SourceDeclarationIdentity};

fn source_fixture(name: &str) -> worth_harness::facade::ScenarioFixture<BridgeHarnessFixture> {
    let (scenario, commit) = NamedTruth::scenario_on_branch("analysis", "alice");
    ScenarioPlan::new(
        name,
        BridgeHarnessFixture::new(vec![NamedTruth::mapping()])
            .with_policy(crate::facade::BridgeRuntimePolicy::development())
            .with_source_declaration(historical_source_declaration(
                SourceDeclarationIdentity::admit_bridge_owned("source:analysis-history"),
                commit,
            ))
            .with_source_adapter_capabilities(BridgeSourceCapabilitySet::new(vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ]))
            .with_relational_truth(scenario),
    )
    .declare_input("source")
    .declare_observation("source")
    .compile()
}

fn historical_source_declaration(
    declaration_identity: SourceDeclarationIdentity,
    commit: crate::facade::TruthCommitIdentity,
) -> SourceDeclaration {
    SourceDeclaration::new(
        declaration_identity,
        BridgeTruthViewSelector::historical_commit(
            crate::facade::TruthBranchIdentity::from_relational_branch_id("analysis"),
            commit,
        ),
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::HistoricalRead,
            BridgeSourceCapability::BranchRead,
            BridgeSourceCapability::ReplayContinuityRead,
        ]),
    )
}

fn execute_typed_source(
    profile: ExecutionProfile,
    target: SourceHarnessTarget,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> SourceHarnessExecution {
    let adapter = BridgeHarnessAdapter;
    let fixture = source_fixture("typed-source-certification");
    let mut session = adapter.create_runtime().expect("source harness runtime");
    adapter
        .prepare_runtime(&mut session, &profile)
        .expect("source harness prepare");
    adapter
        .load_fixture(&mut session, &fixture)
        .expect("source harness load fixture");
    let runtime_bridge = loaded_runtime_bridge(&session);
    execute_source_request(runtime_bridge, &fixture.fixture, target, resource_request)
        .expect("typed source execution should succeed")
}

fn loaded_runtime_bridge(session: &BridgeHarnessSession) -> &crate::facade::RuntimeBridge {
    session
        .runtime
        .as_ref()
        .expect("source harness runtime should be loaded")
}

#[test]
fn materialize_and_replay_retain_typed_source_certification_truth() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let materialized = execute_typed_source(
        ExecutionProfile::development("typed-materialize"),
        SourceHarnessTarget::Materialize {
            declaration_identity: SourceDeclarationIdentity::admit_bridge_owned(
                "source:analysis-history",
            ),
        },
        resource_request,
    );
    let replayed = execute_typed_source(
        ExecutionProfile::development("typed-replay"),
        SourceHarnessTarget::Replay {
            declaration_identity: SourceDeclarationIdentity::admit_bridge_owned(
                "source:analysis-history",
            ),
        },
        resource_request,
    );

    let SourceHarnessExecution::Materialize {
        contract,
        record: materialized_record,
    } = materialized
    else {
        panic!("expected materialized source execution");
    };
    let SourceHarnessExecution::Replay {
        contract: replay_contract,
        record: replay_source_record,
        replayed: replayed_record,
    } = replayed
    else {
        panic!("expected replay source execution");
    };
    let materialized_bundle =
        SourceHarnessCertificationBundle::materialized(&contract, &materialized_record, None);
    let replay_bundle = SourceHarnessCertificationBundle::materialized(
        &replay_contract,
        &replay_source_record,
        Some(replayed_record.digest()),
    );

    assert_eq!(contract.digest(), replay_contract.digest());
    assert_eq!(
        materialized_record.truth_view_digest(),
        replay_source_record.truth_view_digest()
    );
    assert_eq!(
        materialized_record.record_identity(),
        replayed_record.record_identity()
    );
    assert_eq!(
        materialized_bundle.truth_view_digest(),
        replay_bundle.truth_view_digest()
    );
    assert_eq!(
        materialized_bundle.source_contract_digest(),
        replay_bundle.source_contract_digest()
    );
    assert!(materialized_bundle.failure_digest().is_none());
    assert!(replay_bundle.replay_digest().is_some());
    assert_eq!(
        materialized_bundle
            .counter_snapshot()
            .source_materialization_count(),
        1
    );
    assert_eq!(
        replay_bundle
            .counter_snapshot()
            .source_replay_request_count(),
        1
    );
}

#[test]
fn batch_materialization_retains_typed_packet_set_counter_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let execution = execute_typed_source(
        ExecutionProfile::development("typed-batch"),
        SourceHarnessTarget::MaterializeBatch {
            declaration_identity: SourceDeclarationIdentity::admit_bridge_owned(
                "source:analysis-history",
            ),
        },
        resource_request,
    );

    let SourceHarnessExecution::Materialize { contract, record } = execution else {
        panic!("expected batch source materialization");
    };
    let bundle = SourceHarnessCertificationBundle::materialized(&contract, &record, None);

    assert_eq!(record.planned_packet_digests().len(), 2);
    assert_eq!(record.read_packets().len(), 2);
    assert_eq!(
        record.truth_view_digest(),
        record.materialized_packet_set_digest()
    );
    assert_eq!(bundle.counter_snapshot().source_packet_count(), 2);
    assert_eq!(bundle.counter_snapshot().source_packet_member_count(), 2);
}

#[test]
fn unregistered_source_rejection_retains_typed_failure_evidence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let execution = execute_typed_source(
        ExecutionProfile::development("typed-unregistered"),
        SourceHarnessTarget::RejectUnregistered {
            declaration_identity: SourceDeclarationIdentity::admit_bridge_owned(
                "source:hostile-missing",
            ),
        },
        resource_request,
    );

    let SourceHarnessExecution::Rejected { failure } = execution else {
        panic!("expected rejected source execution");
    };
    let bundle = SourceHarnessCertificationBundle::rejected(&failure);

    assert_eq!(
        failure.delivery_error_kind(),
        BridgeDeliveryErrorKind::SourceContractMismatch
    );
    assert_eq!(
        failure.declaration_identity().as_str(),
        "source:hostile-missing"
    );
    assert!(bundle.truth_view_digest().is_none());
    assert!(bundle.failure_digest().is_some());
    assert_eq!(
        bundle.counter_snapshot().source_contract_mismatch_count(),
        1
    );
    assert_eq!(bundle.counter_snapshot().source_materialization_count(), 0);
}

#[test]
fn adapter_snapshot_failures_retain_typed_failure_kind_and_zero_success_residue() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let open_failure = execute_typed_source(
        ExecutionProfile::development("typed-open-rejection")
            .with_metadata("source_adapter_behavior", "reject_open_snapshot"),
        SourceHarnessTarget::RejectOpenSnapshot {
            declaration_identity: SourceDeclarationIdentity::admit_bridge_owned(
                "source:analysis-history",
            ),
        },
        resource_request,
    );
    let drift_failure = execute_typed_source(
        ExecutionProfile::development("typed-drift-rejection")
            .with_metadata("source_adapter_behavior", "drift_snapshot_identity"),
        SourceHarnessTarget::RejectSnapshotDrift {
            declaration_identity: SourceDeclarationIdentity::admit_bridge_owned(
                "source:analysis-history",
            ),
        },
        resource_request,
    );

    assert_typed_source_failure(
        open_failure,
        |kind| matches!(kind, BridgeDeliveryErrorKind::SnapshotAcquisitionFailure(cause) if cause.kind() == crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure),
    );
    assert_typed_source_failure(drift_failure, |kind| {
        matches!(kind, BridgeDeliveryErrorKind::SnapshotIdentityMismatch)
    });
}

fn assert_typed_source_failure(
    execution: SourceHarnessExecution,
    expected_kind: fn(&BridgeDeliveryErrorKind) -> bool,
) {
    let SourceHarnessExecution::Rejected { failure } = execution else {
        panic!("expected rejected source execution");
    };
    let bundle = SourceHarnessCertificationBundle::rejected(&failure);

    assert!(expected_kind(&failure.delivery_error_kind()));
    assert!(bundle.truth_view_digest().is_none());
    assert!(bundle.source_contract_digest().is_none());
    assert_eq!(bundle.counter_snapshot().source_materialization_count(), 0);
    assert_eq!(
        bundle
            .counter_snapshot()
            .retained_source_failure_record_count(),
        1
    );
}
