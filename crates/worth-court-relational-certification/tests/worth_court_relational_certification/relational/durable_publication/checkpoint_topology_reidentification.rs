//! Original topology reidentification and lineage histories survive a real source.
use super::checkpoint_truth_equivalence::{assert_recovered_commit_truth, TruthSubjects};
use super::*;
use worth_relational::facade::{
    history::{AspectHistoryEntry, AspectResolutionContext, BranchId},
    identity::LineageId,
    lineage::{HistoricalResolutionBoundednessBasis, HistoricalResolutionDigestMode},
};
#[test]
fn topology_identity_survival_preserves_reidentification_truth_across_recovery() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let medium = TestDurableMedium::new();
    let runtime = durable_runtime(&medium);
    let created = create_entity_outcome(execution, &runtime, "topology-source");
    let entity =
        worth_testkit_relational::aspect_storage::publications::changed_entities(&created)[0];
    let start_lineage = runtime
        .lineage_node_at_observation(
            execution,
            &runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .1
                .observation(),
            entity,
        )
        .unwrap()
        .unwrap()
        .node()
        .lineage_id();

    let replacement = super::checkpoint_index_generations::update_entity_on_branch(
        execution,
        &runtime,
        entity,
        "topology-source-updated",
        BranchId("main".into()),
    );
    let replaced_entity =
        worth_testkit_relational::aspect_storage::publications::changed_entities(&replacement)[0];
    let replacement_lineage = runtime
        .lineage_node_at_observation(
            execution,
            &runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .1
                .observation(),
            replaced_entity,
        )
        .unwrap()
        .unwrap()
        .node()
        .lineage_id();
    let resolution = runtime
        .discover_authored_lineage_history(
            execution,
            &runtime.main_branch_identity(),
            start_lineage,
        )
        .unwrap();

    assert_eq!(
        resolution.digest_basis().digest_mode(),
        HistoricalResolutionDigestMode::ExactDigestCanonicalOrder
    );
    assert_eq!(
        resolution.digest_basis().boundedness_basis(),
        HistoricalResolutionBoundednessBasis::BranchScopedLineageSeed
    );
    assert!(!resolution.resolved.is_empty());
    assert_eq!(resolution.resolved[0], replacement_lineage);
    assert_eq!(resolution.metrics.resolved_lineage_count, 1);

    let history = runtime
        .history()
        .lineage_entity_aspect_history(execution, &BranchId("main".into()), start_lineage, None)
        .unwrap();
    let history = history.expect("lineage aspect history");
    assert_lineage_history_origin_invariants(&history.entries, start_lineage);

    medium.file_checkpoint(capture(execution, &runtime));
    let (recovered, _) = recover(execution, &medium);

    let recovered_resolution = recovered
        .discover_authored_lineage_history(
            execution,
            &recovered.main_branch_identity(),
            start_lineage,
        )
        .unwrap();
    assert_eq!(
        recovered_resolution.digest_basis(),
        resolution.digest_basis()
    );
    assert_eq!(recovered_resolution.resolved, resolution.resolved);
    assert_eq!(
        recovered_resolution.traversed_event_ids,
        resolution.traversed_event_ids
    );

    assert_recovered_commit_truth(
        execution,
        &runtime,
        &recovered,
        replacement.commit.commit_id,
        TruthSubjects {
            entities: &[replaced_entity],
            relations: &[],
            lineages: &[start_lineage, replacement_lineage],
        },
    );
}

fn assert_lineage_history_origin_invariants(
    entries: &[AspectHistoryEntry],
    start_lineage_id: LineageId,
) {
    assert!(
        !entries.is_empty(),
        "expected lineage-aware aspect history entries for {:?}",
        start_lineage_id
    );
    assert!(entries.iter().all(|entry| matches!(
        entry.resolution,
        AspectResolutionContext::ResolvedViaLineage {
            start_lineage_id: resolved_start,
            ..
        } if resolved_start == start_lineage_id
    )));
}
