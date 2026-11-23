//! A twice-restored entity holds lineages on different cold node pages. A
//! cold-directory read pinned at the second deletion must start from the
//! epoch that observation holds, whichever node page was warmed first.
//!
//! A named cold restore warms every node page eagerly today; the read must
//! not rely on that, so it holds equally for a lazily paged restore.
use super::*;

const PINNED: &str = "pinned-at-second-delete";

fn history_start(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
) -> Option<worth_relational::facade::identity::LineageId> {
    let identity = runtime.branch_identity(&BranchId(PINNED.into())).unwrap();
    let pinned = runtime.observe_branch(&identity).unwrap().1;
    runtime
        .record_history_at_observation(execution, &pinned.observation(), entity)
        .unwrap()
        .map(|history| history.start)
}

/// Enough unnamed, unowned parts in one commit to move the lineage
/// allocator past one 256-node cold page.
fn advance_a_node_page(execution: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime) {
    let mut batch = WorkerIntentBatch::new("page-filler");
    for ordinal in 0..300 {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId(3),
            kind_id: PART,
            client_key: ClientKey::raw(format!("filler-{ordinal}")),
            fields: AspectFieldPatch::default(),
        })));
    }
    let mut transaction = begin(runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let committed = transaction.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}

fn live_lineage(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
) -> worth_relational::facade::identity::LineageId {
    match name_of(execution, runtime, &head(runtime), entity) {
        PersistentNameOfRecord::Live { lineage, .. } => lineage,
        other => panic!("the member is live and named: {other:?}"),
    }
}

#[test]
fn cold_reads_at_a_second_deletion_start_from_its_own_epoch_across_node_pages() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = named_runtime();
    let model = create_entity(execution, &runtime, "model");
    let member = owned_entity(execution, &runtime, &head(&runtime), "member", true);
    create_relation(execution, &runtime, model, member, "owns-member");
    let born = head(&runtime);
    let l1 = live_lineage(execution, &runtime, member);
    delete(execution, &runtime, &[member]);
    advance_a_node_page(execution, &runtime);
    select(execution, &runtime, model, &born).expect("the first deletion is undone");
    let l2 = live_lineage(execution, &runtime, member);
    assert!(
        l1.0 <= 256 && l2.0 > 256,
        "the epochs sit on different pages: {l1:?} {l2:?}"
    );
    delete(execution, &runtime, &[member]);
    let (_, source) = runtime
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    runtime
        .fork_branch(execution, BranchId(PINNED.into()), source)
        .unwrap();
    assert_eq!(history_start(execution, &runtime, member), Some(l2), "live");

    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let mut expanded = named_runtime();
    expanded
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .unwrap();
    assert_eq!(
        history_start(execution, &expanded, member),
        Some(l2),
        "expanded"
    );

    let reopen = super::super::fixture::cold::reopen;
    let (cold_first, _) = reopen(execution, &runtime, named_registry());
    assert_eq!(
        history_start(execution, &cold_first, member),
        Some(l2),
        "a cold read before any node page is warmed"
    );
    let (warmed, _) = reopen(execution, &runtime, named_registry());
    assert!(warmed
        .immutable_lineage_node(execution, l1)
        .unwrap()
        .is_some());
    assert_eq!(
        history_start(execution, &warmed, member),
        Some(l2),
        "the older epoch's page warmed first must not win"
    );
}
