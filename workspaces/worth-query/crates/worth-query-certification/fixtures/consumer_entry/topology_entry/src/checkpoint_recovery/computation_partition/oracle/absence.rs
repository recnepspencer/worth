//! Checkpoint restore rebuilds genuinely absent computation state.

use super::super::entry_edit::EntryFact;
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as Cause,
    WorthQueryPartitionedComputationRunForTest as Run,
};
use worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile;

fn seed(graph: &mut Graph) {
    facts::seed_set(graph, "even", -0.0);
    for number in 0..LARGEST_SET {
        seed_entry(
            graph,
            &["even"],
            number,
            RegionEntry {
                id: number as u64,
                region: number as u32,
                value: 1.0,
                work: 1,
                fault: None,
            },
        );
    }
}

#[test]
fn checkpoint_absence_rebuilds_every_owner_call_like_fresh() {
    let _guard = checkpoint_recovery_test_guard();
    let profile = WorthQueryOutputDemandResourceProfile::standard();
    let application =
        installation::install_configured::<false, TOTALS_WORK, 1>(None, profile, seed);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (_, first) = demand(&request, &application);
    assert_eq!(first[0].runs, [Run::Full(Cause::FirstRun)]);
    let checkpoint = application.with_host_advancement(|host_phase| application.capture_application_checkpoint(&host_phase, worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy::SystemAllocation).unwrap()).expect("installed host call opens");
    drop(application);
    let application =
        installation::install_configured::<false, TOTALS_WORK, 1>(Some(checkpoint), profile, seed);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    edit(
        &request,
        &application,
        at_demand_scope(EntryEdit::new(
            "even",
            7,
            EntryFact::Value,
            2.5_f64.to_bits(),
        )),
        9901,
    );
    let (contacts, runs) = demand(&request, &application);
    assert_eq!(contacts, 1, "restore runs the producer");
    assert_eq!(runs.len(), 1);
    let expected = Cause::Restored;
    assert_eq!(runs[0].runs, [Run::Full(expected)], "restore");
    assert_eq!(
        runs[0].calls,
        OwnerCalls {
            plans: 1,
            keys: LARGEST_SET,
            gathers: LARGEST_SET,
            kernels: LARGEST_SET,
        },
        "restore: printing a Full cause cannot conceal old calls"
    );
    let fresh = installation::install_configured::<false, TOTALS_WORK, 1>(None, profile, seed);
    let (fresh_scope, fresh_principal) = authenticate(&fresh);
    let fresh_request = fresh.request(&fresh_principal, &fresh_scope);
    edit(
        &fresh_request,
        &fresh,
        at_demand_scope(EntryEdit::new(
            "even",
            7,
            EntryFact::Value,
            2.5_f64.to_bits(),
        )),
        9901,
    );
    let (_, reference) = demand(&fresh_request, &fresh);
    assert_eq!(
        runs[0].outcome, reference[0].outcome,
        "restore: outcome and charged work"
    );
    assert_eq!(runs[0].calls, reference[0].calls, "restore: owner calls");
    assert_published_state(&runs, &reference);
}
