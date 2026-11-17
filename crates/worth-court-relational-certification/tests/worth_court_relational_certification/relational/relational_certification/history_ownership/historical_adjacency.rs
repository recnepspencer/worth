//! Authentic complete adjacency restoration, with structural ABA and outside edges.
use super::read_evidence_fixture::SelectedWorld;
use super::transfer_fixture::{basis, select};
use super::*;
use worth_relational::facade::runtime::{
    RelationalAdjacencyDirection as Direction, VisibilityProjectionView,
};

fn revision(
    execution: &ExecutionRequest<'_, '_>,
    view: &VisibilityProjectionView,
    anchor: EntityId,
    kind: KindId,
    direction: Direction,
) -> Option<worth_relational::facade::identity::VersionId> {
    view.read_exact_adjacency_structural_revision(execution, anchor, kind, direction)
        .unwrap()
}

#[test]
fn complete_adjacency_restoration_rejects_later_structural_edits_and_aba() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(execution);
    let expected = revision(
        execution,
        &world.view(&world.selected),
        world.model,
        KindId(2),
        Direction::Outgoing,
    );
    assert_ne!(
        expected,
        revision(
            execution,
            &world.current(),
            world.model,
            KindId(2),
            Direction::Outgoing
        )
    );
    let evidence = world.evidence(execution).unwrap();
    let matches = || {
        evidence
            .matches_adjacency_revision(
                execution,
                &world.current(),
                world.model,
                KindId(2),
                Direction::Outgoing,
                expected,
            )
            .unwrap()
    };
    assert!(matches());
    assert!(
        !evidence
            .matches_adjacency_revision(
                execution,
                &world.view(&world.edited),
                world.model,
                KindId(2),
                Direction::Outgoing,
                expected,
            )
            .unwrap(),
        "an older view cannot borrow the restoration baseline"
    );
    assert!(
        !evidence
            .matches_adjacency_revision(
                execution,
                &world.current(),
                world.outside,
                KindId(2),
                Direction::Outgoing,
                None,
            )
            .unwrap(),
        "an outside anchor was never selected"
    );
    create_relation_of_kind(
        execution,
        &world.runtime,
        world.model,
        world.outside,
        "unrelated-kind",
        KindId(6),
    );
    assert!(
        matches(),
        "an unrelated adjacency kind does not invalidate this proof"
    );
    let next = member(execution, &world.runtime, "later-member");
    let added = create_relation(execution, &world.runtime, world.model, next, "later-edge");
    assert!(
        !matches(),
        "a later relevant relation changes the structural revision"
    );
    let mut tx = begin(&world.runtime);
    tx.push_batch(
        WorkerIntentBatch::new("delete-later-edge").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent { relation_id: added }),
        )),
        execution.allocation_policy(),
    )
    .unwrap();
    let committed = tx.commit(execution, &world.runtime).unwrap();
    release_test_commit_snapshot(&world.runtime, &committed);
    assert!(
        !matches(),
        "equal membership after an ordinary edit is ABA, not restoration"
    );
}

#[test]
fn incoming_adjacency_requires_the_complete_restored_relation_roster() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let model = create_entity(execution, &runtime, "model");
    let owned = member(execution, &runtime, "member");
    let outside = create_entity(execution, &runtime, "outside");
    create_relation(execution, &runtime, model, owned, "owns");
    create_relation_of_kind(
        execution,
        &runtime,
        outside,
        owned,
        "external-incoming",
        KindId(6),
    );
    let selected = basis(&runtime);
    let selected_view = runtime
        .read_truth()
        .project_observation(&selected.observation())
        .unwrap();
    let ownership_revision = revision(
        execution,
        &selected_view,
        owned,
        KindId(2),
        Direction::Incoming,
    );
    let outside_revision = revision(
        execution,
        &selected_view,
        owned,
        KindId(6),
        Direction::Incoming,
    );
    let committed = select(execution, &runtime, model, &selected).unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    let restored = basis(&runtime);
    let current = runtime
        .read_truth()
        .project_observation(&restored.observation())
        .unwrap();
    let evidence = runtime
        .read_truth()
        .read_historical_selection(execution, &selected.observation(), &restored.observation())
        .unwrap();
    assert!(
        evidence
            .matches_adjacency_revision(
                execution,
                &current,
                owned,
                KindId(2),
                Direction::Incoming,
                ownership_revision,
            )
            .unwrap(),
        "installed ownership relations are genuinely restored"
    );
    assert_eq!(
        outside_revision,
        revision(execution, &current, owned, KindId(6), Direction::Incoming)
    );
    assert!(
        !evidence
            .matches_adjacency_revision(
                execution,
                &current,
                owned,
                KindId(6),
                Direction::Incoming,
                outside_revision,
            )
            .unwrap(),
        "equal outside membership is not selected restoration authority"
    );
}

#[test]
fn transferred_members_do_not_borrow_restoration_authority_for_ordinary_outgoing_edges() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = installed_runtime();
    let a = create_entity(execution, &runtime, "a");
    let b = create_entity(execution, &runtime, "b");
    let owned = member(execution, &runtime, "member");
    let outside = create_entity(execution, &runtime, "outside");
    let owner = create_relation(execution, &runtime, a, owned, "owner");
    create_relation_of_kind(
        execution,
        &runtime,
        owned,
        outside,
        "ordinary-outgoing",
        KindId(6),
    );
    let at_a = basis(&runtime);
    super::transfer_fixture::move_to(execution, &runtime, owner, b, owned);
    let selected = basis(&runtime);
    let selected_view = runtime
        .read_truth()
        .project_observation(&selected.observation())
        .unwrap();
    let expected = revision(
        execution,
        &selected_view,
        owned,
        KindId(6),
        Direction::Outgoing,
    );
    let committed = select(execution, &runtime, a, &at_a).unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    // Selecting A's history at the transferred step returns the member to B.
    // The original ordinary outgoing edge is deliberately left current.
    let committed = select(execution, &runtime, a, &selected).unwrap();
    release_test_commit_snapshot(&runtime, &committed);
    let restored = basis(&runtime);
    let current = runtime
        .read_truth()
        .project_observation(&restored.observation())
        .unwrap();
    super::transfer_fixture::assert_owner(execution, &runtime, owned, owner, b, &[a, b]);
    let evidence = runtime
        .read_truth()
        .read_historical_selection(execution, &selected.observation(), &restored.observation())
        .unwrap();
    assert!(
        !evidence
            .matches_adjacency_revision(
                execution,
                &current,
                owned,
                KindId(6),
                Direction::Outgoing,
                expected,
            )
            .unwrap(),
        "transfer preserves current external work; it does not restore its selected revision"
    );
}
