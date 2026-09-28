//! Expression conditions under concurrent change: authority revoked or a
//! source replaced after the advance is prepared, an absent document, and
//! the same request key reused.

use worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolutionDenialKind, WorthQueryEntityResolutionDenialKind,
    WorthQueryOperationAuthorizationDenialKind,
};

use super::*;

#[test]
fn revocation_after_preparation_refuses_the_acceptance() {
    let court = Court::open(5, 9_177_100);
    let days = days!(court.application);
    let retained = retained!(court.application);
    let revoked = prepare!(court, 9_177_110, |prepared| {
        let admitted = prepared.expect("the condition acceptance prepares");
        change_advance_grant(&court, "revoked", 9_177_111);
        admitted
            .condition(&court.required)
            .operand::<DocumentRetentionDaysQueryBinding, _>("days", days)
            .operand::<DocumentRetentionConditionQueryBinding, _>("retained", retained)
            .accept()
    });
    match revoked {
        Ok(WorkflowProgressOutcome::IdempotencyDenied(denial)) => {
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationIdempotencyResolutionDenialKind::Authorization
            );
            assert_eq!(
                denial
                    .authorization()
                    .map(|authorization| authorization.kind()),
                Some(WorthQueryOperationAuthorizationDenialKind::StaleAuthorization)
            );
        }
        other => panic!("a revoked actor must not settle the condition: {other:?}"),
    }
    change_advance_grant(&court, "active", 9_177_112);
    court.assert_still_awaiting(9_177_113);
}

/// Operand queries are scoped to one document, so an absent document is
/// refused by its read and never reaches the condition as a source.
#[test]
fn an_absent_document_is_refused_before_it_can_be_an_operand() {
    let court = Court::open(5, 9_177_200);
    let runtime = court.application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let absent = runtime
        .request(&principal, &scope)
        .query(DocumentRetentionDaysRead {
            identity: "no-such-document".to_owned(),
        })
        .execute();
    match absent {
        Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryEntityResolutionDenialKind::UnknownEntity
        ),
        Err(other) => panic!("an absent document must fail scope resolution: {other:?}"),
        Ok(_) => panic!("an absent document must not read"),
    }
    court.assert_still_awaiting(9_177_210);
}

#[test]
fn a_raced_acceptance_retries_under_the_same_key() {
    let court = Court::open(5, 9_177_300);
    let days = days!(court.application);
    let retained = retained!(court.application);
    let raced = prepare!(court, 9_177_310, |prepared| {
        let admitted = prepared.expect("the condition acceptance prepares");
        set_days(&court.application, 8, 9_177_311);
        admitted
            .condition(&court.required)
            .operand::<DocumentRetentionDaysQueryBinding, _>("days", days)
            .operand::<DocumentRetentionConditionQueryBinding, _>("retained", retained)
            .accept()
    });
    match raced {
        Ok(WorkflowProgressOutcome::Application(WorthQueryApplicationUncommitted::Denied(
            denial,
        ))) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ProductBasisStale
        ),
        other => panic!("a transition over a replaced source must not publish: {other:?}"),
    }
    match accept!(court, 9_177_310, days: days!(court.application), retained: retained!(court.application))
    {
        Ok(WorkflowProgressOutcome::Completed(performed)) => assert!(!performed.replayed()),
        other => panic!("the fenced key must settle fresh operands: {other:?}"),
    }
    court.assert_terminal(9_177_312, "unsatisfied");
}

#[test]
fn a_reused_key_with_changed_sources_is_intent_drift() {
    let court = Court::open(5, 9_177_400);
    assert!(matches!(
        accept!(court, 9_177_410, days: days!(court.application), retained: retained!(court.application)),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    set_days(&court.application, 8, 9_177_411);
    match accept!(court, 9_177_410, days: days!(court.application), retained: retained!(court.application))
    {
        Ok(WorkflowProgressOutcome::Application(WorthQueryApplicationUncommitted::Denied(
            denial,
        ))) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift
        ),
        other => panic!("a changed source under a settled key must not replay: {other:?}"),
    }
    court.assert_terminal(9_177_412, "satisfied");
}
