use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;

#[test]
fn unchanged_source_after_unrelated_world_work_joins_the_active_demand() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let older = key_with_identity("producer", 5, 1, 90);
    let later_same_source = key_with_identity("producer", 6, 1, 90);
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    (*(*registry.state_for_test()).lock().unwrap().records_mut_for_test()).insert_for_test(
        older.clone(),
        { let __source_scope = Some(scope); let mut __base = record(occurrence(), DemandState::Scheduled, 1); *__base.source_scope_mut_for_test() = __source_scope; __base },
    );

    let joined = registry
        .admit_for_test(
            later_same_source,
            None,
            scope,
            occurrence(),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
            None,
            None,
        )
        .expect("an unchanged semantic source joins its active demand")
        .0;

    let state = (*registry.state_for_test()).lock().unwrap();
    assert_eq!((*state.records_for_test()).len_for_test(), 1);
    assert_eq!((*joined.key_for_test()), older);
    assert_eq!((*(*state.records_for_test())[&older].interests_for_test()), 2);
}

#[test]
fn semantic_join_deterministically_selects_the_latest_active_generation() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let oldest = key_with_identity("producer", 5, 1, 90);
    let latest = key_with_identity("producer", 6, 1, 90);
    let requested = key_with_identity("producer", 7, 1, 90);
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    let mut state = (*registry.state_for_test()).lock().unwrap();
    for key in [&oldest, &latest] {
        (*state.records_mut_for_test()).insert_for_test(
            key.clone(),
            { let __source_scope = Some(scope); let mut __base = record(occurrence(), DemandState::Scheduled, 1); *__base.source_scope_mut_for_test() = __source_scope; __base },
        );
    }
    drop(state);

    let joined = registry
        .admit_for_test(
            requested,
            None,
            scope,
            occurrence(),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
            None,
            None,
        )
        .expect("semantic join chooses one active owner deterministically")
        .0;

    assert_eq!((*joined.key_for_test()), latest);
    assert_eq!((*(*(*registry.state_for_test()).lock().unwrap().records_for_test())[&latest].interests_for_test()), 2);
}

#[test]
fn changed_source_can_return_after_its_prior_cycle_was_superseded() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let source_a = key_with_identity("producer", 5, 1, 90);
    let source_b = key_with_identity("producer", 6, 1, 91);
    let returned_a = key_with_identity("producer", 7, 1, 90);
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    (*(*registry.state_for_test()).lock().unwrap().records_mut_for_test()).insert_for_test(
        source_a.clone(),
        { let __source_scope = Some(scope); let mut __base = record(occurrence(), DemandState::Scheduled, 1); *__base.source_scope_mut_for_test() = __source_scope; __base },
    );

    let _source_b_interest = registry
        .admit_for_test(
            source_b.clone(),
            None,
            scope,
            occurrence(),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
            None,
            None,
        )
        .expect("the changed source supersedes its predecessor")
        .0;
    let returned_interest = registry
        .admit_for_test(
            returned_a.clone(),
            None,
            scope,
            occurrence(),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary,
            None,
            None,
        )
        .expect("a later return is a new source cycle, not the stopped old record")
        .0;

    let state = (*registry.state_for_test()).lock().unwrap();
    assert!(matches!(
        &(*(*state.records_for_test())[&source_a].state_for_test()),
        DemandState::Failed(denial)
            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    assert!(matches!(
        &(*(*state.records_for_test())[&source_b].state_for_test()),
        DemandState::Failed(denial)
            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
    ));
    assert_eq!((*returned_interest.key_for_test()), returned_a);
    assert_ne!(source_a, returned_a);
}

#[test]
fn unrelated_world_generation_does_not_order_equal_custody_sources() {
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    let older = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource::new_for_test(scope, key_with_identity("producer", 5, 1, 90).source_into_for_test());
    let later_same_source = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource::new_for_test(scope, key_with_identity("producer", 6, 1, 90).source_into_for_test());

    assert_eq!(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::root_revision_order(&older, &later_same_source),
        Some(std::cmp::Ordering::Equal)
    );
}

#[test]
fn distinct_source_queries_are_not_ordered_or_semantically_joined() {
    let first = key_with_query_identity("producer", 5, 1, 90, 1);
    let second = key_with_query_identity("producer", 6, 1, 90, 2);
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    let first_root = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource::new_for_test(scope, (*first.source_for_test()).clone());
    let second_root = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource::new_for_test(scope, (*second.source_for_test()).clone());

    assert!(!first.same_occurrence_for_test(&second));
    assert!(!first.same_semantic_source_for_test(&second));
    assert_eq!(first.replacement_order_for_test(&second), None);
    assert_eq!(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::root_revision_order(&first_root, &second_root),
        None
    );
}

#[test]
fn recovery_rejoins_the_older_key_that_retains_newer_semantic_custody() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let occurrence = receipt.product_branch().occurrence_for_test();
    let registry = WorthQueryOutputDemandRegistry::default();
    let older = key_with_identity("producer", 5, 1, 90);
    let retained = key_with_identity("producer", 6, 1, 90);
    let newer_non_owner = key_with_identity("producer", 7, 1, 90);
    let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
    let mut retained_record = record(occurrence, DemandState::Scheduled, 0);
    (*retained_record.required_interests_mut_for_test()) = 1;
    (*retained_record.source_scope_mut_for_test()) = Some(scope);
    (*retained_record.source_commits_mut_for_test()).push(commit.clone());
    let mut state = (*registry.state_for_test()).lock().unwrap();
    (*state.records_mut_for_test()).insert_for_test(older.clone(), retained_record);
    (*state.records_mut_for_test()).insert_for_test(
        newer_non_owner.clone(),
        { let __required_interests = 1; let __source_scope = Some(scope); let mut __base = record(occurrence, DemandState::Scheduled, 1); *__base.required_interests_mut_for_test() = __required_interests; *__base.source_scope_mut_for_test() = __source_scope; __base },
    );
    state.source_custody.insert(
        commit.clone(),
        {
// Bound in the original literal order, passed in the door's.
let (occurrence, root_kind, source, discovery, bound_sources, consumed_sources, retired_sources, retired, token_count, cleanup, completed) = (occurrence, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind::Discovered(TypeId::of_for_test::<()>()), None, None, Some(vec![worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource::new_for_test(scope, (*retained.source_for_test()).clone())]), vec![(*retained.source_for_test()).clone()], Vec::new(), None, 1, Default::default(), false);
worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceCustody::new_for_test(occurrence, root_kind, source, discovery, consumed_sources, retired_sources, bound_sources, retired, token_count, cleanup, completed)
},
    );
    drop(state);

    let recovered = registry
        .admit_for_test(
            retained,
            None,
            scope,
            occurrence,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Recovery,
            Some(&commit),
            None,
        )
        .expect("recovery finds the semantic record that owns retained custody")
        .0;

    let state = (*registry.state_for_test()).lock().unwrap();
    assert_eq!((*recovered.key_for_test()), older);
    assert_eq!((*state.records_for_test()).len_for_test(), 2);
    assert_eq!((*(*state.records_for_test())[&older].interests_for_test()), 1);
    assert_eq!((*(*state.records_for_test())[&newer_non_owner].interests_for_test()), 1);
}
