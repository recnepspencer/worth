//! Deterministic mid-poll revocation courts exercising EVERY authority
//! boundary inside `next()`:
//!   1. After initial authority admission & BEFORE `poll_funded_bounded`
//!      (exercising `HeadAdvanced` on both `Pending` and `Overflow` paths).
//!   2. After `poll_funded_bounded` returns `Pending` & BEFORE `confirm_authority_at_head`.
//!   3. After `poll_funded_bounded` returns `Overflow` & BEFORE `confirm_authority_at_head`.
//!   4. After refresh read, projection & footprint capture & BEFORE `confirm_authority_at_head`
//!      (never leaking a refreshed `Delivered` snapshot!).
use super::access::{InspectBoard, ReadBoard, ReadBoardAccess};
use super::queries::GovTodayIntent;
use super::world::{
    install, live_limits, run_mutate, run_mutate_in_advancement, FUTURE_DUE_DAYS, TODAY_CUTOFF_DAYS,
};
use super::{mutate_op, seed};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationLiveLimits, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::{
    WholeResultLiveStageForTest, WorthQueryApplicationWholeResultLiveOutcome,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryOperationAuthorizationRefusalKind,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

fn assert_grant_revoked_and_detached<Q, R: std::fmt::Debug>(
    outcome: WorthQueryApplicationWholeResultLiveOutcome<Q, R>,
    remaining_consumers: usize,
    label: &str,
) {
    match outcome {
        WorthQueryApplicationWholeResultLiveOutcome::AuthorizationDenied(auth) => {
            assert_eq!(
                auth.kind(),
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    WorthQueryOperationAuthorizationRefusalKind::StaleAuthorization
                ),
                "{label}: expected exact Refused(StaleAuthorization), got {auth:?}"
            );
            assert_eq!(
                auth.subject(),
                "ReadBoard",
                "{label}: refusal subject must name the revoked capability"
            );
        }
        other => panic!("{label}: expected AuthorizationDenied, got {other:?}"),
    }
    assert_eq!(
        remaining_consumers, 0,
        "{label}: revoked handle must detach active source custody before Drop"
    );
}

fn run_mid_poll_revocation_at_stage(
    target_stage: WholeResultLiveStageForTest,
    buffer_capacity: usize,
    pre_commits: usize,
    label: &str,
) {
    let host = install(
        &[
            seed(1, "p1", "open", Some(FUTURE_DUE_DAYS)),
            seed(2, "p1", "open", Some(FUTURE_DUE_DAYS)),
        ],
        &[],
    );
    let scope = request_scope();
    let subscriber = authenticate_operator(host.installed_schema(), &scope);
    let mut sub = host
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
            WorthQueryApplicationLiveLimits::bounded(buffer_capacity, 64),
        )
        .expect("governed sub opens while grant is active");

    for idx in 1..=pre_commits {
        let task = ((idx - 1) % 2) as u64 + 1;
        run_mutate(
            &host,
            mutate_op("p1", 1, task, 0, TODAY_CUTOFF_DAYS, ""),
            idx as u64,
        );
    }

    let mut revoked_mid_poll = false;
    let outcome = host
        .runtime()
        .with_application_advancement(&scope, |phase| {
            let mut hook = |stage| {
                if stage == target_stage && !revoked_mid_poll {
                    revoked_mid_poll = true;
                    run_mutate_in_advancement(
                        &phase,
                        &host,
                        mutate_op("p1", 6, 0, 2, 0, "grant-p1"),
                        99,
                    );
                }
            };
            sub.next_in_advancement_with_stage_hook_for_test(
                &phase,
                &host.runtime().request(&subscriber, &scope),
                &mut hook,
            )
            .expect("next_in_advancement completes")
        })
        .expect("advancement runs");

    assert!(
        revoked_mid_poll,
        "{label}: deterministic mid-poll hook at {target_stage:?} MUST fire"
    );
    let remaining = host
        .runtime()
        .close_live_delivery()
        .remaining_live_consumers();
    assert_grant_revoked_and_detached(outcome, remaining, label);
    drop(sub);
}

#[test]
fn mid_poll_revocation_after_initial_authority_hits_head_advanced_on_pending_and_overflow() {
    run_mid_poll_revocation_at_stage(
        WholeResultLiveStageForTest::AfterInitialAuthorityBeforePoll,
        64,
        0,
        "head-advanced-pending",
    );
    run_mid_poll_revocation_at_stage(
        WholeResultLiveStageForTest::AfterInitialAuthorityBeforePoll,
        1,
        2,
        "head-advanced-overflow",
    );
}

#[test]
fn mid_poll_revocation_before_post_poll_pending_and_overflow_confirmation_fails_closed() {
    run_mid_poll_revocation_at_stage(
        WholeResultLiveStageForTest::AfterPollBeforePendingConfirm,
        64,
        0,
        "post-poll-pending-confirm",
    );
    run_mid_poll_revocation_at_stage(
        WholeResultLiveStageForTest::AfterPollBeforeOverflowConfirm,
        1,
        2,
        "post-poll-overflow-confirm",
    );
}

#[test]
fn mid_poll_revocation_after_refresh_and_footprint_capture_before_delivery_fails_closed() {
    let _ = live_limits();
    run_mid_poll_revocation_at_stage(
        WholeResultLiveStageForTest::AfterFootprintCaptureBeforeDeliveryConfirm,
        64,
        1,
        "post-refresh-delivery-confirm",
    );
}
