//! Binding without an admission must not read pending completed custody as truth.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceCustody;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId;

#[test]
fn registry_binding_reader_ignores_logically_absent_completed_custody() {
    let world = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true);
    let mut active = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::commit_on_world_with_output_demand_observation(
        &world,
        201,
        "open",
        "binding-active",
    );
    let change = Arc::new(active.take_performed_relational_product_change().unwrap());
    let observation = active
        .committed_product_publication()
        .take_output_demand_observation_for_test()
        .unwrap();
    let source = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryPerformedOutputDemandSource::new_for_test(active, change, observation, None);
    let completed = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::commit_on_world_with_output_demand_observation(
        &world,
        202,
        "binding-active",
        "binding-completed",
    );
    let current_commit = (*source.change_for_test()).product_commit().clone();
    let completed_commit = completed
        .committed_product_publication()
        .composite_commit()
        .clone();
    let occurrence = (*source.receipt_for_test()).product_branch().occurrence_for_test();
    let (scope, _) = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::real_scope_and_occurrence();
    for seed in 0..16 {
        let selected = BoundOutputSource::new_for_test(scope, key("binding", seed + 1, 3).source_into_for_test());
        let newer = BoundOutputSource::new_for_test(scope, key("binding", seed + 2, 3).source_into_for_test());
        let make = || {
            let registry = WorthQueryOutputDemandRegistry::default();
            let mut state = (*registry.state_for_test()).lock().unwrap();
            state.source_custody.insert(
                current_commit.clone(),
                SourceCustody::new_for_test(occurrence, PreparedOutputRootKind::Discovered(EqualityTypeId::of_for_test::<()>()), Some(source.clone()), None, None, Vec::new(), Vec::new(), None, 1, Default::default(), false),
            );
            state.source_custody.insert(
                completed_commit.clone(),
                SourceCustody::new_for_test(occurrence, PreparedOutputRootKind::Discovered(EqualityTypeId::of_for_test::<()>()), None, None, Some(vec![newer.clone()]), vec![(*newer.identity_for_test()).clone()], Vec::new(), None, 0, Default::default(), true),
            );
            drop(state);
            registry.complete_prepared_source_for_test(&completed);
            registry
        };
        let eager = make();
        (*eager.state_for_test()).lock().unwrap().prune_completed_custody();
        eager
            .bind_prepared_output_source_for_test(&current_commit, selected.clone())
            .unwrap();
        let deferred = make();
        assert!(
            (*(*deferred.state_for_test()).lock().unwrap().source_custody[&completed_commit]
                .source_for_test())
                .is_none()
        );
        assert_eq!(
            deferred.retained_source_custody_count(),
            1,
            "a completed cleanup note retains no performed source"
        );
        assert!((*deferred.state_for_test()).lock().unwrap().pending_custody_prune_for_test());
        deferred
            .bind_prepared_output_source_for_test(&current_commit, selected.clone())
            .unwrap();
        let a = (*eager.state_for_test()).lock().unwrap();
        let mut b = (*deferred.state_for_test()).lock().unwrap();
        assert!(a.source_custody[&current_commit]
            .source_denial_for_test(&(*selected.identity_for_test()))
            .is_none());
        assert_eq!(
            (*b.source_custody[&current_commit].retired_sources_for_test()),
            (*a.source_custody[&current_commit].retired_sources_for_test()),
            "seed {seed}"
        );
        // The admission-free reader drains only its own named custody; the
        // completed note stays queued for its cleanup owner, but no reader
        // sees it: both registries expose the same visible custody.
        let visible = |state: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRegistryState| {
            state
                .source_custody
                .keys()
                .filter(|commit| !state.custody_prunable_for_test(commit))
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(visible(&b), visible(&a), "seed {seed}");
        assert!(b.custody_prunable_for_test(&completed_commit));
        assert!(b.pending_custody_prune_for_test());
        // Its production owner, the foreign-cleanup drain admission runs,
        // then reclaims it without the binding's help.
        b.drain_next_foreign_custody_for_test();
        assert!(!b.source_custody.contains_key(&completed_commit));
        assert!(!b.pending_custody_prune_for_test());
        assert_eq!(b.source_custody.len(), a.source_custody.len());

        assert!((*b.records_for_test()).is_empty_for_test());
    }
}
