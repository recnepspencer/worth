use crate::facade::diagnostics::{DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope};
use crate::facade::history::{AspectHistoryLineageEventSpan, BranchId, HistoryAspectQueryTarget};
use crate::facade::identity::{KindId, PartitionId};
use crate::facade::lineage::{
    HistoricalResolutionBoundednessBasis, HistoricalResolutionDigestMode,
    HistoricalResolutionRequest, LineageGraphRequest, LineageGraphTraversalBasis,
};
use crate::facade::transactions::{
    EntityMutationIntent, MutationIntent, ReplaceEntityIntent, WorkerIntentBatch,
};
use crate::tests::support::*;

// CONTRACT: historical_lineage_resolution
// LANES: success, adversarial, recovery

#[test]
fn historical_lineage_resolution_follows_replace_events() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "source");
    let entity = changed_entities(&created)[0];
    let start_lineage = runtime
        .lineage_access()
        .for_record(execution, entity)
        .unwrap()
        .unwrap()
        .lineage_id;

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("replace").push(MutationIntent::Entity(
            EntityMutationIntent::Replace(ReplaceEntityIntent {
                entity_id: entity,
                replacement: crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("replacement"),
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "replacement",
                    ),
                },
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let outcome = txn.commit(execution, &runtime).unwrap();
    let resolution = runtime
        .lineage_access()
        .resolve_historical_lineage(
            execution,
            HistoricalResolutionRequest {
                branch_id: BranchId("main".to_string()),
                lineage_id: start_lineage,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            },
        )
        .unwrap();

    assert_eq!(resolution.start, start_lineage);
    assert_eq!(
        resolution.boundedness_basis,
        HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed
    );
    assert_eq!(resolution.traversed_event_ids.len(), 1);
    assert_eq!(resolution.resolved.len(), 1);
    assert_ne!(resolution.resolved[0], start_lineage);
    assert_eq!(resolution.metrics.traversed_event_count, 1);
    assert!(resolution.metrics.event_visit_count >= 1);
    assert_eq!(resolution.metrics.resolved_lineage_count, 1);
    assert_eq!(
        resolution.digest_basis().digest_mode(),
        HistoricalResolutionDigestMode::ExactDigestCanonicalOrder
    );
    assert_eq!(resolution.digest_basis().requested_start(), start_lineage);
    assert_eq!(
        resolution.digest_basis().canonical_traversed_event_ids(),
        resolution.traversed_event_ids
    );
    assert_eq!(resolution.trace.digest_basis(), resolution.digest_basis());
    assert_eq!(
        runtime
            .lineage_access()
            .graph(
                execution,
                LineageGraphRequest {
                    branch_id: BranchId("main".to_string()),
                    traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization,
                }
            )
            .unwrap()
            .events
            .iter()
            .filter(|event| event.commit.commit_id == outcome.commit.commit_id)
            .count(),
        2
    );
}

#[test]
fn historical_lineage_resolution_does_not_scan_unrelated_branch_events() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(execution, &runtime, "source");
    let entity = changed_entities(&created)[0];
    let start_lineage = runtime
        .lineage_access()
        .for_record(execution, entity)
        .unwrap()
        .unwrap()
        .lineage_id;

    for index in 0..6 {
        let label = format!("unrelated-{index}");
        let _ = create_entity_outcome(execution, &runtime, &label);
    }

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("replace").push(MutationIntent::Entity(
            EntityMutationIntent::Replace(ReplaceEntityIntent {
                entity_id: entity,
                replacement: crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("replacement"),
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "replacement",
                    ),
                },
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let _ = txn.commit(execution, &runtime).unwrap();

    let total_branch_events = runtime
        .lineage_access()
        .graph(
            execution,
            LineageGraphRequest {
                branch_id: BranchId("main".to_string()),
                traversal_basis: LineageGraphTraversalBasis::FullBranchGraphMaterialization,
            },
        )
        .unwrap()
        .metrics
        .event_count;

    let resolution = runtime
        .lineage_access()
        .resolve_historical_lineage(
            execution,
            HistoricalResolutionRequest {
                branch_id: BranchId("main".to_string()),
                lineage_id: start_lineage,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            },
        )
        .unwrap();

    assert_eq!(resolution.metrics.traversed_event_count, 1);
    assert_eq!(resolution.metrics.event_visit_count, 1);
    assert!(total_branch_events > resolution.metrics.event_visit_count);
}

#[test]
fn lineage_aspect_history_keeps_origin_events_and_marks_resolution_context() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(execution, &runtime, "source");
    let entity = changed_entities(&created)[0];
    let start_lineage = runtime
        .lineage_access()
        .for_record(execution, entity)
        .unwrap()
        .unwrap()
        .lineage_id;

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("replace").push(MutationIntent::Entity(
            EntityMutationIntent::Replace(ReplaceEntityIntent {
                entity_id: entity,
                replacement: crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("replacement"),
                    fields: crate::tests::support::single_string_aspect_field_patch(
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        "replacement",
                    ),
                },
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    let replacement = txn.commit(execution, &runtime).unwrap();
    let history = runtime
        .lineage_access()
        .entity_aspect_history(
            execution,
            HistoricalResolutionRequest {
                branch_id: BranchId("main".to_string()),
                lineage_id: start_lineage,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            },
            None,
        )
        .unwrap()
        .unwrap();
    let traced = runtime
        .lineage_access()
        .entity_aspect_history_with_trace(
            execution,
            HistoricalResolutionRequest {
                branch_id: BranchId("main".to_string()),
                lineage_id: start_lineage,
                boundedness_basis: HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed,
            },
            None,
        )
        .unwrap();
    let artifact = traced.trace.diagnostic_artifact();
    let digest = traced.lineage_aspect_resolution_digest();

    assert_eq!(history.start_lineage_id, start_lineage);
    assert_eq!(
        traced.trace.requested_target,
        HistoryAspectQueryTarget::Lineage(start_lineage)
    );
    assert_eq!(traced.trace.returned_entries, 3);
    assert_eq!(traced.trace.traversed_lineage_events, 1);
    assert_eq!(
        traced.trace.searched_lineage_event_span,
        Some(AspectHistoryLineageEventSpan {
            first_event_id: history.traversed_event_ids[0],
            last_event_id: history.traversed_event_ids[0],
        })
    );
    assert_eq!(artifact.scope, DiagnosticsScope::Lineage);
    assert_eq!(artifact.kind, DiagnosticsArtifactKind::DetailedTrace);
    assert_eq!(
        artifact.entries[0].code,
        DiagnosticCode::LineageAspectHistoryResolved
    );
    assert_eq!(
        digest.requested_target,
        HistoryAspectQueryTarget::Lineage(start_lineage)
    );
    assert_eq!(digest.entry_count, 3);
    assert_eq!(digest.traversed_lineage_events, 1);
    assert_eq!(
        digest.resolved_lineage_chain_len,
        history.resolved_lineage_chain.len() as u64
    );
    assert_eq!(traced.history.as_ref(), Some(&history));
    assert_eq!(history.traversed_event_ids.len(), 1);
    assert_eq!(history.entries.len(), 3);
    assert_lineage_history_origin_invariants(&history.entries, start_lineage);
    assert_eq!(
        history.entries[0].origin.commit_id,
        created.commit.commit_id
    );
    assert_eq!(
        history.entries[1].origin.commit_id,
        replacement.commit.commit_id
    );
    assert_eq!(
        history.entries[2].origin.commit_id,
        replacement.commit.commit_id
    );
}
