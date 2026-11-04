//! Drop keeps the old claim until an admission pays for its transfer.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;

#[test]
fn registry_drop_cascade_transfers_before_newest_owner_can_close() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let old = key_with_identity("upstream", 1, 1, 10);
    let newest = key_with_identity("upstream", 2, 1, 11);
    let dependent = key("dependent", 1, 2);
    let ready = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready();
    let mut state = (*registry.state_for_test()).lock().unwrap();
    (*state.records_for_test()).insert_for_test(
        old.clone(),
        DemandRecord {
            framework_required_count: 1,
            ..record(
                occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "newer row",
                )),
                0,
            )
        },
    );
    let next_row = record(occurrence(), worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready(), 1);
    let next_wake = Arc::clone(&(*next_row.wake_for_test()));
    (*state.records_for_test()).insert_for_test(newest.clone(), next_row);
    let dependent_row = DemandRecord {
        prerequisites: vec![Arc::new(old.clone())],
        ..record(occurrence(), ready, 1)
    };
    let wake = Arc::clone(&(*dependent_row.wake_for_test()));
    (*state.records_for_test()).insert_for_test(dependent.clone(), dependent_row);
    (*state.required_keys_mut_for_test()).insert(Arc::new(newest.clone()));
    retain_owned_key(&mut state, &dependent);
    retain_owned_key(&mut state, &newest);
    drop(state);
    let next_owner = interest(&registry, newest.clone(), next_wake);
    drop(interest(&registry, dependent.clone(), wake));
    // The stale claim keeps the newest member through both Drops.
    drop(next_owner);
    let state = (*registry.state_for_test()).lock().unwrap();
    assert!(
        (*state.records_for_test()).contains_key_for_test(&old),
        "Drop must leave the claim transfer unpaid"
    );
    assert_eq!((*(*state.records_for_test())[&dependent].prerequisites_for_test())[0].as_ref(), &old);
    assert!((*(*state.records_for_test())[&dependent].closed_cleanup_for_test()).queued_for_test());
    drop(state);
    registry.drain_closed_cleanup_admitted().unwrap();
    let state = (*registry.state_for_test()).lock().unwrap();
    assert!(!(*state.records_for_test()).contains_key_for_test(&old));
    assert_eq!((*(*state.records_for_test())[&newest].framework_required_count_for_test()), 1);
    assert!((*state.required_keys_for_test()).contains(&newest));
    assert_eq!((*(*state.records_for_test())[&dependent].prerequisites_for_test())[0].as_ref(), &newest);
}

#[test]
fn registry_drop_cascade_names_its_released_row_bound() {
    let receipts = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::receipt_batch();
    let mut contacts = Vec::new();
    for length in [1, 2, 4] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let mut state = (*registry.state_for_test()).lock().unwrap();
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::add_live_custody(&mut state, &receipts, 3);
        state.note_custody_changes_for_test(|_, entry| {
            (*entry.token_count_mut_for_test()) = 1;
            false
        });
        for slot in 1..=length {
            let mut row = record(
                occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                    WorthQueryOutputDemandDenialKind::NoEffect,
                    "closed chain",
                )),
                usize::from(slot == 1),
            );
            (*row.framework_required_count_mut_for_test()) = usize::from(slot != 1);
            if slot == 1 {
                (*row.prerequisites_mut_for_test())
                    .extend((2..=length).map(|upstream| Arc::new(key("chain", 1, upstream))));
            }
            (*state.records_for_test()).insert_for_test(key("chain", 1, slot), row);
        }
        let root = key("chain", 1, 1);
        retain_owned_key(&mut state, &root);
        let wake = Arc::clone(&(*(*state.records_for_test())[&root].wake_for_test()));
        drop(state);
        let before = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited();
        drop(interest(&registry, root, wake));
        contacts.push(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited() - before);
        registry.drain_closed_cleanup_admitted().unwrap();
        assert_eq!(
            (*(*registry.state_for_test()).lock().unwrap().records_for_test()).len_for_test(),
            receipts.len() * 3
        );
    }
    // Drop only searches its own one-row occurrence, regardless of its edges.
    assert_eq!(contacts, [1, 1, 1]);
}

#[test]
fn registry_drop_cascade_names_its_whole_map_claimant_bound() {
    let receipts = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::receipt_batch();
    let mut contacts = Vec::new();
    for neighbors in [1, 64] {
        let registry = WorthQueryOutputDemandRegistry::default();
        let old = key_with_identity("upstream", 1, 1, 10);
        let newest = key_with_identity("upstream", 2, 1, 11);
        let dependent = key("dependent", 1, 2);
        let mut state = (*registry.state_for_test()).lock().unwrap();
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::walk_bounds::add_custody(&mut state, &receipts, 1);
        for slot in 0..neighbors {
            (*state.records_for_test()).insert_for_test(
                key("neighbor", 1, 20 + slot),
                record(occurrence(), DemandState::Admitted, 1),
            );
        }
        (*state.records_for_test()).insert_for_test(
            old.clone(),
            DemandRecord {
                framework_required_count: 1,
                ..record(
                    occurrence(),
                    DemandState::Failed(WorthQueryOutputDemandDenial::new_for_test(
                        WorthQueryOutputDemandDenialKind::Superseded,
                        "newer",
                    )),
                    0,
                )
            },
        );
        (*state.records_for_test()).insert_for_test(
            newest.clone(),
            record(occurrence(), worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready(), 1),
        );
        (*state.required_keys_mut_for_test()).insert(Arc::new(newest));
        let row = DemandRecord {
            prerequisites: vec![Arc::new(old)],
            ..record(occurrence(), worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::closed_retirement::ready(), 1)
        };
        let wake = Arc::clone(&(*row.wake_for_test()));
        (*state.records_for_test()).insert_for_test(dependent.clone(), row);
        retain_owned_key(&mut state, &dependent);
        drop(state);
        let whole = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::whole_rows().expect("bounded registry observation");
        let occurrence = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited();
        drop(interest(&registry, dependent, wake));
        contacts.push((
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::whole_rows().expect("bounded registry observation") - whole,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::record_map::occurrence_rows_visited() - occurrence,
        ));
    }
    // One replaced-row scan and one member scan of the dependent's own row;
    // linking its cue makes no claimant pass or upstream walk.
    assert_eq!(contacts, [(0, 2), (0, 2)]);
}
