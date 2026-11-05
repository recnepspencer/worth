use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::interest;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::support::record;
use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;

use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceCustody;
pub fn real_scope_and_occurrence() -> (
    worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding,
    worth_runtime_world::facade::ProductBranchIncarnation,
) {
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    world.application.with_host_advancement(|phase| {
        let custody = phase.test_host_request_for_test();
        let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
        let account =
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account(
                &custody,
                &world, "open", &request,
            );
        let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(account.entity_id_for_test());
        let product = world
            .application
            .product_runtime()
            .admit_product_branch_for_test(world.application.product_runtime().default_branch_for_test())
            .expect("the fixture product occurrence is live");
        (scope, product.observation_for_test().lifecycle_incarnation())
    }).expect("the fixture source read owns its host call")
}
pub fn custody_wait(superseded: bool, token: bool) {
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let (scope, _) = real_scope_and_occurrence();
    let registry = WorthQueryOutputDemandRegistry::default();
    let selected = key("producer", 5, 1);
    let sources = vec![
        BoundOutputSource::new_for_test(scope, (*selected.source_for_test()).clone()),
        BoundOutputSource::new_for_test(scope, key("producer", 5, 2).source_into_for_test()),
    ];
    {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        state.source_custody.insert(
            commit.clone(),
            SourceCustody::new_for_test(
                receipt.product_branch().occurrence_for_test(),
                PreparedOutputRootKind::Discovered(TypeId::of_for_test::<()>()),
                None,
                None,
                sources
                    .iter()
                    .map(|source| (*source.identity_for_test()).clone())
                    .collect(),
                if superseded {
                    sources
                        .iter()
                        .map(|source| {
                            (
                                (*source.identity_for_test()).clone(),
                                WorthQueryOutputDemandDenial::new_for_test(
                                    WorthQueryOutputDemandDenialKind::Superseded,
                                    "newer source",
                                ),
                            )
                        })
                        .collect()
                } else {
                    Vec::new()
                },
                Some(sources),
                None,
                usize::from(token),
                Default::default(),
                !superseded,
            ),
        );
        let mut active = record(
            receipt.product_branch().occurrence_for_test(),
            DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                WorthQueryOutputDemandDenialKind::NoEffect,
                "terminal source",
            )),
            usize::from(!token),
        );
        (*active.source_commits_mut_for_test()).push(commit.clone());
        (*state.records_mut_for_test()).insert_for_test(selected.clone(), active);
    }
    registry.complete_prepared_source_for_test(&receipt);
    (*registry.state_for_test())
        .lock()
        .unwrap()
        .drain_completed_custody_for_test();
    {
        let state = (*registry.state_for_test()).lock().unwrap();
        assert_eq!(state.source_custody.len(), 1);
        assert!((*state.source_custody[&commit].cleanup_for_test()).queued);
    }
    if token {
        registry.release_prepared_token_for_test(&commit);
    } else {
        let notifications = Arc::clone(
            &(*(*(*registry.state_for_test())
                .lock()
                .unwrap()
                .records_for_test())[&selected]
                .wake_for_test()),
        );
        drop(interest(&registry, selected.clone(), notifications));
    }
    (*registry.state_for_test())
        .lock()
        .unwrap()
        .drain_completed_custody_for_test();
    let state = (*registry.state_for_test()).lock().unwrap();
    assert!(state.source_custody.is_empty());
    assert!((*state.records_for_test())
        .values_for_test()
        .all(|row| (*row.source_commits_for_test()).is_empty()));
    assert!(!state.pending_custody_prune_for_test());
}
