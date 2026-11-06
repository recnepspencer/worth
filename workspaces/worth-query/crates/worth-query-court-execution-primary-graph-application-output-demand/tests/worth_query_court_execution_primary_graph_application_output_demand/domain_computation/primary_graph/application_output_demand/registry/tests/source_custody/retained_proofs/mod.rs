use crate::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::*;
#[test]
fn completed_multi_root_custody_waits_for_token_and_active_record() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let occurrence = receipt.product_branch().occurrence_for_test();
    let (scope, _) = real_scope_and_occurrence();
    let first =
        BoundOutputSource::new_for_test(scope, key("producer", 5, 1).source_into_for_test());
    let second =
        BoundOutputSource::new_for_test(scope, key("producer", 5, 2).source_into_for_test());
    let mut state = DemandRegistryState::default();
    state.source_custody.insert(commit.clone(), {
        // Bound in the original literal order, passed in the door's.
        let (
            cleanup,
            occurrence,
            root_kind,
            source,
            discovery,
            bound_sources,
            consumed_sources,
            retired_sources,
            retired,
            token_count,
            completed,
        ) = (
            Default::default(),
            occurrence,
            PreparedOutputRootKind::Discovered(TypeId::of_for_test::<()>()),
            None,
            None,
            Some(vec![first.clone(), second.clone()]),
            vec![
                (*first.identity_for_test()).clone(),
                (*second.identity_for_test()).clone(),
            ],
            Vec::new(),
            None,
            1,
            true,
        );
        SourceCustody::new_for_test(
            occurrence,
            root_kind,
            source,
            discovery,
            consumed_sources,
            retired_sources,
            bound_sources,
            retired,
            token_count,
            cleanup,
            completed,
        )
    });
    state.note_custody_for_test(&commit);
    state.prune_completed_custody().unwrap();
    assert_eq!(state.source_custody.len(), 1);
    (*state
        .source_custody
        .get_mut(&commit)
        .unwrap()
        .token_count_mut_for_test()) = 0;
    let demand_key = key("producer", 5, 1);
    let mut active = record(occurrence, DemandState::Admitted, 1);
    (*active.source_commits_mut_for_test()).push(commit.clone());
    (*state.records_mut_for_test()).insert_for_test(demand_key.clone(), active);
    state.note_custody_for_test(&commit);
    state.prune_completed_custody().unwrap();
    assert_eq!(state.source_custody.len(), 1);
    let mut record = (*state.records_mut_for_test())
        .get_mut(&demand_key)
        .unwrap();
    (*record.interests_mut_for_test()) = 0;
    (*record.state_mut_for_test()) =
        DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
            WorthQueryOutputDemandDenialKind::NoEffect,
            "terminal source",
        ));
    record.finish_for_test();
    state.note_custody_for_test(&commit);
    state.prune_completed_custody().unwrap();
    assert!(state.source_custody.is_empty());
}

#[test]
fn fully_superseded_multi_root_custody_prunes_after_its_last_token() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let (scope, _) = real_scope_and_occurrence();
    let first =
        BoundOutputSource::new_for_test(scope, key("producer", 5, 1).source_into_for_test());
    let second =
        BoundOutputSource::new_for_test(scope, key("producer", 5, 2).source_into_for_test());
    let superseded = WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::Superseded,
        "newer source owns this root",
    );
    let mut state = DemandRegistryState::default();
    state.source_custody.insert(commit.clone(), {
        // Bound in the original literal order, passed in the door's.
        let (
            cleanup,
            occurrence,
            root_kind,
            source,
            discovery,
            bound_sources,
            consumed_sources,
            retired_sources,
            retired,
            token_count,
            completed,
        ) = (
            Default::default(),
            receipt.product_branch().occurrence_for_test(),
            PreparedOutputRootKind::Discovered(TypeId::of_for_test::<()>()),
            None,
            None,
            Some(vec![first.clone(), second.clone()]),
            Vec::new(),
            vec![
                ((*first.identity_for_test()).clone(), superseded.clone()),
                ((*second.identity_for_test()).clone(), superseded),
            ],
            None,
            1,
            false,
        );
        SourceCustody::new_for_test(
            occurrence,
            root_kind,
            source,
            discovery,
            consumed_sources,
            retired_sources,
            bound_sources,
            retired,
            token_count,
            cleanup,
            completed,
        )
    });
    state.note_custody_for_test(&commit);
    state.prune_completed_custody().unwrap();
    assert_eq!(state.source_custody.len(), 1);
    (*state
        .source_custody
        .get_mut(&commit)
        .unwrap()
        .token_count_mut_for_test()) = 0;
    state.note_custody_for_test(&commit);
    state.prune_completed_custody().unwrap();
    assert!(state.source_custody.is_empty());
}
