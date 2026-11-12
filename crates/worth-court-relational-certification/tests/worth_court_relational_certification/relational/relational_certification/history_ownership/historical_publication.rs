//! Historical selection reissues custody through both independent Native writers.
use super::publication_fixture::PublishedModel;
use super::*;
use worth_relational::facade::mvcc::*;
#[test]
fn selected_publication_keeps_original_ids_under_fresh_commit_and_cold_current_custody() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(execution);
    let (_, selected) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let prior = world
        .runtime
        .prepare_historical_output_selection(
            execution,
            &world.installed,
            &selected,
            &selected,
            &world.meaning,
        )
        .unwrap();
    let old_publication = prior.publication().clone();
    assert!(old_publication
        .matches_current_ownership_at(execution, &world.installed, &selected)
        .unwrap());
    let original_ids: Vec<_> = old_publication.members().map(|(_, _, _, id)| id).collect();
    assert_eq!(original_ids, [world.output]);
    create_entity(execution, &world.runtime, "outside-current-edit");
    let (_, current) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let selected_output = world
        .runtime
        .prepare_historical_output_selection(
            execution,
            &world.installed,
            &current,
            &selected,
            &world.meaning,
        )
        .unwrap();
    let admission = world
        .owner
        .admit_historical_output_selection([selected_output.publication().authority()])
        .unwrap();
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
                    selections: vec![selected_output],
                },
            )),
        )
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = world
        .runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("selected publication must perform atomically");
    };
    let committed = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    let publications: Vec<_> = committed
        .required_output_publications(&world.installed, execution)
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(publications.len(), 1);
    assert_eq!(
        publications[0]
            .members()
            .map(|(_, _, _, id)| id)
            .collect::<Vec<_>>(),
        original_ids
    );
    assert_eq!(publications[0].meaning(), old_publication.meaning());
    assert_eq!(publications[0].producer(), old_publication.producer());
    let (_, after) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    assert_eq!(
        world
            .runtime
            .observe_model_owner(execution, &after, &ownership(), world.output)
            .unwrap(),
        Some(world.model)
    );
    assert!(publications[0]
        .matches_current_ownership_at(execution, &world.installed, &after)
        .unwrap());
    assert!(
        !old_publication
            .matches_current_ownership_at(execution, &world.installed, &after)
            .unwrap(),
        "equal output identities and values cannot keep a displaced publication current"
    );
    let obligations = world
        .runtime
        .observe_owned_model_publications(
            execution,
            &world.installed,
            &after,
            &ownership(),
            world.model,
        )
        .unwrap();
    let mut routes = obligations.routes();
    let Some(worth_relational::facade::branch::RelationalOwnedModelOutputRoute::Published(route)) =
        routes.next()
    else {
        panic!("selected live obligation");
    };
    assert!(routes.next().is_none());
    assert!(route
        .matches_publication(execution, &publications[0])
        .unwrap());
    assert!(
        !route
            .matches_publication(execution, &old_publication)
            .unwrap(),
        "equal meaning is insufficient after a different publication commit"
    );
    let antecedent = world
        .runtime
        .readmit_restored_producer_prior(execution, &world.installed, &after, &world.meaning)
        .unwrap()
        .unwrap();
    assert!(antecedent.publication().is_historical_selection());
    assert!(antecedent.matches_producer_publication(&old_publication));
    assert_eq!(antecedent.producer_scope(), world.source);
    let warm = world
        .runtime
        .prepare_historical_output_selection(
            execution,
            &world.installed,
            &after,
            &after,
            &world.meaning,
        )
        .unwrap();
    assert_eq!(
        warm.publication().members().collect::<Vec<_>>(),
        publications[0].members().collect::<Vec<_>>()
    );
    release_test_commit_snapshot(&world.runtime, &committed);
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
    let restored = cold
        .prepare_historical_output_selection(execution, &installed, &basis, &basis, &world.meaning)
        .unwrap();
    assert_eq!(
        restored.publication().members().collect::<Vec<_>>(),
        warm.publication().members().collect::<Vec<_>>()
    );
    assert_eq!(
        restored.publication().meaning(),
        warm.publication().meaning()
    );
    assert_eq!(
        cold.observe_model_owner(execution, &basis, &ownership(), world.output)
            .unwrap(),
        Some(world.model)
    );
    let predecessor = warm.publication().clone();
    let admission = world
        .owner
        .admit_required_output_succession(
            world.owner.admit_required_output("generator", [2; 32]),
            predecessor.authority(),
        )
        .unwrap();
    let proposal = begin(&world.runtime)
        .validate(execution, &world.runtime)
        .unwrap()
        .bind_required_output_publication(
            &world.installed,
            RequiredOutputPublication::Preserve {
                admission,
                predecessor,
                meaning: world.meaning.clone(),
                roles: vec![RequiredOutputRoleTarget {
                    role: "result".into(),
                    entity_name: "Member".into(),
                    kind: KindId(3),
                    posture: RequiredOutputRolePosture::Preserve,
                    target: EntityReference::Existing(world.output),
                }],
            },
        )
        .unwrap();
    let candidate = world
        .runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = world
        .runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the original producer must Preserve its selected identities");
    };
    let preserved = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    let own = preserved
        .required_output_publications(&world.installed, execution)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        own[0].members().map(|(_, _, _, id)| id).collect::<Vec<_>>(),
        original_ids
    );
    release_test_commit_snapshot(&world.runtime, &preserved);
}

#[test]
fn an_owned_graph_selection_cannot_omit_its_publication_custody() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(execution);
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
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
    let port = world.runtime.preparation_port();
    let input = port
        .admit_owned_graph_selection(execution, begin(&world.runtime), graph)
        .unwrap();
    let error = port
        .prepare_owned_graph_selection(execution, input, None)
        .unwrap_err();
    let RelationalOwnedGraphSelectionPreparationError::Preparation(
        TransactionCommitError::Publication { error, .. },
    ) = error
    else {
        panic!("{error:?}");
    };
    assert!(
        matches!(error.authored_retirement_refusal(), Some(worth_relational::facade::publication::AuthoredRetirementRefusal::OwnedPublicationRequiresCascade { owner, .. }) if owner == world.source)
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
}

#[test]
fn historical_selection_requires_the_installed_producer_source_kind() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let world = PublishedModel::new(execution);
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let wrong = world
        .installed
        .clone()
        .with_source_scope_owned_producers([("generator".into(), KindId(1))]);
    let error = world
        .runtime
        .prepare_historical_output_selection(execution, &wrong, &basis, &basis, &world.meaning)
        .unwrap_err();
    let TransactionCommitError::Publication { error, .. } = error else {
        panic!("{error:?}");
    };
    assert!(matches!(error.authored_retirement_refusal(), Some(worth_relational::facade::publication::AuthoredRetirementRefusal::HistoricalSelectionNotOwned { .. })));
    assert_eq!(
        error.context.affected_records,
        [RecordRef::Entity(world.source)]
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
}
