//! A hundred cycles certify queued pressure, then reopening and retry.
use super::*;
use support::capacity_region::{settle as setup_settle, start as setup_start, Attempt};
pub(super) fn run(custody_budget: usize, tight: bool) -> Attempt {
    let retained_positions = 8;
    let (application, invalidation) =
        limited_application(custody_budget, 128 * 1024 * 1024, retained_positions);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, c) = setup_chain!(application, request);
    let mut d = setup_start!(
        request
            .demand(PlanarOutputDemand::new("anchor-source-b"))
            .start_in_program::<program::ChainProgram, program::ChainRoot>(&application),
        "register unrelated"
    );
    setup_settle!(d, request, "initial unrelated");
    let roots = source_roots(&request);
    let settled_custody = application.required_custody_bytes_for_test();
    let mut steady = None;
    let mut identity_steady = None;
    let mut queries = None;
    for cycle in 0..100_u64 {
        change_root_input!(request, application, 2 + cycle % 2, 0x9176_3d00_u64 + cycle);
        let before = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
        let entries = query_entries();
        let advanced = d.advance(&request);
        if cycle == 0
            && matches!(&advanced, Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                if denial.kind() == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded)
        {
            return Attempt::Below("unrelated cannot advance the queued chain");
        }
        assert!(
            matches!(
                advanced.unwrap(),
                WorthQueryApplicationOutputDemandProgress::Settled(_)
            ),
            "the unrelated demand settles in one advance"
        );
        let ran = query_entries() - entries;
        assert_eq!(
            *queries.get_or_insert(ran),
            ran,
            "cycle {cycle}: untouched output is not fully verified"
        );
        // D alone advances; three-row custody stops queued B/C refreshes.
        // Only A reads then. With five rows the whole dirty chain reads once.
        if cycle == 0 && tight {
            let after = primary_graph::query_read_kernel_entries_by_root_on_this_thread_for_test();
            let reads = roots.map(|root| {
                after.get(&root).copied().unwrap_or(0) - before.get(&root).copied().unwrap_or(0)
            });
            if reads == [0, 0, 0, 0] {
                return Attempt::Below("queued root source admission");
            }
            if reads[1] != 0 || reads[2] != 0 {
                return Attempt::Above("queued refreshes fit");
            }
        }
        assert_root_reads(
            roots,
            before,
            cycle,
            if tight { [1, 0, 0, 0] } else { [1, 1, 1, 0] },
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
        "{custody_budget} bytes: closing the chain holds no more than it settled with"
    );
    let mut b = request
        .demand(ChainDemand("anchor-b".to_owned()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    if tight {
        let stopped = b.advance(&request);
        if stopped.is_ok() {
            return Attempt::Above("reopened middle was admitted instead of refused");
        }
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
        // The subject is a retry that settles at once. A budget whose retry
        // stops again is tighter than the subject.
        match b.advance(&request) {
            Ok(WorthQueryApplicationOutputDemandProgress::Settled(_)) => (),
            Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
                if denial.kind() == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded =>
            {
                return Attempt::Below("the reopened middle's retry stops again")
            }
            other => panic!("the retry settles in one advance: {:?}", other.err()),
        }
    }
    settled_in_one_advance!(b, request, "the reopened middle consumer");
    drop(b);
    assert!(
        application.required_custody_bytes_for_test() <= settled_custody,
        "{custody_budget} bytes: the unrelated caller retains no refreshed chain row"
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
        "{custody_budget} bytes: custody returns within the settled chain's"
    );
    Attempt::Hit
}
