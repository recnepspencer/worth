//! Companion records follow an owned-graph selection (tree-based undo's
//! reversal facts, public #56 ruling (u)): relation-free records outside the
//! owned closure that the selection retires and rematerializes in their
//! original full-generation slots, with typed refusals for every misuse.
use super::*;
use std::collections::BTreeSet;
use worth_relational::facade::branch::{
    RelationalOwnedGraphCompanions, RelationalOwnedModelSelectionDenial as GraphDenial,
};
use worth_relational::facade::mvcc::RelationalPublicationOutcome;

fn companions(retire: &[EntityId], restore: &[EntityId]) -> RelationalOwnedGraphCompanions {
    RelationalOwnedGraphCompanions {
        retire: retire.iter().copied().collect::<BTreeSet<_>>(),
        restore: restore.iter().copied().collect::<BTreeSet<_>>(),
    }
}

fn head(runtime: &RelationalRuntime) -> worth_relational::facade::branch::AdmittedRelationalBranchBasis {
    runtime.observe_branch(&runtime.main_branch_identity()).unwrap().1
}

fn publish(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    graph: worth_relational::facade::branch::RelationalOwnedModelGraphSelection,
) {
    let input = runtime.preparation_port().admit_owned_graph_selection(execution, begin(runtime), graph).unwrap();
    let candidate = runtime.preparation_port().prepare_owned_graph_selection(execution, input, None).unwrap();
    let RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(execution, candidate)
    else {
        panic!("graph selection must publish");
    };
    runtime.settle_performed_publication(performed).unwrap();
}

/// A companion the current basis does not hold live refuses before any
/// effect: the selection would retire what is not there.
#[test]
fn a_companion_absent_from_the_current_basis_refuses() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let model = create_entity(execution, &runtime, "model");
    let fact = create_entity(execution, &runtime, "fact");
    let selected = head(&runtime);
    let mut transaction = begin(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("remove-fact").push(MutationIntent::Entity(EntityMutationIntent::Delete(
                DeleteEntityIntent { entity_id: fact },
            ))),
            execution.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &result);
    let current = head(&runtime);
    let refused = runtime.prepare_owned_model_graph_selection_with_companions(
        execution,
        &current,
        &selected,
        &ownership(),
        &ownership(),
        model,
        companions(&[fact], &[]),
    );
    assert!(matches!(refused, Err(GraphDenial::CompanionUnavailable { entity }) if entity == fact), "{refused:?}");
}

/// The model itself is in its owned closure, so it can never be a companion.
#[test]
fn a_companion_inside_the_owned_closure_refuses() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let model = create_entity(execution, &runtime, "model");
    let basis = head(&runtime);
    let refused = runtime.prepare_owned_model_graph_selection_with_companions(
        execution,
        &basis,
        &basis,
        &ownership(),
        &ownership(),
        model,
        companions(&[model], &[]),
    );
    assert!(matches!(refused, Err(GraphDenial::CompanionOverlap { entity }) if entity == model), "{refused:?}");
}

/// A companion with a relation refuses: retiring it would strand the edge.
#[test]
fn a_related_companion_refuses() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let model = create_entity(execution, &runtime, "model");
    let before = head(&runtime);
    let fact = create_entity(execution, &runtime, "fact");
    let catalog = create_entity(execution, &runtime, "catalog");
    create_relation_of_kind(execution, &runtime, fact, catalog, "reference", KindId(4));
    let current = head(&runtime);
    let refused = runtime.prepare_owned_model_graph_selection_with_companions(
        execution,
        &current,
        &before,
        &ownership(),
        &ownership(),
        model,
        companions(&[fact], &[]),
    );
    assert!(matches!(refused, Err(GraphDenial::CompanionRelated { entity }) if entity == fact), "{refused:?}");
}

/// A selection back to a basis without the companion retires it; the
/// selection forward again rematerializes it with its original ID.
#[test]
fn a_companion_retires_and_comes_back_with_its_original_id() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let model = create_entity(execution, &runtime, "model");
    let before = head(&runtime);
    let fact = create_entity(execution, &runtime, "fact");
    let with_fact = head(&runtime);
    let back = runtime
        .prepare_owned_model_graph_selection_with_companions(
            execution,
            &with_fact,
            &before,
            &ownership(),
            &ownership(),
            model,
            companions(&[fact], &[]),
        )
        .unwrap();
    publish(execution, &runtime, back);
    assert!(begin(&runtime).read_entity(execution, fact).unwrap().base().is_none(), "the companion retired");
    let without_fact = head(&runtime);
    let forward = runtime
        .prepare_owned_model_graph_selection_with_companions(
            execution,
            &without_fact,
            &with_fact,
            &ownership(),
            &ownership(),
            model,
            companions(&[], &[fact]),
        )
        .unwrap();
    publish(execution, &runtime, forward);
    let restored = begin(&runtime).read_entity(execution, fact).unwrap();
    assert_eq!(
        restored.base().map(|base| base.lifecycle),
        Some(worth_relational::facade::storage::RecordLifecycleState::Live),
        "the companion is back in its original slot"
    );
}
