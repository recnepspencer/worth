//! An older sibling's source is not antecedent evidence of the selected step.
use super::read_evidence_fixture::{edit, SelectedWorld};
use super::*;
use worth_relational::facade::{history::BranchId, runtime::PositionedRelationalSnapshot};
fn position(
    runtime: &RelationalRuntime,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
) -> PositionedRelationalSnapshot {
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let positioned = runtime.read_truth().positioned_snapshot(&snapshot).unwrap();
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
    positioned
}
#[test]
fn an_older_sibling_source_does_not_enter_historical_verification() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(request);
    let (_, source) = world
        .runtime
        .observe_fork_source(world.runtime.main_branch_identity().branch_id())
        .unwrap();
    let fork = world
        .runtime
        .fork_branch(request, BranchId("unrelated-source".into()), source)
        .unwrap();
    let (_, child) = world
        .runtime
        .observe_branch(fork.target_identity())
        .unwrap();
    let mut tx = world
        .runtime
        .begin_branch_transaction(
            &child,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    tx.push_batch(
        WorkerIntentBatch::new("sibling-source").push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId(3),
                kind_id: KindId(3),
                client_key: ClientKey::raw("sibling-member"),
                fields: AspectFieldPatch::default(),
            }),
        )),
        request.allocation_policy(),
    )
    .unwrap();
    let sibling = tx.commit(request, &world.runtime).unwrap();
    let sibling_source = world
        .runtime
        .read_truth()
        .positioned_snapshot(&sibling.snapshot)
        .unwrap();
    edit(request, &world.runtime, world.model, "other", "later-main");
    let selected = transfer_fixture::basis(&world.runtime);
    let selected_source = position(&world.runtime, &selected);
    assert!(
        sibling_source.version_id() < selected_source.version_id(),
        "an ordinal-only fence would admit the sibling"
    );
    let restored =
        transfer_fixture::select(request, &world.runtime, world.model, &selected).unwrap();
    let after = transfer_fixture::basis(&world.runtime);
    let evidence = world
        .runtime
        .read_truth()
        .read_historical_selection(request, &selected.observation(), &after.observation())
        .unwrap();
    assert!(evidence
        .includes_source(request, &world.runtime, &selected_source)
        .unwrap());
    assert!(evidence
        .includes_source(
            request,
            &world.runtime,
            &position(&world.runtime, &world.restored)
        )
        .unwrap());
    assert!(!evidence
        .includes_source(request, &world.runtime, &sibling_source)
        .unwrap());
    release_test_commit_snapshot(&world.runtime, &sibling);
    release_test_commit_snapshot(&world.runtime, &restored);
}
