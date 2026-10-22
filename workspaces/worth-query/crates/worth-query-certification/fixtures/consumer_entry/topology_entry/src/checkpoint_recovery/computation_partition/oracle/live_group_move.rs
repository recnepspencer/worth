//! Equal-result performed moves must install the new reader attribution.
use super::differential::{
    alphabet::{Change, Model},
    reference::Reference,
};
use super::*;
use worth_query_host::facade::application_contribution::WorthQueryPartitionedComputationRunForTest as Run;
use worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test as verification;
#[test]
fn equal_total_live_group_moves_publish_new_attribution_without_unrelated_preparation() {
    moves([3, 1]);
}
#[test]
fn new_and_last_group_moves_publish_checked_shape_and_latest_readers_without_unrelated_work() {
    moves([5, 1]);
}
fn moves(destinations: [u32; 2]) {
    let _guard = checkpoint_recovery_test_guard();
    let mut receipts = Vec::new();
    for unrelated in [1, 10, 100] {
        let mut model = Model::live_group_move(unrelated);
        let mut reference = Reference::new(&model);
        let app = install_with_reuse::<true>(|graph| model.seed(graph));
        let (scope, principal) = authenticate(&app);
        let request = app.request(&principal, &scope);
        let (_, initial) = performed(&request, &app);
        let mut total = initial[0].outcome.as_ref().unwrap().0;
        reference.demanded(None);
        let mut command = 0x29_0000;
        for destination in destinations {
            let change = model.move_first_to(destination);
            reference.edit(&change);
            command += 1;
            let Change::Entry(change) = change else {
                unreachable!()
            };
            edit(&request, &app, change, command);
            verification();
            let (contacts, moved) = performed(&request, &app);
            let move_work = verification();
            assert_eq!(contacts, 1, "a moved key actually runs the producer");
            assert_eq!(moved.len(), 1);
            assert_eq!(
                moved[0].calls,
                OwnerCalls {
                    plans: 0,
                    keys: 1,
                    gathers: if destinations[0] == 5 && destination == 1 {
                        1
                    } else {
                        2
                    },
                    kernels: if destinations[0] == 5 && destination == 1 {
                        1
                    } else {
                        2
                    }
                }
            );
            assert_eq!(moved[0].runs, [Run::Incremental]);
            assert_eq!(
                moved[0].outcome.as_ref().unwrap().0,
                total,
                "the complete aggregate is equal after regrouping"
            );
            assert!(
                !moved[0].published.is_empty(),
                "equal aggregate publishes fresh computation custody"
            );
            let fresh = differential::full_run(&model, &reference, None);
            assert_eq!(moved[0].outcome, fresh[0].outcome);
            assert_published_state(&moved, &fresh);
            reference.demanded(None);
            let value = if destination != 1 { 2.5 } else { 2.0 };
            let change = model.set_first_value(value);
            reference.edit(&change);
            command += 1;
            let Change::Entry(change) = change else {
                unreachable!()
            };
            edit(&request, &app, change, command);
            verification();
            let (_, changed) = performed(&request, &app);
            let value_work = verification();
            assert_eq!(
                changed[0].calls,
                OwnerCalls {
                    plans: 0,
                    keys: 0,
                    gathers: 1,
                    kernels: 1
                },
                "only the new destination reads this value"
            );
            let fresh = differential::full_run(&model, &reference, None);
            assert_eq!(changed[0].outcome, fresh[0].outcome);
            assert_published_state(&changed, &fresh);
            reference.demanded(None);
            total = changed[0].outcome.as_ref().unwrap().0;
            receipts.push((unrelated, destination, move_work, value_work));
        }
    }
    for (population, destination, moved, value) in &receipts {
        let first = receipts
            .iter()
            .find(|(_, target, _, _)| target == destination)
            .unwrap()
            .2;
        assert_eq!(value.completion_call_roster_visits, 0);
        assert_eq!(value.sealed_computation_index_visits, 0);
        assert_eq!(moved.completion_call_roster_visits, 0);
        assert_eq!(moved.sealed_computation_index_visits, 0);
        assert_eq!(moved.fallback_carried_metadata_visits, 0);
        assert_eq!(moved.computation_call_build_visits, 0);
        assert_eq!(moved.computation_tree_plan_visits, 0);
        assert_eq!(
            moved.computation_prepared_partition_visits,
            first.computation_prepared_partition_visits,
            "{population}: only affected surviving groups enter preparation"
        );
        assert_eq!(
            moved.completion_fresh_key_visits, first.completion_fresh_key_visits,
            "{population}: exact reader-set patch follows affected gathers"
        );
        assert_eq!(
            moved.posting_fact_visits, first.posting_fact_visits,
            "{population}: publication projection remains sparse"
        );
        assert_eq!(
            moved.posting_registration_visits, first.posting_registration_visits,
            "{population}: installation and predecessor retirement remain sparse"
        );
    }
    eprintln!("group{destinations:?} equal-result publication and destination-only value edits at1/10/100: {receipts:?}");
}

fn performed(request: &Request<'_, '_, '_>, app: &Application<true>) -> (usize, Vec<OracleRun>) {
    use worth_query_host::facade::application_entry::WorthQueryOutputSettlementPosture;
    room().clear();
    published_states();
    let mut demand = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<OracleProgram<true>, RegionConnection>(app)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.advance(request).unwrap()
    else {
        panic!("one advance settles the performed move")
    };
    assert_eq!(
        settled.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    let mut runs = take_runs(None);
    let published = published_states();
    runs.last_mut()
        .expect("performed computation runs")
        .published = published;
    (settled.producer_contacts_in_this_demand(), runs)
}
