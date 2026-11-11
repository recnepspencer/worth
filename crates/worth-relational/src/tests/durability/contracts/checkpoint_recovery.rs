use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_checkpoint_recovers_lineage_authority() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = durable_runtime_with_test_schema();
    let first = create_entity_outcome(execution, &runtime, "first");
    let second = create_entity_outcome(execution, &runtime, "second");
    let first_entity = changed_entities(&first)[0];
    let second_entity = changed_entities(&second)[0];
    let first_lineage = runtime
        .lineage_access()
        .for_record(execution, first_entity)
        .unwrap()
        .unwrap()
        .lineage_id;
    let second_lineage = runtime
        .lineage_access()
        .for_record(execution, second_entity)
        .unwrap()
        .unwrap()
        .lineage_id;
    let checkpoint = crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_commit_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .filter(|envelope| envelope.has_lineage_authority())
            .count()
    );
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_event_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .map(|envelope| envelope.lineage_digest_basis().lineage_event_count())
            .sum::<usize>()
    );
    assert_eq!(
        checkpoint
            .lineage
            .digest_basis()
            .published_lineage_decision_count,
        runtime
            .history()
            .commit_envelopes_snapshot()
            .iter()
            .map(|envelope| envelope.lineage_digest_basis().lineage_decision_count())
            .sum::<usize>()
    );
    assert_eq!(
        checkpoint.lineage.counters().node_count,
        checkpoint.lineage.nodes().len()
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
    let graph = recovered.lineage_access().graph(
        execution,
        crate::facade::lineage::LineageGraphRequest {
            branch_id: BranchId("main".to_string()),
            traversal_basis:
                crate::facade::lineage::LineageGraphTraversalBasis::FullBranchGraphMaterialization,
        },
    ).unwrap();

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.events.len(), 2);
    assert!(graph
        .events
        .iter()
        .all(|event| event.kind == LineageEventKind::Create));
    assert_eq!(
        recovered
            .lineage_access()
            .for_record(execution, first_entity)
            .unwrap()
            .unwrap()
            .lineage_id(),
        first_lineage
    );
    assert_eq!(
        recovered
            .lineage_access()
            .for_record(execution, second_entity)
            .unwrap()
            .unwrap()
            .lineage_id(),
        second_lineage
    );
}
