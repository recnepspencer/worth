//! Progressed-source membership uses exact current facts and selected anchors, not all of B.
use super::publication_fixture::PublishedModel;
use super::transfer_fixture::{basis, move_to};
use super::*;
use worth_relational::facade::runtime::RelationalHistoricalSourcePresence as Presence;

fn exercise(worker: bool) {
    use worth_execution::{ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
    let host = serial_request();
    let setup = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(setup);
    let mut related = world.meaning.clone();
    related.family = "progressed-related".into();
    let (old_related, old_edge) = publication_fixture::publish_related(
        setup,
        &world.runtime,
        &world.owner,
        &world.installed,
        &related,
        "progressed-original",
    );
    let b = create_entity(setup, &world.runtime, "progressed-outside-owner");
    let other = member(setup, &world.runtime, "progressed-outside-member");
    create_relation(setup, &world.runtime, b, other, "outside-owns-other");
    let current = basis(&world.runtime);
    let graph = world
        .runtime
        .prepare_owned_model_graph_selection(
            setup,
            &current,
            &current,
            &ownership(),
            &ownership(),
            world.model,
        )
        .unwrap();
    let edge = graph
        .current_relations()
        .values()
        .find(|edge| edge.kind_id == KindId(2) && edge.target == world.source)
        .unwrap()
        .relation_id;
    drop(graph);
    move_to(setup, &world.runtime, edge, b, world.source);
    let selected = basis(&world.runtime);
    move_to(setup, &world.runtime, edge, world.model, world.source);
    let committed = super::transfer_publications::select_outputs(setup, &world, &selected);
    release_test_commit_snapshot(&world.runtime, &committed);
    let restored = basis(&world.runtime);
    let evidence = world
        .runtime
        .read_truth()
        .read_historical_selection(setup, &selected.observation(), &restored.observation())
        .unwrap();
    // E is restored under B. B's unrelated member never enters A's selected boundary.
    assert_eq!(
        world
            .runtime
            .observe_model_owner(setup, &restored, &ownership(), world.source)
            .unwrap(),
        Some(b)
    );
    let mut fresh = world.meaning.clone();
    fresh.family = "progressed-new-publication".into();
    let new_source = publication_fixture::publish(
        setup,
        &world.runtime,
        &world.owner,
        &world.installed,
        &fresh,
        "progressed-new-source",
    );
    let progressed = basis(&world.runtime);
    let lease = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(1).unwrap(),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .unwrap();
    let execution = if worker {
        ExecutionRequest::leased(&lease)
    } else {
        ExecutionRequest::serial(&host)
    };
    assert_eq!(execution.has_accounting(), !worker);
    assert_eq!(
        evidence
            .source_presence(&execution, new_source, KindId(3))
            .unwrap(),
        Presence::NotLive
    );
    assert_eq!(
        evidence
            .source_presence_at(
                &execution,
                &world.runtime,
                &progressed,
                &ownership(),
                new_source,
                KindId(3)
            )
            .unwrap(),
        Presence::LiveInDomain
    );
    assert_eq!(
        world
            .runtime
            .observe_model_owner(&execution, &progressed, &ownership(), new_source)
            .unwrap(),
        Some(b)
    );
    assert_eq!(
        evidence
            .source_presence_at(
                &execution,
                &world.runtime,
                &progressed,
                &ownership(),
                other,
                KindId(3)
            )
            .unwrap(),
        Presence::OutsideDomain
    );
    assert_eq!(
        evidence
            .source_presence_at(
                &execution,
                &world.runtime,
                &progressed,
                &ownership(),
                new_source,
                KindId(1)
            )
            .unwrap(),
        Presence::KindMismatch { actual: KindId(3) }
    );
    let (replacement, _) =
        publication_fixture::replace_related(setup, &world, &related, old_related, old_edge);
    let after_retirement = basis(&world.runtime);
    assert_eq!(
        evidence
            .source_presence(&execution, old_related, KindId(3))
            .unwrap(),
        Presence::LiveInDomain
    );
    assert_eq!(
        evidence
            .source_presence_at(
                &execution,
                &world.runtime,
                &after_retirement,
                &ownership(),
                old_related,
                KindId(3)
            )
            .unwrap(),
        Presence::NotLive
    );
    assert_eq!(
        evidence
            .source_presence_at(
                &execution,
                &world.runtime,
                &after_retirement,
                &ownership(),
                replacement,
                KindId(3)
            )
            .unwrap(),
        Presence::LiveInDomain
    );
    assert_eq!(
        evidence
            .source_presence_at(
                &execution,
                &world.runtime,
                &progressed,
                &ownership(),
                old_related,
                KindId(3)
            )
            .unwrap(),
        Presence::LiveInDomain,
        "the exact selected truth does not fall through to today's retired state"
    );
    assert_eq!(
        basis(&world.runtime).descriptor(),
        after_retirement.descriptor()
    );
    assert!(matches!(evidence.source_presence_at(&execution, &world.runtime, &current,
        &ownership(), world.source, KindId(3)),
        Err(worth_relational::facade::runtime::RelationalHistoricalSourceReadDenial::Selection(
            worth_relational::facade::runtime::RelationalHistoricalSelectionReadDenial::SelectedBasisMismatch { .. }))));
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        ExecutionPosture::Automatic,
        DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::new(1).unwrap(),
            None,
            Some(1),
        ),
    );
    let bounded = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    assert!(
        matches!(evidence.source_presence_at(&ExecutionRequest::serial(&bounded), &world.runtime,
        &progressed, &ownership(), new_source, KindId(3)),
        Err(worth_relational::facade::runtime::RelationalHistoricalSourceReadDenial::Ownership(cause))
            if matches!(*cause, worth_relational::facade::branch::RelationalModelOwnershipDenial::Request(_)))
    );
}
#[test]
fn progressed_source_proves_exact_liveness_and_only_its_transferred_selected_domain() {
    exercise(false);
}
#[test]
fn worker_only_progressed_source_proves_exact_liveness_and_only_its_transferred_selected_domain() {
    exercise(true);
}
