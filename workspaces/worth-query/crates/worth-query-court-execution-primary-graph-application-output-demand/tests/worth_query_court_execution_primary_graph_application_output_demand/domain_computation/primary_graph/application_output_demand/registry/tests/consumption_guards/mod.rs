//! Root consumption is returned only for an abandoned, undelivered answer.
//! Receipts and source payloads come from real commits; stages are assigned
//! at the registry seam that owns disposal, not inferred from a handler run.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceCustody;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryPendingOutputDelivery;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryPerformedOutputDemandSource;
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support;
use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId;

pub fn retained_source(
    identity: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceEpoch,
) -> SourceCustody {
    let mut receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application_with_output_demand_observation();
    let change = Arc::new(receipt.take_performed_relational_product_change().unwrap());
    let observation = receipt
        .committed_product_publication()
        .take_output_demand_observation_for_test()
        .unwrap();
    let source = WorthQueryPerformedOutputDemandSource {
        receipt,
        change,
        observation,
        output_source_identity: None,
    };
    let scope = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::real_scope_and_occurrence().0;
    SourceCustody {
        occurrence: (*source.receipt_for_test()).product_branch().occurrence(),
        root_kind: PreparedOutputRootKind::Discovered(EqualityTypeId::of_for_test::<()>()),
        source: Some(source),
        discovery: None,
        bound_sources: Some(vec![BoundOutputSource { scope, identity }]),
        consumed_sources: Vec::new(),
        retired_sources: Vec::new(),
        retired: None,
        token_count: 1,
        cleanup: Default::default(),
        completed: false,
    }
}

pub fn repeat_source(initial: &SourceCustody) -> SourceCustody {
    SourceCustody {
        occurrence: (*initial.occurrence_for_test()),
        root_kind: (*initial.root_kind_for_test()).clone(),
        source: (*initial.source_for_test()).clone(),
        discovery: None,
        bound_sources: (*initial.bound_sources_for_test()).clone(),
        consumed_sources: Vec::new(),
        retired_sources: Vec::new(),
        retired: None,
        token_count: 1,
        cleanup: Default::default(),
        completed: false,
    }
}

#[test]
fn completed_and_retired_custody_return_no_consumption() {
    let identity = (*key("guarded-root", 5, 1).source_for_test());
    let initial = retained_source(identity.clone());
    let scope = (*initial.bound_sources_for_test()).as_ref().unwrap()[0].scope;
    for completed in [false, true] {
        let mut custody = repeat_source(&initial);
        (*custody.consumed_sources_for_test()).push(identity.clone());
        if completed {
            (*custody.completed_mut_for_test()) = true;
        } else {
            // Keep the payload so this row independently pins the retired guard.
            (*custody.retired_mut_for_test()) = Some(WorthQueryOutputDemandDenial::new_for_test(
                WorthQueryOutputDemandDenialKind::Superseded,
                "retired root",
            ));
        }
        custody.return_undelivered_root_for_test(&identity, scope);
        assert_eq!(
            (*custody.consumed_sources_for_test()).as_slice(),
            std::slice::from_ref(&identity)
        );
    }
}

#[test]
fn delivered_and_ready_roots_keep_their_consumption_on_removal() {
    let row_key = key("delivered-root", 5, 1);
    let initial = retained_source((*row_key.source_for_test()).clone());
    let source = (*initial.source_for_test()).as_ref().unwrap();
    let commit = source.change.product_commit().clone();
    let scope = (*initial.bound_sources_for_test()).as_ref().unwrap()[0].scope;
    for ready in [false, true] {
        let mut state = DemandRegistryState::default();
        let mut custody = repeat_source(&initial);
        (*custody.consumed_sources_for_test()).push((*row_key.source_for_test()).clone());
        state.source_custody.insert(commit.clone(), custody);
        let stage = if ready {
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready()
        } else {
            let backing = PreparedReadyBacking::new_for_test();
            DemandState::Output(WorthQueryOutputProgress::new_for_test(
                WorthQueryOutputCheckpoint::Delivered {
                    receipt: source.receipt.clone(),
                    delivery: None,
                    ready_backing: backing,
                },
            ))
        };
        let mut row = record((*initial.occurrence_for_test()), stage, 0);
        (*row.source_scope_mut_for_test()) = Some(scope);
        (*row.source_commits_for_test()).push(commit.clone());
        (*state.records_for_test()).insert(row_key.clone(), row);
        drop(state.remove_record(&row_key).unwrap());
        assert_eq!(
            state.source_custody[&commit].consumed_sources.as_slice(),
            std::slice::from_ref(&(*row_key.source_for_test())),
            "ready={ready}: delivered work cannot be admitted again"
        );
    }
}

#[test]
fn an_abandoned_join_returns_the_requested_source_it_consumed() {
    let older = key_with_identity("joined-root", 5, 1, 90);
    let requested = key_with_identity("joined-root", 6, 1, 90);
    assert_ne!((*older.source_for_test()), (*requested.source_for_test()));
    assert!(older.same_semantic_source_for_test(&requested));
    let initial = retained_source((*requested.source_for_test()).clone());
    let source = (*initial.source_for_test()).as_ref().unwrap();
    let commit = source.change.product_commit().clone();
    let scope = (*initial.bound_sources_for_test()).as_ref().unwrap()[0].scope;
    let registry = WorthQueryOutputDemandRegistry::default();
    {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        state
            .source_custody
            .insert(commit.clone(), repeat_source(&initial));
        let mut row = record((*initial.occurrence_for_test()), DemandState::Admitted, 0);
        (*row.source_scope_mut_for_test()) = Some(scope);
        state.records.insert(older.clone(), row);
    }
    let (interest, _) = registry
        .admit_for_test(
            requested.clone(),
            Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::admission::selected_output::SelectedOutputAdmission::new(&commit, None)),
            scope,
            (*initial.occurrence_for_test()),
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Required,
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        interest.key, older,
        "the admission actually joins the older key"
    );
    let mut state = (*registry.state_for_test()).lock().unwrap();
    assert_eq!(
        state.source_custody[&commit].consumed_sources.as_slice(),
        std::slice::from_ref(&(*requested.source_for_test()))
    );
    let backing = PreparedReadyBacking::new_for_test();
    let mut row = state.records.get_mut(&older).unwrap();
    row.state = DemandState::Output(WorthQueryOutputProgress::new_for_test(
        WorthQueryOutputCheckpoint::Published {
            receipt: source.receipt.clone(),
            delivery: WorthQueryPendingOutputDelivery::NoChange,
            ready_backing: backing,
        },
    ));
    row.performed_source = None; // The real publication finish releases this payload.
    row.interests = 0;
    row.finish();
    drop(state.remove_record(&older).unwrap());
    assert!(
        state.source_custody[&commit].consumed_sources.is_empty(),
        "the returned identity is the requested source, not the older row key"
    );
    drop(state);
    drop(interest);
}
