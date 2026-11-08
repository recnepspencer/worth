//! Two families select absence and their own original versions through the same owners.
use super::publication_fixture::PublishedModel;
use super::*;
use worth_relational::facade::{branch::AdmittedRelationalBranchBasis, mvcc::*};

#[test]
fn graph_selection_removes_both_families_then_restores_their_original_ids_and_cold_facts() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (world, before) = PublishedModel::with_before_publication(execution);
    let mut second = world.meaning.clone();
    second.family = "second-family".into();
    let second_output = publication_fixture::publish(
        execution,
        &world.runtime,
        &world.owner,
        &world.installed,
        &second,
        "second-output",
    );
    let (_, published) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let absent = select(execution, &world, &before, 2, true);
    assert_eq!(
        absent
            .required_output_publications(&world.installed, execution)
            .count(),
        0
    );
    for output in [world.output, second_output] {
        assert!(begin(&world.runtime)
            .read_entity(execution, output)
            .unwrap()
            .base()
            .is_none());
    }
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    for meaning in [&world.meaning, &second] {
        assert!(world
            .runtime
            .readmit_required_output_route_absence(execution, &world.installed, &basis, meaning)
            .unwrap()
            .is_some());
    }
    release_test_commit_snapshot(&world.runtime, &absent);
    require_cold_absence(execution, &world, [&world.meaning, &second]);
    let restored = select(execution, &world, &published, 2, false);
    let publications = restored
        .required_output_publications(&world.installed, execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(publications.len(), 2);
    let outputs = publications
        .iter()
        .flat_map(|p| p.members().map(|(_, _, _, id)| id))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(outputs, [world.output, second_output].into_iter().collect());
    assert!(publications.iter().all(|p| p.is_historical_selection()));
    let (_, restored_basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    for meaning in [&world.meaning, &second] {
        assert!(world
            .runtime
            .readmit_required_output_route_absence(
                execution,
                &world.installed,
                &restored_basis,
                meaning
            )
            .unwrap()
            .is_none());
    }
    for output in outputs {
        assert_eq!(
            world
                .runtime
                .observe_model_owner(execution, &restored_basis, &ownership(), output)
                .unwrap(),
            Some(world.model)
        );
    }
    release_test_commit_snapshot(&world.runtime, &restored);
}

#[test]
fn selecting_before_a_source_existed_removes_that_sources_publication_and_presence_once() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let (world, before) = PublishedModel::with_before_publication(execution);
    let later = member(execution, &world.runtime, "later-source");
    create_relation(
        execution,
        &world.runtime,
        world.model,
        later,
        "owns-later-source",
    );
    let mut meaning = world.meaning.clone();
    meaning.source_scope = later;
    let output = publication_fixture::publish(
        execution,
        &world.runtime,
        &world.owner,
        &world.installed,
        &meaning,
        "later-output",
    );
    let selected = select(execution, &world, &before, 2, true);
    for retired in [later, output] {
        assert!(begin(&world.runtime)
            .read_entity(execution, retired)
            .unwrap()
            .base()
            .is_none());
    }
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    assert!(world
        .runtime
        .readmit_required_output_route_absence(execution, &world.installed, &basis, &meaning)
        .unwrap()
        .is_some());
    release_test_commit_snapshot(&world.runtime, &selected);
}

fn select(
    execution: &ExecutionRequest<'_, '_>,
    world: &PublishedModel,
    selected: &AdmittedRelationalBranchBasis,
    count: usize,
    absent: bool,
) -> CommitResult {
    let (_, current) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let graph = world
        .runtime
        .prepare_owned_model_graph_selection(
            execution,
            &current,
            selected,
            &ownership(),
            &ownership(),
            world.model,
        )
        .unwrap();
    let selections = world
        .runtime
        .prepare_historical_output_selections(execution, &world.installed, &graph)
        .unwrap();
    assert_eq!(selections.len(), count);
    assert!(selections.iter().all(|p| p.selects_absence() == absent));
    let admission = world
        .owner
        .admit_historical_output_selection(selections.iter().map(|s| s.publication().authority()))
        .unwrap();
    let port = world.runtime.preparation_port();
    let input = port
        .admit_owned_graph_selection(execution, begin(&world.runtime), graph)
        .unwrap();
    let candidate = port
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
        .compare_and_publish(execution, candidate)
    else {
        panic!("selection must publish atomically")
    };
    world
        .runtime
        .settle_performed_publication(performed)
        .unwrap()
}

fn require_cold_absence(
    execution: &ExecutionRequest<'_, '_>,
    world: &PublishedModel,
    meanings: [&RequiredOutputPublicationMeaning; 2],
) {
    let checkpoint = world
        .runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let mut cold = RelationalRuntimeApi::builder()
        .schema_registry(publication_fixture::registry())
        .build()
        .unwrap();
    let expected = cold
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_custom_invariants(
            RelationalSchemaRegistry::new(),
            vec![ownership().invariant_registration().unwrap()],
        )
        .unwrap();
    cold.durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .unwrap();
    let receipt = cold
        .readmit_recovered_initial_schema_installation(expected)
        .unwrap();
    let (_, installed) = receipt.issue_application_publication_owner();
    let installed = installed.with_source_scope_owned_producers([("generator".into(), KindId(3))]);
    let (_, basis) = cold.observe_branch(&cold.main_branch_identity()).unwrap();
    for meaning in meanings {
        assert!(cold
            .readmit_required_output_route_absence(execution, &installed, &basis, meaning)
            .unwrap()
            .is_some());
    }
    assert!(begin(&cold)
        .read_entity(execution, world.output)
        .unwrap()
        .base()
        .is_none());
}
