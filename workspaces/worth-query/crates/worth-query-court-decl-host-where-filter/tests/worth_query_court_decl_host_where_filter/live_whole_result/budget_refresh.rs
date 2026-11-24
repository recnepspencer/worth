//! Stage-pinned tight-budget refresh court binary-searching both minimal work
//! AND minimal memory budgets on `next_in_advancement_with_stage_hook_for_test`
//! after an invalidating mutation. Proves that exact `min_work - 1` and
//! `min_mem - 1` fail specifically INSIDE `WholeResultInvalidationFootprint::capture`
//! (`reached_capture && !completed_capture`), while `min` completes capture and
//! delivers cleanly, and refused handles immediately detach source custody.
use super::queries::ReadyIntent;
use super::world::{install, live_limits, run_mutate, FUTURE_DUE_DAYS, TODAY_CUTOFF_DAYS};
use super::{mutate_op, seed};
use worth_execution::{ExecutionAllocationPolicy, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_host::facade::primary_graph::{
    WholeResultLiveStageForTest, WorthQueryApplicationProjectionDenialKind,
    WorthQueryApplicationWholeResultLiveOutcome,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

struct RefreshProbe {
    delivered: bool,
    exact_exec_stop: bool,
    reached_capture: bool,
    completed_capture: bool,
    remaining_consumers: usize,
}

#[test]
fn refresh_tight_work_and_memory_budgets_pin_footprint_capture_and_detach() {
    let probe_refresh = |mem: u64, work: u64| -> RefreshProbe {
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
        let mut reached_capture = false;
        let mut completed_capture = false;
        let outcome = host.runtime().with_funded_application_advancement(
            &scope,
            ExecutionAllocationPolicy::Execution(&lease),
            |phase| {
                let mut hook = |stage| match stage {
                    WholeResultLiveStageForTest::BeforeFootprintCapture => reached_capture = true,
                    WholeResultLiveStageForTest::AfterFootprintCaptureBeforeDeliveryConfirm => {
                        completed_capture = true;
                    }
                    _ => {}
                };
                sub.next_in_advancement_with_stage_hook_for_test(
                    &phase,
                    &host.runtime().request(&principal, &scope),
                    &mut hook,
                )
            },
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
        let remaining_consumers = host
            .runtime()
            .close_live_delivery()
            .remaining_live_consumers();
        drop(sub);
        RefreshProbe {
            delivered,
            exact_exec_stop,
            reached_capture,
            completed_capture,
            remaining_consumers,
        }
    };

    let generous = probe_refresh(16 * 1024 * 1024, 200_000);
    assert!(
        generous.delivered && generous.reached_capture && generous.completed_capture,
        "generous budget must reach & complete footprint capture and deliver"
    );
    assert_eq!(
        generous.remaining_consumers, 1,
        "active delivered lease stays subscribed"
    );

    // Binary-search exact minimal refresh work budget and pin failure to footprint capture.
    let (mut lo_w, mut hi_w) = (1_u64, 200_000_u64);
    while lo_w < hi_w {
        let mid = lo_w + (hi_w - lo_w) / 2;
        if probe_refresh(16 * 1024 * 1024, mid).delivered {
            hi_w = mid;
        } else {
            lo_w = mid + 1;
        }
    }
    assert!(lo_w > 1, "refresh + footprint capture must charge work");
    let w_below = probe_refresh(16 * 1024 * 1024, lo_w - 1);
    assert!(!w_below.delivered, "min_refresh_work - 1 must refuse");
    assert!(
        w_below.exact_exec_stop && w_below.reached_capture && !w_below.completed_capture,
        "min_refresh_work - 1 must fail specifically INSIDE WholeResultInvalidationFootprint::capture \
         (exec={}, reached={}, completed={})",
        w_below.exact_exec_stop,
        w_below.reached_capture,
        w_below.completed_capture
    );
    assert_eq!(
        w_below.remaining_consumers, 0,
        "work-exhausted refresh handle must detach source custody before Drop"
    );

    // Binary-search exact minimal refresh memory budget and pin failure to footprint capture.
    let (mut lo_m, mut hi_m) = (16_u64, 16 * 1024 * 1024_u64);
    while lo_m < hi_m {
        let mid = lo_m + (hi_m - lo_m) / 2;
        if probe_refresh(mid, 200_000).delivered {
            hi_m = mid;
        } else {
            lo_m = mid + 1;
        }
    }
    assert!(lo_m > 16, "refresh + footprint capture must charge memory");
    let m_below = probe_refresh(lo_m - 1, 200_000);
    assert!(!m_below.delivered, "min_refresh_mem - 1 must refuse");
    assert!(
        m_below.exact_exec_stop && m_below.reached_capture && !m_below.completed_capture,
        "min_refresh_mem - 1 must fail specifically INSIDE WholeResultInvalidationFootprint::capture \
         (exec={}, reached={}, completed={})",
        m_below.exact_exec_stop,
        m_below.reached_capture,
        m_below.completed_capture
    );
    assert_eq!(
        m_below.remaining_consumers, 0,
        "memory-exhausted refresh handle must detach source custody before Drop"
    );
}
