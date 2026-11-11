//! Real endpoint moves and Native selection for generic multi-document courts.
use super::*;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    identity::RelationId,
    mvcc::{RelationalOwnedGraphSelectionPreparationError, RelationalPublicationOutcome},
};
pub(super) fn basis(runtime: &RelationalRuntime) -> AdmittedRelationalBranchBasis {
    runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1
}
pub(super) fn move_to(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    edge: RelationId,
    owner: EntityId,
    entity: EntityId,
) {
    let mut tx = begin(runtime);
    tx.push_batch(
        WorkerIntentBatch::new("transfer-member").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: edge,
                kind_id: KindId(2),
                source: EntityReference::Existing(owner),
                target: EntityReference::Existing(entity),
            }),
        )),
        execution.allocation_policy(),
    )
    .unwrap();
    let committed = tx.commit(execution, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}
pub(super) fn select(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    model: EntityId,
    selected: &AdmittedRelationalBranchBasis,
) -> Result<CommitResult, RelationalOwnedGraphSelectionPreparationError> {
    let graph = runtime
        .prepare_owned_model_graph_selection(
            execution,
            &basis(runtime),
            selected,
            &ownership(),
            &ownership(),
            model,
        )
        .unwrap();
    let port = runtime.preparation_port();
    let input = port.admit_owned_graph_selection(execution, begin(runtime), graph)?;
    let candidate = port.prepare_owned_graph_selection(execution, input, None)?;
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("lawful transfer selection must publish");
    };
    Ok(runtime.settle_performed_publication(performed).unwrap())
}
pub(super) fn assert_owner(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
    edge: RelationId,
    owner: EntityId,
    documents: &[EntityId],
) {
    let current = basis(runtime);
    assert_eq!(
        runtime
            .observe_model_owner(execution, &current, &ownership(), entity)
            .unwrap(),
        Some(owner)
    );
    let mut read = begin(runtime);
    assert!(read
        .read_entity(execution, entity)
        .unwrap()
        .base()
        .is_some());
    let selected = read.read_relation(execution, edge).unwrap();
    let selected = selected.base().unwrap();
    assert_eq!(selected.source, owner);
    assert_eq!(selected.target, entity);
    assert_eq!(
        selected.lifecycle,
        worth_relational::facade::storage::RecordLifecycleState::Live
    );
    for &document in documents {
        let closure = runtime
            .prepare_owned_model_entity_boundary(
                execution,
                &current,
                &current,
                &ownership(),
                document,
            )
            .unwrap();
        assert_eq!(
            closure.current_entities().contains(&entity),
            document == owner
        );
    }
}
