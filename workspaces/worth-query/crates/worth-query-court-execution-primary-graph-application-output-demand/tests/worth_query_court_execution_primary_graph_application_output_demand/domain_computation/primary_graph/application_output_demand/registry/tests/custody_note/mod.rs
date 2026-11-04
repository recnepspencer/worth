//! A held custody note survives rotation until the last stale claim leaves.
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::BoundOutputSource;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::source_custody::PreparedOutputRootKind;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::SourceCustody;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
pub use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId;

pub fn fixture(
    receipt: &WorthQueryApplicationCommitReceipt,
    ready: &DemandState,
    seed: u64,
    scope: worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding,
) -> (
    WorthQueryOutputDemandRegistry,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandKey,
) {
    let registry = WorthQueryOutputDemandRegistry::default();
    let old = key_with_identity("upstream", seed + 1, 1, 10);
    let newest = key_with_identity("upstream", seed + 2, 1, 11);
    let dependent = key("dependent", seed + 1, 2);
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let mut state = (*registry.state_for_test()).lock().unwrap();
    let mut next = record(
        receipt.product_branch().occurrence_for_test(),
        copy_ready(ready),
        1,
    );
    (*next.source_commits_mut_for_test()).push(commit.clone());
    let next_wake = Arc::clone(&(*next.wake_for_test()));
    (*state.records_mut_for_test()).insert_for_test(newest.clone(), next);
    (*state.records_mut_for_test()).insert_for_test(old.clone(), {
        let __framework_required_count = 1;
        let mut __base = record(
            occurrence(),
            DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                WorthQueryOutputDemandDenialKind::Superseded,
                "newer row",
            )),
            0,
        );
        *__base.framework_required_count_mut_for_test() = __framework_required_count;
        __base
    });
    let row = {
        let __prerequisites = vec![Arc::new(old)];
        let mut __base = record(occurrence(), copy_ready(ready), 1);
        *__base.prerequisites_mut_for_test() = __prerequisites;
        __base
    };
    let mut row = row;
    (*row.required_interests_mut_for_test()) = 1;

    let wake = Arc::clone(&(*row.wake_for_test()));
    (*state.records_mut_for_test()).insert_for_test(dependent.clone(), row);
    let member = state.prepare_required_member_for_test(&newest).unwrap();
    state.install_required_member_for_test(member);
    state.source_custody.insert(commit, {
        // Bound in the original literal order, passed in the door's.
        let (
            occurrence,
            root_kind,
            source,
            discovery,
            bound_sources,
            consumed_sources,
            retired_sources,
            retired,
            token_count,
            cleanup,
            completed,
        ) = (
            receipt.product_branch().occurrence_for_test(),
            PreparedOutputRootKind::Discovered(EqualityTypeId::of_for_test::<()>()),
            None,
            None,
            Some(vec![BoundOutputSource::new_for_test(
                scope,
                (*newest.source_for_test()).clone(),
            )]),
            vec![(*newest.source_for_test()).clone()],
            Vec::new(),
            None,
            0,
            Default::default(),
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
    retain_owned_key(&mut state, &dependent);
    retain_owned_key(&mut state, &newest);
    drop(state);
    registry.complete_prepared_source_for_test(receipt);
    let next_owner = interest(&registry, newest.clone(), next_wake);
    let dependent_owner = required_interest(&registry, dependent, wake);
    (registry, next_owner, dependent_owner, newest)
}

#[test]
fn registry_custody_note_survives_a_held_drain_and_claim_transfer() {
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::receipt_batch().remove(0);
    let ready = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready();
    let (registry, owner, dependent, newest) = fixture(
        &receipt,
        &ready,
        0,
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::source_custody::real_scope_and_occurrence().0,
    );
    let commit = receipt.committed_product_publication().composite_commit();
    drop(owner);
    (*registry.state_for_test())
        .lock()
        .unwrap()
        .drain_next_foreign_custody_for_test();
    {
        let state = (*registry.state_for_test()).lock().unwrap();
        assert!(
            state.source_custody.contains_key(commit),
            "stale claim still holds custody"
        );
        assert!(
            (*state.source_custody[commit].cleanup_for_test()).queued,
            "held debt stays noted"
        );
    }
    drop(dependent);
    registry.drain_closed_cleanup_admitted().unwrap();
    let mut state = (*registry.state_for_test()).lock().unwrap();
    assert_eq!(
        (*(*state.records_for_test())[&newest].framework_required_count_for_test()),
        1
    );
    state.drain_next_foreign_custody_for_test();
    assert!(!state.source_custody.contains_key(commit));
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::post_commit::custody_reader::resolve_before_binding(&mut state, &receipt);
    assert!(
        !state.pending_custody_prune_for_test(),
        "binding cannot see the pruned newer source"
    );
}

pub fn copy_ready(ready: &DemandState) -> DemandState {
    let DemandState::Output(output) = ready else {
        panic!("Ready fixture")
    };
    let Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint::Ready(completion)) = &(*output.checkpoint_for_test())
    else {
        panic!("Ready fixture")
    };
    DemandState::Output(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress::new_for_test(
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint::Ready(completion.clone()),
    ))
}

#[path = "differential/mod.rs"]
pub mod differential;
