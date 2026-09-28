use crate::facade::config::CascadeDeletePolicy;
use crate::facade::history::BranchId;
use crate::facade::identity::{EntityId, PartitionId};
use crate::facade::lineage::HistoricalLineageResolution;
use crate::facade::transactions::{
    CreateIntent, EntityMutationIntent, EntitySpec, MutationIntent, ReplaceEntityIntent,
    WorkerIntentBatch,
};
use crate::tests::support::{
    aspect_key, changed_entities, create_entity_outcome, field_key,
    runtime_with_declared_aspect_schema, single_string_aspect_field_patch,
    test_owner_begin_transaction_for_branch,
};

fn runtime_with_test_schema() -> crate::facade::runtime::RelationalRuntime {
    runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations)
}

fn replace_entity_on_branch(
    runtime: &crate::facade::runtime::RelationalRuntime,
    entity: EntityId,
    replacement: &str,
    branch: BranchId,
) {
    let mut transaction = test_owner_begin_transaction_for_branch(runtime, branch);
    transaction
        .push_batch(
            WorkerIntentBatch::new(replacement).push(MutationIntent::Entity(
                EntityMutationIntent::Replace(ReplaceEntityIntent {
                    entity_id: entity,
                    replacement: EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: crate::facade::identity::KindId(1),
                        client_key: crate::facade::symbols::ClientKey::raw(replacement),
                        fields: single_string_aspect_field_patch(
                            aspect_key("name"),
                            field_key("name"),
                            replacement,
                        ),
                    },
                }),
            )),
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(runtime)
        .expect("replacement should commit");
}

#[test]
fn retained_fork_lineage_ancestry_is_flat_across_owner_created_history() {
    assert_owner_history_scale(64);
}

#[test]
#[ignore = "scheduled 4,096-branch owner-history scale profile"]
fn retained_fork_lineage_ancestry_is_flat_at_scale_profile() {
    assert_owner_history_scale(4_096);
}

#[test]
#[ignore = "scheduled 65,536-commit same-branch lineage scale profile"]
fn selected_main_lineage_is_flat_at_65_536_unrelated_commits() {
    assert_same_branch_history_scale(65_536, &[65_536]);
}

struct RetainedScaleFixture {
    runtime: crate::facade::runtime::RelationalRuntime,
    original: EntityId,
    observation: crate::facade::mvcc::RelationalBranchObservation,
}

impl RetainedScaleFixture {
    fn resolution(&self) -> HistoricalLineageResolution {
        retained_resolution(&self.runtime, self.original, &self.observation)
    }
}

fn retained_scale_fixture(replacement_count: usize) -> RetainedScaleFixture {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "flat-lineage-source");
    let original = changed_entities(&created)[0];
    let mut current = original;
    for replacement in 1..=replacement_count {
        replace_entity_on_branch(
            &runtime,
            current,
            &format!("flat-lineage-successor-{replacement}"),
            BranchId("main".to_owned()),
        );
        let version = runtime
            .publication()
            .latest_bundle()
            .expect("replacement publication")
            .commit
            .version_id;
        current = runtime
            .read_truth()
            .project_historical_version(version)
            .all_authoritative_entity_records()
            .into_iter()
            .find(|record| record.lineage_id.is_some())
            .expect("visible replacement successor")
            .entity_id;
    }
    let retained_branch = BranchId("retained-feature".to_owned());
    runtime
        .history_authority()
        .fork_branch_from(retained_branch.clone(), &BranchId("main".to_owned()))
        .expect("retained feature fork");
    let retained_identity = runtime
        .branch_identity(&retained_branch)
        .expect("retained feature identity");
    let (_, retained_basis) = runtime
        .observe_branch(&retained_identity)
        .expect("retained feature observation");
    RetainedScaleFixture {
        runtime,
        original,
        observation: retained_basis.observation(),
    }
}

fn retained_resolution(
    runtime: &crate::facade::runtime::RelationalRuntime,
    entity_id: EntityId,
    observation: &crate::facade::mvcc::RelationalBranchObservation,
) -> HistoricalLineageResolution {
    runtime
        .lineage_access()
        .resolve_record_history_for_observation(entity_id, observation)
        .expect("retained fork history")
}

fn assert_owner_history_scale(unrelated_commit_count: usize) {
    let fixture = retained_scale_fixture(1);
    let baseline = fixture.resolution();
    let baseline_commit_count = fixture.runtime.history().immutable_commit_count();
    let baseline_branch_count = fixture.runtime.history.branch_count();
    for ordinal in 1..=unrelated_commit_count {
        let branch = BranchId(format!("owner-scale-{ordinal}"));
        fixture
            .runtime
            .history_authority()
            .fork_branch_from(branch.clone(), &BranchId("main".to_owned()))
            .expect("owner-created scale branch");
        create_entity_on_branch(&fixture.runtime, branch, ordinal);
    }

    let scaled = fixture.resolution();
    assert_eq!(scaled.resolved, baseline.resolved);
    assert_eq!(scaled.traversed_event_ids, baseline.traversed_event_ids);
    assert_eq!(scaled.metrics, baseline.metrics);
    assert_eq!(
        fixture.runtime.history().immutable_commit_count(),
        baseline_commit_count + unrelated_commit_count
    );
    assert_eq!(
        fixture.runtime.history.branch_count(),
        baseline_branch_count + unrelated_commit_count
    );
}

fn create_entity_on_branch(
    runtime: &crate::facade::runtime::RelationalRuntime,
    branch: BranchId,
    ordinal: usize,
) {
    let key = format!("owner-scale-entity-{ordinal}");
    let mut transaction = test_owner_begin_transaction_for_branch(runtime, branch);
    transaction
        .push_batch(
            WorkerIntentBatch::new(key.clone()).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: crate::facade::identity::KindId(1),
                    client_key: crate::facade::symbols::ClientKey::raw(&key),
                    fields: single_string_aspect_field_patch(
                        aspect_key("name"),
                        field_key("name"),
                        &key,
                    ),
                },
            ))),
        )
        .expect("test staging stays within configured resource budgets");
    transaction
        .commit(runtime)
        .expect("owner-created scale commit");
}

fn assert_same_branch_history_scale(unrelated_commit_count: usize, checkpoints: &[usize]) {
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&runtime, "same-branch-scale-source");
    let original = changed_entities(&created)[0];
    replace_entity_on_branch(
        &runtime,
        original,
        "same-branch-scale-successor",
        BranchId("main".to_owned()),
    );
    let mut expected_metrics = None;
    for ordinal in 1..=unrelated_commit_count {
        create_entity_on_branch(&runtime, BranchId("main".to_owned()), ordinal);
        if checkpoints.contains(&ordinal) {
            let identity = runtime.main_branch_identity();
            let (_, basis) = runtime
                .observe_branch(&identity)
                .expect("selected main scale observation");
            let resolution = retained_resolution(&runtime, original, &basis.observation());
            assert_eq!(resolution.metrics.reachable_commit_node_visits, 0);
            assert_eq!(resolution.metrics.reachable_commit_catalog_probes, 0);
            assert_eq!(resolution.metrics.reachable_commit_parent_edge_visits, 0);
            assert_eq!(resolution.metrics.event_visit_count, 2);
            match expected_metrics {
                Some(expected) => assert_eq!(resolution.metrics, expected),
                None => expected_metrics = Some(resolution.metrics),
            }
        }
    }
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("unknown-record scale observation");
    runtime.performance_access().reset_counters();
    let unknown = runtime
        .lineage_access()
        .resolve_record_history_for_observation(
            EntityId::new(PartitionId::new(77), 99_999, 1),
            &basis.observation(),
        );
    assert!(unknown.is_none());
    let counters = runtime.performance_access().counters();
    assert_eq!(counters.lineage_historical_resolution_requests, 1);
    assert_eq!(counters.lineage_historical_resolution_index_probes, 1);
    assert_eq!(
        counters.lineage_historical_resolution_reachable_commit_node_visits,
        0
    );
    assert_eq!(
        counters.lineage_historical_resolution_reachable_commit_parent_edge_visits,
        0
    );
    assert_eq!(
        counters.lineage_historical_resolution_reachable_commit_catalog_probes,
        0
    );
}
