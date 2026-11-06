//! HistoricalSelection runs under genuine worker-only original request custody.
use super::publication_fixture::PublishedModel;
use super::*;
use worth_relational::facade::mvcc::*;
#[test]
fn worker_only_selection_restores_two_publication_families_at_their_original_ids() {
    use worth_execution::{ExecutionResourceLease, LeaseRequest};
    use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
    let host = serial_request();
    let setup = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(setup);
    let mut second = world.meaning.clone();
    second.family = "worker-second-family".into();
    let other = publication_fixture::publish(
        setup,
        &world.runtime,
        &world.owner,
        &world.installed,
        &second,
        "worker-second-output",
    );
    let selected = transfer_fixture::basis(&world.runtime);
    create_entity(setup, &world.runtime, "outside-later-edit");
    let current = transfer_fixture::basis(&world.runtime);
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
    let execution = &ExecutionRequest::leased(&lease);
    assert!(!execution.has_accounting());
    let graph = world
        .runtime
        .prepare_owned_model_graph_selection(
            execution,
            &current,
            &selected,
            &ownership(),
            &ownership(),
            world.model,
        )
        .unwrap();
    let selections = world
        .runtime
        .prepare_historical_output_selections(execution, &world.installed, &graph)
        .unwrap();
    assert_eq!(selections.len(), 2);
    let admission = world
        .owner
        .admit_historical_output_selection(selections.iter().map(|s| s.publication().authority()))
        .unwrap();
    let port = world.runtime.preparation_port();
    let input = port
        .admit_owned_graph_selection(execution, begin(&world.runtime), graph)
        .unwrap();
    let prepared = port
        .prepare_owned_graph_selection(
            execution,
            input,
            Some((
                &world.installed,
                RequiredOutputPublication::HistoricalSelection {
                    admission,
                    selections,
                },
            )),
        )
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = world
        .runtime
        .publication_port()
        .compare_and_publish(execution, prepared)
    else {
        panic!("worker-only historical selection must publish");
    };
    let committed = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    let publications = committed
        .required_output_publications(&world.installed, execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(publications.len(), 2);
    assert!(publications.iter().all(|p| p.is_historical_selection()));
    let restored = publications
        .iter()
        .flat_map(|p| p.members().map(|(_, _, _, id)| id))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(restored, [world.output, other].into_iter().collect());
    let after = transfer_fixture::basis(&world.runtime);
    let evidence = world
        .runtime
        .read_truth()
        .read_historical_selection(execution, &selected.observation(), &after.observation())
        .unwrap();
    for meaning in [&world.meaning, &second] {
        assert_eq!(
            evidence
                .selected_output_route(
                    execution,
                    &meaning.family,
                    meaning.source_scope,
                    meaning.source_partition
                )
                .unwrap(),
            Some(meaning)
        );
    }
    let cancelled = worth_execution::CancellationSource::new();
    cancelled.cancel();
    let stopped = host.clone().with_cancellation(cancelled.token());
    assert!(matches!(
        evidence.selected_output_route(
            &ExecutionRequest::serial(&stopped),
            &second.family,
            second.source_scope,
            second.source_partition
        ),
        Err(worth_relational::facade::runtime::RelationalHistoricalSelectionReadDenial::Request(_))
    ));
    for publication in &publications {
        assert!(publication
            .matches_current_ownership_at(execution, &world.installed, &after)
            .unwrap());
    }
    for meaning in [&world.meaning, &second] {
        let selected = world
            .runtime
            .prepare_historical_output_selection(
                execution,
                &world.installed,
                &after,
                &after,
                meaning,
            )
            .unwrap();
        assert!(selected.publication().is_historical_selection());
        // The antecedent walk through a HistoricalSelection origin also polls
        // the original worker request; it never needs serial accounting.
        let prior = world
            .runtime
            .readmit_restored_producer_prior(execution, &world.installed, &after, meaning)
            .unwrap()
            .unwrap();
        assert!(prior.publication().is_historical_selection());
        assert_eq!(prior.producer_scope(), world.source);
    }
    assert!(!execution.has_accounting());
    release_test_commit_snapshot(&world.runtime, &committed);
}
