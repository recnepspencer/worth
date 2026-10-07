//! Invocation and observation fallbacks against freshly installed producers.

use super::super::entry_edit::EntryFact;
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as Cause, WorthQueryPartitionedComputationRun as Run,
};

fn seed(graph: &mut Graph) {
    facts::seed_set(graph, "even", -0.0);
    seed_entry(
        graph,
        &["even"],
        0,
        RegionEntry {
            id: 0,
            region: 0,
            value: 1.0,
            work: 1,
            fault: None,
        },
    );
}

fn changed<const WORK: usize, const RUNS: usize>(
    application: &Application<false, WORK, RUNS>,
) -> Vec<OracleRun> {
    let (scope, principal) = authenticate(application);
    let request = application.request(&principal, &scope);
    edit(
        &request,
        application,
        at_demand_scope(EntryEdit::new(
            "even",
            0,
            EntryFact::Value,
            2.5_f64.to_bits(),
        )),
        9902,
    );
    demand(&request, application).1
}

fn differential<const WORK: usize, const RUNS: usize>(expected: &[Cause]) {
    let application =
        installation::install_configured::<false, WORK, RUNS>(None, Default::default(), seed);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (_, initial) = demand(&request, &application);
    assert_eq!(initial.len(), RUNS);
    assert_eq!(initial[0].runs, [Run::Full(Cause::FirstRun)]);
    if RUNS == 2 {
        assert_eq!(initial[1].runs, [Run::Full(Cause::NoPriorHanded)]);
    }
    let retained = changed(&application);
    let fresh =
        installation::install_configured::<false, WORK, RUNS>(None, Default::default(), seed);
    let reference = changed(&fresh);
    assert_eq!(retained.len(), RUNS);
    assert_eq!(reference.len(), RUNS);
    assert_published_state(&retained, &reference);
    for (number, ((kept, full), cause)) in retained.iter().zip(&reference).zip(expected).enumerate()
    {
        assert_eq!(kept.runs, [Run::Full(*cause)], "invocation {number}");
        assert_eq!(
            kept.calls,
            OwnerCalls {
                plans: 1,
                keys: 1,
                gathers: 1,
                kernels: 1
            }
        );
        assert_eq!(
            full.runs,
            [Run::Full(if number == 0 {
                Cause::FirstRun
            } else {
                Cause::NoPriorHanded
            })]
        );
        assert_eq!(kept.calls, full.calls);
        assert_eq!(
            kept.outcome, full.outcome,
            "outcome, charged work and work boundary"
        );
    }
}

#[test]
fn an_observation_bound_above_declared_work_falls_back_without_eviction() {
    let _guard = checkpoint_recovery_test_guard();
    // Incoming membership examines 160 edges and endpoints, so its sealed
    // observation bound plus each set's outgoing membership exceeds 1,024 units. One item's encoding and kernel fit.
    let seed = |graph: &mut Graph| {
        seed(graph);
        for number in 0..LARGEST_SET {
            let name = format!("incoming-{number}");
            facts::seed_set(graph, &name, -0.0);
            facts::seed_member(graph, &name, "oracle-entry-0");
        }
    };
    let application =
        installation::install_variant::<false, 1_024, 1, 1>(None, Default::default(), seed);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (_, first) = demand(&request, &application);
    assert_eq!(first[0].runs, [Run::Full(Cause::FirstRun)], "{first:?}");
    edit(
        &request,
        &application,
        at_demand_scope(EntryEdit::new(
            "even",
            0,
            EntryFact::Value,
            2.5_f64.to_bits(),
        )),
        9902,
    );
    let (_, kept) = demand(&request, &application);
    assert_eq!(kept[0].runs, [Run::Full(Cause::ObservationOverBudget)]);
    let fresh = installation::install_variant::<false, 1_024, 1, 1>(None, Default::default(), seed);
    let (fresh_scope, fresh_principal) = authenticate(&fresh);
    let fresh_request = fresh.request(&fresh_principal, &fresh_scope);
    edit(
        &fresh_request,
        &fresh,
        at_demand_scope(EntryEdit::new(
            "even",
            0,
            EntryFact::Value,
            2.5_f64.to_bits(),
        )),
        9902,
    );
    let (_, reference) = demand(&fresh_request, &fresh);
    assert_eq!(
        kept[0].calls,
        OwnerCalls {
            plans: 1,
            keys: 1,
            gathers: 1,
            kernels: 1
        }
    );
    assert_eq!(kept[0].calls, reference[0].calls);
    assert_eq!(kept[0].outcome, reference[0].outcome);
    assert_published_state(&kept, &reference);
}

#[test]
fn several_invocations_receive_prior_once_and_publish_several_computations() {
    let _guard = checkpoint_recovery_test_guard();
    differential::<TOTALS_WORK, 2>(&[Cause::SeveralComputations, Cause::NoPriorHanded]);
}
