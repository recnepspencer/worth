//! Even a genuine producer publication cannot enter another document's selection.
use super::publication_fixture::PublishedModel;
use super::*;
use worth_relational::facade::{mvcc::*, publication::AuthoredRetirementRefusal};
#[test]
fn selecting_a_publication_outside_the_selected_documents_ownership_refuses_its_actual_publication()
{
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(execution);
    let foreign_model = create_entity(execution, &world.runtime, "foreign-model");
    let foreign_source = member(execution, &world.runtime, "foreign-source");
    create_relation(
        execution,
        &world.runtime,
        foreign_model,
        foreign_source,
        "foreign-ownership",
    );
    let mut foreign_meaning = world.meaning.clone();
    foreign_meaning.source_scope = foreign_source;
    let foreign_output = publication_fixture::publish(
        execution,
        &world.runtime,
        &world.owner,
        &world.installed,
        &foreign_meaning,
        "foreign-output",
    );
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let selected = world
        .runtime
        .prepare_historical_output_selection(
            execution,
            &world.installed,
            &basis,
            &basis,
            &world.meaning,
        )
        .unwrap();
    let foreign = world
        .runtime
        .prepare_historical_output_selection(
            execution,
            &world.installed,
            &basis,
            &basis,
            &foreign_meaning,
        )
        .unwrap();
    let foreign_commit = foreign.publication().authority().publication_coordinate().0;
    let admission = world
        .owner
        .admit_historical_output_selection([
            selected.publication().authority(),
            foreign.publication().authority(),
        ])
        .unwrap();
    let graph = world
        .runtime
        .prepare_owned_model_graph_selection(
            execution,
            &basis,
            &basis,
            &ownership(),
            &ownership(),
            world.model,
        )
        .unwrap();
    assert!(!graph
        .entities()
        .selected_entities()
        .contains(&foreign_output));
    let port = world.runtime.preparation_port();
    let input = port
        .admit_owned_graph_selection(execution, begin(&world.runtime), graph)
        .unwrap();
    let error = port
        .prepare_owned_graph_selection(
            execution,
            input,
            Some((
                &world.installed,
                RequiredOutputPublication::HistoricalSelection {
                    admission,
                    selections: vec![selected, foreign],
                },
            )),
        )
        .unwrap_err();
    let RelationalOwnedGraphSelectionPreparationError::Preparation(
        TransactionCommitError::Publication { error, .. },
    ) = error
    else {
        panic!("{error:?}")
    };
    assert!(
        matches!(error.authored_retirement_refusal(), Some(AuthoredRetirementRefusal::HistoricalSelectionNotOwned { publication }) if publication.0 == foreign_commit),
        "{error:?}"
    );
    assert_eq!(
        error.context.affected_records,
        [RecordRef::Entity(foreign_source)]
    );
    assert_eq!(
        world
            .runtime
            .observe_branch(&world.runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
    assert_eq!(
        world
            .runtime
            .observe_model_owner(execution, &basis, &ownership(), foreign_output)
            .unwrap(),
        Some(foreign_model)
    );
}
