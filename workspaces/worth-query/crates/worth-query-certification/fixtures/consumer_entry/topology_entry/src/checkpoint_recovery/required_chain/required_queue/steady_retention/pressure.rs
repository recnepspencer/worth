//! The complete 100-cycle workload under ample calibration or deliberate pressure.
use super::*;
use support::custody_calibration::{Inventory, Readings};
pub(super) fn run(
    rows_per_demand: usize,
    custody_budget: usize,
    mut readings: Option<&mut Readings>,
) {
    let retained_positions = 8;
    let (application, invalidation) =
        limited_application(custody_budget, 128 * 1_024 * 1_024, retained_positions);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, c, mut d) = chain_with_unrelated!(
        application,
        request,
        {
            if let Some(readings) = &mut readings {
                readings.record(
                    "before_rows",
                    Inventory::new(
                        application.required_custody_breakdown_for_test(),
                        invalidation.retained_custody_breakdown_for_test(),
                        application.output_lineage_retained_bytes_for_test(),
                    ),
                );
            }
        },
        {
            if let Some(readings) = &mut readings {
                readings.record(
                    "settled_rows",
                    Inventory::new(
                        application.required_custody_breakdown_for_test(),
                        invalidation.retained_custody_breakdown_for_test(),
                        application.output_lineage_retained_bytes_for_test(),
                    ),
                );
            }
        }
    );
    let roots = source_roots(&request);
    let settled_custody = application.required_custody_bytes_for_test();
    let mut steady = None;
    let mut identity_steady = None;
    let mut queries = None;
    for cycle in 0..100_u64 {
        change_root_input!(request, application, 2 + cycle % 2, 0x9176_3d00_u64 + cycle);
        let before = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
        unrelated_settles_unverified!(queries, cycle, d, request);
        // D alone advances; three-row custody stops queued B/C refreshes.
        // Only A reads then. With five rows the whole dirty chain reads once.
        assert_root_reads(
            roots,
            before,
            cycle,
            if rows_per_demand == 3 && readings.is_none() {
                [1, 0, 0, 0]
            } else {
                [1, 1, 1, 0]
            },
        );
        if cycle >= 2 * retained_positions as u64 {
            // The held A key and the held B/C/D keys stay fixed. A's
            // newly refreshed source epoch changes with each input edit.
            let named = (
                application
                    .registry_row_keys_for_test()
                    .into_iter()
                    .filter(|(_, root, generation, _)| *root != roots[0] || *generation == 0)
                    .collect::<Vec<_>>(),
                invalidation
                    .native_retained_allocations_for_test()
                    .map(|(count, _)| count),
            );
            assert_eq!(identity_steady.get_or_insert(named.clone()), &named,
                    "cycle {cycle}: held original chain row keys and hint/branch/completion counts are steady");
        }
        assert_steady!(steady, cycle, retained_positions, application, invalidation);
    }
    // A refreshed row belongs to its own demand, never to the caller
    // that happened to run it, so closing the chain leaves no more custody
    // than the settled chain held. The middle consumer then reopens alone
    // over its stale, undemanded root. At five rows it refreshes that
    // root and settles in one advance. At three, custody cannot keep the
    // middle refresh at once: the advance stops retryable and ends the
    // refreshes it carried, which leaves less custody held than before
    // it, so the retry settles beside the unrelated caller's open Ready.
    drop((a, b, c));
    let closed_custody = application.required_custody_bytes_for_test();
    assert!(
        closed_custody <= settled_custody,
        "{rows_per_demand} rows per demand: closing the chain holds no more than it settled with"
    );
    let mut b = request
        .demand(ChainDemand("anchor-b".to_owned()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    if rows_per_demand == 3 && readings.is_none() {
        let stopped = b.advance(&request);
        assert!(
            matches!(
                &stopped,
                Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                    if denial.kind() == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
                        && denial.recovery_posture()
                            == primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable
            ),
            "a refresh custody cannot keep at once stops retryable: {:?}",
            stopped.as_ref().err()
        );
        assert!(
            application.required_custody_bytes_for_test() < closed_custody,
            "the stop holds less than the advance began with"
        );
    }
    settled_in_one_advance!(b, request, "the reopened middle consumer");
    drop(b);
    assert!(
        application.required_custody_bytes_for_test() <= settled_custody,
        "{rows_per_demand} rows per demand: the unrelated caller retains no refreshed chain row"
    );
    let closed_ready_custody = application.required_custody_bytes_for_test();
    assert!(
        closed_ready_custody <= custody_budget,
        "closed Ready custody fits its declared budget"
    );
    let current_commit = request.retain_read().unwrap().selected_commit().clone();
    // Unchanged cached rows retain their custody plateau without publication.
    for reopen in 0..8 {
        let mut b = request
            .demand(ChainDemand("anchor-b".to_owned()))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                &application,
            )
            .unwrap();
        settled_in_one_advance!(b, request, "the unchanged reopened middle consumer");
        drop(b);
        assert_eq!(
            application.required_custody_bytes_for_test(),
            closed_ready_custody,
            "reopen {reopen}: closed Ready custody stays on its actual plateau"
        );
        assert_eq!(
            request.retain_read().unwrap().selected_commit(),
            &current_commit,
            "reopen {reopen}: unchanged output is not republished"
        );
    }
    settled_in_one_advance!(d, request, "the unrelated required demand");
    drop(d);
    assert!(
        application.required_custody_bytes_for_test() <= closed_ready_custody,
        "closing the unrelated caller adds no custody"
    );
    assert!(
        application.required_custody_bytes_for_test() <= settled_custody,
        "{rows_per_demand} rows per demand: custody returns within the settled chain's"
    );
}
