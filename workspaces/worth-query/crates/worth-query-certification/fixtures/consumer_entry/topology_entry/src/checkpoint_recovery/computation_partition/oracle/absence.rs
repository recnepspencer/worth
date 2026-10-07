//! Prior-absence steps run at the real ledger and checkpoint boundaries.

use super::super::entry_edit::EntryFact;
use super::*;
use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as Cause, WorthQueryPartitionedComputationRun as Run,
};
use worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile;

#[derive(Clone, Copy, Debug)]
enum Step {
    Evict,
    Restore,
}

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
fn absence_steps_run_every_owner_call_and_equal_fresh_outcome_and_work() {
    let _guard = checkpoint_recovery_test_guard();
    for step in [Step::Evict, Step::Restore] {
        // The fixed 256 KiB ledger admits the chained output addresses but cannot hold the 160
        // entries' eight retained fact rows each, even before their payload.
        let profile = match step {
            Step::Evict => WorthQueryOutputDemandResourceProfile::standard()
                .with_lineage_retained_bytes(NonZeroUsize::new(256 * 1024).unwrap()),
            Step::Restore => Default::default(),
        };
        let application =
            installation::install_configured::<false, TOTALS_WORK, 1>(None, profile, seed);
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let (_, first) = demand(&request, &application);
        assert_eq!(first[0].runs, [Run::Full(Cause::FirstRun)]);
        let application = match step {
            Step::Evict => application,
            Step::Restore => {
                let checkpoint = application.capture_application_checkpoint().unwrap();
                drop(application);
                installation::install_configured::<false, TOTALS_WORK, 1>(
                    Some(checkpoint),
                    profile,
                    seed,
                )
            }
        };
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
        assert_eq!(contacts, 1, "{step:?} runs the producer");
        assert_eq!(runs.len(), 1);
        let expected = match step {
            Step::Evict => Cause::Evicted,
            Step::Restore => Cause::Restored,
        };
        assert_eq!(runs[0].runs, [Run::Full(expected)], "{step:?}");
        assert_eq!(
            runs[0].calls,
            OwnerCalls {
                plans: 1,
                keys: LARGEST_SET,
                gathers: LARGEST_SET,
                kernels: LARGEST_SET,
            },
            "{step:?}: printing a Full cause cannot conceal old calls"
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
            "{step:?}: outcome and charged work"
        );
        assert_eq!(runs[0].calls, reference[0].calls, "{step:?}: owner calls");
        assert_published_state(&runs, &reference);
    }
}
