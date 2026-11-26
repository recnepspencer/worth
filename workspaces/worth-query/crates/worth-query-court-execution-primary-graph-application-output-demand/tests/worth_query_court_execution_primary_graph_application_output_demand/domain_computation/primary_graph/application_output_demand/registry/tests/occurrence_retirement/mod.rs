//! A retiring occurrence closes live work and removes unheld cached work.
use super::*;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready;

/// Retirement releases a whole claim chain, not one hop. Releasing the leaf's
/// claim queues the middle row with its own claim on the upstream, and no
/// admitted drain comes for a retired occurrence, so a one-hop release keeps
/// both rows and the World history their outputs observe (public #358).
#[test]
fn retiring_occurrence_releases_its_whole_claim_chain() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let upstream = key("chain-upstream", 1, 1);
    let middle = key("chain-middle", 1, 1);
    let leaf = key("chain-leaf", 1, 1);
    {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        for (row, claim) in [(&upstream, None), (&middle, Some(&upstream)), (&leaf, Some(&middle))] {
            let mut row_record = record(occurrence, ready(), 0);
            *row_record.framework_required_count_mut_for_test() = usize::from(row != &leaf);
            *row_record.prerequisites_mut_for_test() = claim.map(|claim| Arc::new(claim.clone())).into_iter().collect();
            (*state.records_mut_for_test()).insert_for_test(row.clone(), row_record);
        }
    }

    registry.release_product_occurrence_for_test(occurrence);

    let state = (*registry.state_for_test()).lock().unwrap();
    for row in [&upstream, &middle, &leaf] {
        assert!(
            !(*state.records_for_test()).contains_key_for_test(row),
            "{row:?} outlived its occurrence's retirement"
        );
    }
}

#[test]
fn retiring_occurrence_closes_live_work_and_completion_cannot_resurrect_it() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let demand_key = key("producer", 5, 1);
    let wake = Arc::new(DemandWake::new_for_test(Mutex::new(0), Condvar::new()));
    (*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_mut_for_test())
    .insert_for_test(demand_key.clone(), {
        let __wake = Arc::clone(&wake);
        let mut __base = record(occurrence, DemandState::Scheduling, 1);
        *__base.wake_mut_for_test() = __wake;
        __base
    });
    let demand_interest = interest(&registry, demand_key.clone(), wake);

    registry.release_product_occurrence_for_test(occurrence);
    registry.finish_scheduling_for_test(
        &demand_interest,
        None,
        &mut Ok(WorthQueryOutputSchedulingResult::Scheduled),
    );

    assert!(matches!(
        (*(*registry.state_for_test()).lock().unwrap().records_for_test())[&demand_key].state_for_test(),
        DemandState::Failed(denial)
            if denial.kind() == WorthQueryOutputDemandDenialKind::Closed
    ));
    drop(demand_interest);
    assert!(!(*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_for_test())
    .contains_key_for_test(&demand_key));
}

#[test]
fn retiring_occurrence_removes_unheld_cached_work_and_marks_preparation_closed() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let occurrence = occurrence();
    let scheduled = key("scheduled", 1, 1);
    (*(*registry.state_for_test())
        .lock()
        .unwrap()
        .records_mut_for_test())
    .insert_for_test(
        scheduled.clone(),
        record(occurrence, DemandState::Scheduled, 0),
    );
    let preparation = registry.begin_source_preparation_for_test(occurrence);

    registry.release_product_occurrence_for_test(occurrence);

    let state = (*registry.state_for_test()).lock().unwrap();
    assert!(!(*state.records_for_test()).contains_key_for_test(&scheduled));
    assert!((*(*state.source_preparations_for_test())[&occurrence].retired_for_test()));
    assert!(state
        .source_custody
        .values()
        .all(|custody| custody.prepared_count_for_test() == 0));
    drop(state);
    drop(preparation);
    assert!((*(*registry.state_for_test())
        .lock()
        .unwrap()
        .source_preparations_for_test())
    .is_empty());
}
