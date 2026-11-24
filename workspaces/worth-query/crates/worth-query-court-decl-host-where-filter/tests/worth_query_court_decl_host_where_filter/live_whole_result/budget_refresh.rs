//! Independent tight-budget refresh court binary-searching both minimal work
//! AND minimal memory budgets on `next_in_advancement` after an invalidating
//! mutation (reaching footprint capture), plus verifying immediate source
//! custody detachment when `ExecutionRequest` terminates the lease.
use super::queries::ReadyIntent;
use super::world::{install, live_limits, run_mutate, FUTURE_DUE_DAYS, TODAY_CUTOFF_DAYS};
use super::{mutate_op, seed};
use worth_execution::{ExecutionAllocationPolicy, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjectionDenialKind, WorthQueryApplicationWholeResultLiveOutcome,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

#[test]
fn refresh_tight_work_and_memory_budgets_cover_footprint_capture_and_detach() {
    let probe_refresh = |mem: u64, work: u64| -> (bool, bool, usize) {
        let host = install(
            &[
                seed(1, "p1", "open", Some(FUTURE_DUE_DAYS)),
                seed(2, "p1", "open", Some(TODAY_CUTOFF_DAYS)),
                seed(3, "p1", "open", Some(FUTURE_DUE_DAYS)),
                seed(4, "p1", "open", Some(TODAY_CUTOFF_DAYS)),
            ],
            &[(1, 2), (3, 2), (3, 4)],
        );
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let mut sub = host
            .runtime()
            .request(&principal, &scope)
            .query(ReadyIntent {
                project: "p1".into(),
            })
            .subscribe_whole_result(live_limits())
            .expect("sub opens");
        run_mutate(&host, mutate_op("p1", 0, 2, 0, 0, "done"), 1);

        let lease = ExecutionResourceLease::serial(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Serial,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(mem), Some(work)),
            ),
            cancellation: scope.cancellation().execution_token(),
            deadline: scope.deadline(),
        })
        .expect("lease admits");
        let outcome = host.runtime().with_funded_application_advancement(
            &scope,
            ExecutionAllocationPolicy::Execution(&lease),
            |phase| sub.next_in_advancement(&phase, &host.runtime().request(&principal, &scope)),
        );
        let (delivered, exact_exec_stop) = match outcome {
            Ok(Ok(WorthQueryApplicationWholeResultLiveOutcome::Delivered(u))) => {
                assert_eq!(
                    u.snapshot()
                        .rows()
                        .iter()
                        .map(|r| r.key)
                        .collect::<Vec<_>>(),
                    vec![1, 4]
                );
                (true, false)
            }
            Ok(Ok(WorthQueryApplicationWholeResultLiveOutcome::ExecutionRequest(_))) => {
                (false, true)
            }
            Ok(Ok(WorthQueryApplicationWholeResultLiveOutcome::ProjectionDenied(
                WorthQueryApplicationProjectionDenialKind::ValueDecode(_),
            )))
            | Err(_) => (false, false),
            other => panic!("unexpected refresh budget outcome: {other:?}"),
        };
        let remaining = host.runtime().close_live_delivery().remaining_live_consumers();
        drop(sub);
        (delivered, exact_exec_stop, remaining)
    };

    let (ok, _, active_rem) = probe_refresh(16 * 1024 * 1024, 200_000);
    assert!(ok, "generous budget must deliver refreshed whole result");
    assert_eq!(active_rem, 1, "active delivered lease stays subscribed");

    // Binary-search exact minimal refresh work budget.
    let (mut lo_w, mut hi_w) = (1_u64, 200_000_u64);
    while lo_w < hi_w {
        let mid = lo_w + (hi_w - lo_w) / 2;
        if probe_refresh(16 * 1024 * 1024, mid).0 {
            hi_w = mid;
        } else {
            lo_w = mid + 1;
        }
    }
    assert!(lo_w > 1, "refresh + footprint capture must charge work");
    let (ok_w_below, exec_w_below, rem_w_below) = probe_refresh(16 * 1024 * 1024, lo_w - 1);
    assert!(!ok_w_below, "min_refresh_work - 1 must refuse");
    assert!(
        exec_w_below,
        "exact min_refresh_work - 1 boundary must fail in ExecutionRequest (footprint capture / post-refresh)"
    );
    assert_eq!(
        rem_w_below, 0,
        "work-exhausted refresh handle must detach source custody before Drop"
    );

    // Binary-search exact minimal refresh memory budget.
    let (mut lo_m, mut hi_m) = (16_u64, 16 * 1024 * 1024_u64);
    while lo_m < hi_m {
        let mid = lo_m + (hi_m - lo_m) / 2;
        if probe_refresh(mid, 200_000).0 {
            hi_m = mid;
        } else {
            lo_m = mid + 1;
        }
    }
    assert!(lo_m > 16, "refresh + footprint capture must charge memory");
    let (ok_m_below, exec_m_below, rem_m_below) = probe_refresh(lo_m - 1, 200_000);
    assert!(!ok_m_below, "min_refresh_mem - 1 must refuse");
    assert!(
        exec_m_below,
        "exact min_refresh_mem - 1 boundary must fail in ExecutionRequest (footprint capture)"
    );
    assert_eq!(
        rem_m_below, 0,
        "memory-exhausted refresh handle must detach source custody before Drop"
    );
}
