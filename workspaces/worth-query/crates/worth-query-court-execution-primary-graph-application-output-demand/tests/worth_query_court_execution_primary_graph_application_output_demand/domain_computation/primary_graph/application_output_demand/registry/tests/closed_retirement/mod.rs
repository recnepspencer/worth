use std::sync::Arc;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandRecord;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::claim_on_superseded;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::claims;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::interest;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key_with_identity;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready;

use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
#[test]
fn a_closing_dependent_keeps_its_claim_while_the_newest_row_still_refreshes() {
    let (registry, interest, old, newest) = claim_on_superseded(DemandState::Running);
    let dependent = interest.key.clone();
    drop(interest);
    let state = registry.state.lock().unwrap();
    // The superseded Ready stays for a World that supersedes the refresh.
    assert_eq!(state.records[&old].framework_required_count, 1);
    assert_eq!(state.records[&newest].framework_required_count, 0);
    assert_eq!(claims(&state.records[&dependent]), vec![old]);
}
#[test]
fn a_reclaim_keeps_a_superseded_row_whose_claim_did_not_move() {
    let (registry, interest, old, newest) = claim_on_superseded(ready());
    let dependent = interest.key.clone();
    {
        let mut state = registry.state.lock().unwrap();
        // The newest row is no required member: the claim has no key to move to.
        state.required_keys.clear();
    }
    // A refused observation asks the registry for an unheld row. The superseded
    // row's claim cannot move, so it is passed over and nothing is reclaimed.
    assert!(!registry.reclaim_unheld_observation().unwrap());
    {
        let state = registry.state.lock().unwrap();
        assert_eq!(state.records[&old].framework_required_count, 1);
        assert_eq!(claims(&state.records[&dependent]), vec![old.clone()]);
    }
    // The dependent's own retirement releases the claim on a row still there.
    registry
        .state
        .lock()
        .unwrap()
        .records
        .get_mut(&dependent)
        .unwrap()
        .state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        "dependent retired",
    ));
    drop(interest);
    while registry.reclaim_unheld_observation().unwrap() {}
    registry.drain_closed_cleanup_admitted().unwrap();
    let state = registry.state.lock().unwrap();
    assert!(!state.records.contains_key(&dependent));
    assert!(!state.records.contains_key(&old));
    assert!(state.records.contains_key(&newest));
}
#[test]
fn an_unpublished_refresh_its_last_owner_lets_go_gives_the_occurrence_back() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let replaced = key_with_identity("upstream", 5, 1, 90);
    let refresh = key_with_identity("upstream", 6, 1, 91);
    let mut replaced_record = DemandRecord {
        framework_required_count: 1,
        ..record(occurrence(), ready(), 0)
    };
    let DemandState::Output(output) = &mut replaced_record.state else {
        unreachable!("the replaced row holds a Ready");
    };
    output.finish_stopped(WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::Superseded,
        "",
    ));
    let refresh_record = DemandRecord {
        successor_of: Some(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::succession::Succession::new([0; 32])),
        ..record(occurrence(), DemandState::Scheduled, 1)
    };
    let wake = Arc::clone(&refresh_record.wake);
    {
        let mut state = registry.state.lock().unwrap();
        state.records.insert(replaced.clone(), replaced_record);
        state.records.insert(refresh.clone(), refresh_record);
    }
    drop(interest(&registry, refresh.clone(), wake));
    let state = registry.state.lock().unwrap();
    assert!(!state.records.contains_key(&refresh));
    // The Ready the refresh replaced answers for the occurrence again.
    assert!(
        matches!(&state.records[&replaced].state, DemandState::Output(output)
        if matches!(output.advancement(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputAdvancement::Idle))
    );
}
#[test]
fn a_closing_dependent_moves_its_claim_once_the_newest_row_is_ready() {
    let (registry, interest, old, newest) = claim_on_superseded(ready());
    let dependent = interest.key.clone();
    drop(interest);
    registry.drain_closed_cleanup_admitted().unwrap();
    let state = registry.state.lock().unwrap();
    assert!(!state.records.contains_key(&old));
    assert_eq!(state.records[&newest].framework_required_count, 1);
    assert_eq!(claims(&state.records[&dependent]), vec![newest]);
}
