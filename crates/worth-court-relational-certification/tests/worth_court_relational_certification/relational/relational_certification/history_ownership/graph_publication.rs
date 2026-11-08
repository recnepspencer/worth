//! Native selection publishes one atomic owned graph without changing outside edges.
use super::*;
use worth_relational::facade::mvcc::RelationalPublicationOutcome;
#[test]
fn graph_selection_restores_owned_sources_and_outgoing_references_with_original_ids() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let model = create_entity(execution, &runtime, "model");
    let catalog = create_entity_in_partition(execution, &runtime, "catalog", PartitionId(5));
    let other_catalog =
        create_entity_in_partition(execution, &runtime, "other-catalog", PartitionId(6));
    let outside = create_entity(execution, &runtime, "outside");
    let member = member(execution, &runtime, "member");
    let owns = create_relation(execution, &runtime, model, member, "owns");
    let outgoing =
        create_relation_of_kind(execution, &runtime, member, catalog, "outgoing", KindId(4));
    let incoming =
        create_relation_of_kind(execution, &runtime, outside, model, "incoming", KindId(4));
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    super::graph_boundary::rewire(execution, &runtime, outgoing, member, other_catalog);
    let newer = super::fixture::member(execution, &runtime, "current-only");
    let extra_owns = create_relation(execution, &runtime, model, newer, "owns-new");
    let (_, current) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let graph = runtime
        .prepare_owned_model_graph_selection(
            execution,
            &current,
            &selected,
            &ownership(),
            &ownership(),
            model,
        )
        .unwrap();
    let input = runtime
        .preparation_port()
        .admit_owned_graph_selection(execution, begin(&runtime), graph)
        .unwrap();
    let candidate = runtime
        .preparation_port()
        .prepare_owned_graph_selection(execution, input, None)
        .unwrap();
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        current.descriptor(),
        "preparation is atomic and invisible"
    );
    let RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("graph selection must publish");
    };
    let committed = runtime.settle_performed_publication(performed).unwrap();
    let mut read = begin(&runtime);
    assert_eq!(
        read.read_relation(execution, outgoing)
            .unwrap()
            .base()
            .unwrap()
            .target,
        catalog
    );
    assert_eq!(
        read.read_relation(execution, owns)
            .unwrap()
            .base()
            .unwrap()
            .target,
        member
    );
    assert_eq!(
        read.read_relation(execution, incoming)
            .unwrap()
            .base()
            .unwrap()
            .source,
        outside
    );
    assert_eq!(
        read.read_relation(execution, incoming)
            .unwrap()
            .base()
            .unwrap()
            .target,
        model
    );
    assert!(read.read_entity(execution, newer).unwrap().base().is_none());
    let retired = read.read_relation(execution, extra_owns).unwrap();
    assert_eq!(
        retired.base().unwrap().lifecycle,
        worth_relational::facade::storage::RecordLifecycleState::DeletedRetained
    );
    assert_eq!(
        retired.base().unwrap().retired_at_version,
        Some(committed.commit.version_id)
    );
    let (_, after) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let closure = runtime
        .prepare_owned_model_graph_selection(
            execution,
            &after,
            &after,
            &ownership(),
            &ownership(),
            model,
        )
        .unwrap();
    assert!(!closure.current_relations().contains_key(&extra_owns));
    let worth_foundational::FoundationalBranchTarget::Basis(parent) = current.reference().target()
    else {
        panic!("current head is committed");
    };
    assert_eq!(
        committed
            .commit
            .parents
            .iter()
            .map(|id| id.0)
            .collect::<Vec<_>>(),
        [parent.selected_commit_id()]
    );
    release_test_commit_snapshot(&runtime, &committed);
}
