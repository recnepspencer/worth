//! Authority fail-closed courts (revoked grant & disabled principal), finite
//! work/memory budget exhaustion courts, and Directory Home close/reopen +
//! resubscribe durability courts for whole-result live subscriptions.
use super::access::{InspectBoard, ReadBoard, ReadBoardAccess};
use super::queries::{GovReadyIntent, GovTodayIntent, ReadyIntent};
use super::world::{
    close_directory_home, install, live_limits, open_directory_home, run_mutate, FUTURE_DUE_DAYS,
    TODAY_CUTOFF_DAYS,
};
use super::{keys_of, mutate_op, seed};
use worth_execution::{ExecutionAllocationPolicy, ExecutionResourceLease, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveOpenRequestDenial, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationLiveCloseOutcome, WorthQueryApplicationLiveOpenDenialKind,
    WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryApplicationWholeResultLiveOutcome,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryOperationAuthorizationRefusalKind,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

#[test]
fn revoked_grant_and_disabled_principal_fail_closed_with_zero_leaked_rows() {
    let host = install(
        &[
            seed(1, "p1", "open", Some(FUTURE_DUE_DAYS)),
            seed(2, "p1", "open", Some(TODAY_CUTOFF_DAYS)),
        ],
        &[(1, 2)],
    );
    let scope = request_scope();
    let subscriber = authenticate_operator(host.installed_schema(), &scope);

    let mut today_sub = host
        .runtime()
        .request(&subscriber, &scope)
        .query(GovTodayIntent {
            project: "p1".into(),
            cutoff_days: TODAY_CUTOFF_DAYS,
        })
        .subscribe_whole_result_governed(
            ReadBoard::reference(),
            InspectBoard::reference(),
            ReadBoardAccess {
                project: "p1".into(),
            },
            live_limits(),
        )
        .expect("governed today live sub opens");
    let mut ready_sub = host
        .runtime()
        .request(&subscriber, &scope)
        .query(GovReadyIntent {
            project: "p1".into(),
        })
        .subscribe_whole_result_governed(
            ReadBoard::reference(),
            InspectBoard::reference(),
            ReadBoardAccess {
                project: "p1".into(),
            },
            live_limits(),
        )
        .expect("governed ready live sub opens");

    // Queue real mutations, THEN revoke subscriber's BoardGrant BEFORE polling!
    run_mutate(&host, mutate_op("p1", 1, 1, 0, TODAY_CUTOFF_DAYS, ""), 1);
    run_mutate(&host, mutate_op("p1", 0, 2, 0, 0, "done"), 2);
    run_mutate(&host, mutate_op("p1", 6, 0, 2, 0, "grant-p1"), 3);

    assert!(matches!(
        today_sub
            .next(&host.runtime().request(&subscriber, &scope))
            .unwrap(),
        WorthQueryApplicationWholeResultLiveOutcome::AuthorizationDenied(_)
    ));
    assert!(matches!(
        today_sub
            .next(&host.runtime().request(&subscriber, &scope))
            .unwrap(),
        WorthQueryApplicationWholeResultLiveOutcome::Closed
    ));
    assert!(matches!(
        ready_sub
            .next(&host.runtime().request(&subscriber, &scope))
            .unwrap(),
        WorthQueryApplicationWholeResultLiveOutcome::AuthorizationDenied(_)
    ));

    // Re-opening while grant is revoked also fails closed!
    let open_err = host
        .runtime()
        .request(&subscriber, &scope)
        .query(GovTodayIntent {
            project: "p1".into(),
            cutoff_days: TODAY_CUTOFF_DAYS,
        })
        .subscribe_whole_result_governed(
            ReadBoard::reference(),
            InspectBoard::reference(),
            ReadBoardAccess {
                project: "p1".into(),
            },
            live_limits(),
        )
        .err()
        .expect("opening governed subscription while grant revoked must refuse");
    assert!(
        matches!(
            open_err,
            WorthQueryApplicationLiveOpenRequestDenial::CapabilityAdmission(ref auth)
                if auth.kind()
                    == WorthQueryOperationAuthorizationDenialKind::Refused(
                        WorthQueryOperationAuthorizationRefusalKind::CapabilityAuthorizationMissing
                    )
                    && auth.subject() == "ReadBoard"
        ),
        "expected exact CapabilityAdmission(Refused(CapabilityAuthorizationMissing)) for ReadBoard, got {open_err:?}"
    );

    // Restore grant, open a public subscription, queue a mutation, then DISABLE the principal!
    run_mutate(&host, mutate_op("p1", 6, 0, 1, 0, "grant-p1"), 4);
    let mut public_sub = host
        .runtime()
        .request(&subscriber, &scope)
        .query(ReadyIntent {
            project: "p1".into(),
        })
        .subscribe_whole_result(live_limits())
        .expect("public subscription opens while principal enabled");
    run_mutate(&host, mutate_op("p1", 0, 2, 0, 0, "open"), 5);
    run_mutate(&host, mutate_op("p1", 7, 0, 0, 0, ""), 6);

    assert!(matches!(
        public_sub
            .next(&host.runtime().request(&subscriber, &scope))
            .unwrap(),
        WorthQueryApplicationWholeResultLiveOutcome::StalePrincipal
    ));
    assert!(matches!(
        public_sub
            .next(&host.runtime().request(&subscriber, &scope))
            .unwrap(),
        WorthQueryApplicationWholeResultLiveOutcome::Closed
    ));
}

#[test]
fn whole_result_finite_work_and_memory_budgets_refuse_honestly() {
    let host = install(
        &[
            seed(1, "p1", "open", Some(TODAY_CUTOFF_DAYS)),
            seed(2, "p1", "open", Some(FUTURE_DUE_DAYS)),
        ],
        &[(1, 2)],
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);

    let try_open = |mem: u64, work: u64| {
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
        match host.runtime().with_funded_application_advancement(
            &scope,
            ExecutionAllocationPolicy::Execution(&lease),
            |phase| {
                host.runtime()
                    .request(&principal, &scope)
                    .query(ReadyIntent {
                        project: "p1".into(),
                    })
                    .subscribe_whole_result_in_advancement(&phase, live_limits())
                    .map(|sub| {
                        let _ = sub.close();
                    })
            },
        ) {
            Ok(inner) => inner,
            Err(adv) => Err(WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest(adv)),
        }
    };

    try_open(16 * 1024 * 1024, 100_000).expect("generous budget opens whole-result sub");

    let is_budget_refusal = |err: &WorthQueryApplicationLiveOpenRequestDenial| {
        matches!(
            err,
            WorthQueryApplicationLiveOpenRequestDenial::Open(d)
                if matches!(
                    d.kind(),
                    WorthQueryApplicationLiveOpenDenialKind::Admission(
                        WorthQueryApplicationQueryAdmissionDenialKind::ExecutionRequest(_)
                    )
                )
        ) || matches!(
            err,
            WorthQueryApplicationLiveOpenRequestDenial::ExecutionRequest(_)
                | WorthQueryApplicationLiveOpenRequestDenial::ProductSelection(_)
                | WorthQueryApplicationLiveOpenRequestDenial::PrincipalResolution(_)
                | WorthQueryApplicationLiveOpenRequestDenial::ScopeResolution(_)
        )
    };

    // Binary search minimal work budget and verify exact 1-unit boundary refusal.
    let (mut lo_w, mut hi_w) = (1_u64, 100_000_u64);
    while lo_w < hi_w {
        let mid = lo_w + (hi_w - lo_w) / 2;
        if try_open(16 * 1024 * 1024, mid).is_ok() {
            hi_w = mid;
        } else {
            lo_w = mid + 1;
        }
    }
    assert!(lo_w > 1, "whole-result open must charge nonzero work");
    let work_err = try_open(16 * 1024 * 1024, lo_w - 1)
        .expect_err("min_work - 1 must refuse whole-result open");
    assert!(
        is_budget_refusal(&work_err),
        "unexpected work denial: {work_err:?}"
    );

    // Binary search minimal memory budget and verify exact 1-byte boundary refusal.
    let (mut lo_m, mut hi_m) = (16_u64, 16 * 1024 * 1024_u64);
    while lo_m < hi_m {
        let mid = lo_m + (hi_m - lo_m) / 2;
        if try_open(mid, 100_000).is_ok() {
            hi_m = mid;
        } else {
            lo_m = mid + 1;
        }
    }
    assert!(lo_m > 16, "whole-result open must charge nonzero memory");
    let mem_err =
        try_open(lo_m - 1, 100_000).expect_err("min_mem - 1 must refuse whole-result open");
    assert!(
        is_budget_refusal(&mem_err),
        "unexpected memory denial: {mem_err:?}"
    );

    let mut sub = host
        .runtime()
        .request(&principal, &scope)
        .query(ReadyIntent {
            project: "p1".into(),
        })
        .subscribe_whole_result(live_limits())
        .expect("sub opens");
    run_mutate(&host, mutate_op("p1", 0, 2, 0, 0, "done"), 1);
    let tight = ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(16 * 1024 * 1024), Some(1)),
        ),
        cancellation: scope.cancellation().execution_token(),
        deadline: scope.deadline(),
    })
    .unwrap();
    let next_outcome = host
        .runtime()
        .with_funded_application_advancement(
            &scope,
            ExecutionAllocationPolicy::Execution(&tight),
            |phase| {
                sub.next_in_advancement(&phase, &host.runtime().request(&principal, &scope))
                    .unwrap()
            },
        )
        .unwrap();
    assert!(
        matches!(
            next_outcome,
            WorthQueryApplicationWholeResultLiveOutcome::ExecutionRequest(_)
        ),
        "expected ExecutionRequest on tight work budget during next, got {next_outcome:?}"
    );
}

#[test]
fn directory_home_close_reopen_and_resubscribe_preserves_durable_state() {
    let dir = std::env::temp_dir().join(format!(
        "worth-p2c-dir-home-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let seeds = vec![
        seed(1, "p1", "open", Some(FUTURE_DUE_DAYS)),
        seed(2, "p1", "open", Some(TODAY_CUTOFF_DAYS)),
    ];
    {
        let home = open_directory_home(dir.clone(), &seeds, &[(1, 2)]);
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let mut sub = host
            .runtime()
            .request(&principal, &scope)
            .query(ReadyIntent {
                project: "p1".into(),
            })
            .subscribe_whole_result(live_limits())
            .expect("ready sub opens on directory home");
        assert_eq!(
            sub.initial()
                .unwrap()
                .rows()
                .iter()
                .map(|r| r.key)
                .collect::<Vec<_>>(),
            vec![2]
        );
        run_mutate(host, mutate_op("p1", 0, 2, 0, 0, "done"), 1);
        assert_eq!(
            keys_of(
                sub.next(&host.runtime().request(&principal, &scope))
                    .unwrap()
            ),
            vec![1]
        );
        assert!(matches!(
            sub.close(),
            WorthQueryApplicationLiveCloseOutcome::Completed(_)
        ));
        close_directory_home(home);
    }
    {
        let home = open_directory_home(dir.clone(), &seeds, &[(1, 2)]);
        let host = home.runtime();
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let mut sub = host
            .runtime()
            .request(&principal, &scope)
            .query(ReadyIntent {
                project: "p1".into(),
            })
            .subscribe_whole_result(live_limits())
            .expect("ready sub reopens on persisted directory home");
        assert_eq!(
            sub.initial()
                .unwrap()
                .rows()
                .iter()
                .map(|r| r.key)
                .collect::<Vec<_>>(),
            vec![1],
            "reopened home must reflect durable persisted blocker completion"
        );
        run_mutate(host, mutate_op("p1", 0, 2, 0, 0, "open"), 2);
        assert_eq!(
            keys_of(
                sub.next(&host.runtime().request(&principal, &scope))
                    .unwrap()
            ),
            vec![2]
        );
        assert!(matches!(
            sub.close(),
            WorthQueryApplicationLiveCloseOutcome::Completed(_)
        ));
        close_directory_home(home);
    }
    let _ = std::fs::remove_dir_all(&dir);
}
