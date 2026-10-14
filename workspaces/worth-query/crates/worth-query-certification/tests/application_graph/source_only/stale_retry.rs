//! One native registered attempt loses its basis, then retries its original meaning.
use super::{
    fixture,
    host::Host,
    readback::{observe_head, read_retention},
};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::{
    application_entry::*,
    primary_graph::{
        WorthQueryApplicationCommitDenialKind as Kind,
        WorthQueryApplicationCommitDenialStage as Stage, WorthQueryApplicationUncommitted as U,
        WorthQueryInvariantExecutionDenialKind as InvariantKind,
    },
};

#[path = "stale_retry/race.rs"]
mod race;
use race::{assert_stale, stale_owner};

#[test]
fn unpinned_stale_retry_reselects_with_original_input_key_and_condition() {
    let host = fixture::install();
    let operation = stale_owner(&host, 21, None);
    let before = observe_head(host.runtime(), host.current_world());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let other = host
        .runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .branches()
                .fork(host.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let operation = match operation.advance(&request.on_branch(other)) {
        WorthQuerySourceOnlyAdvance::Refused {
            operation,
            denial:
                WorthQuerySourceOnlyDenial::Mutation(
                    WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch,
                ),
        } => operation,
        _ => panic!("foreign branch returns actual retry custody"),
    };
    assert_stale(operation.native_retry_outcome().unwrap());
    assert_eq!(host.contacts.counts(), (2, 2));
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    let fresh = host.runtime().request(&fresh_principal, &fresh_scope);
    let done = fixture::completed(operation, &fresh);
    assert!(
        matches!(done.evidence(), WorthQuerySourceCompletion::Fresh { result, .. } if result.retention_days == 19)
    );
    assert_ne!(
        observe_head(host.runtime(), host.current_world()).selected_commit(),
        before.selected_commit()
    );
    assert_eq!(read_retention(host.runtime(), host.current_world()), 19);
    assert_eq!(host.contacts.counts(), (3, 3));
    done.close(&fresh)
        .unwrap_or_else(|(_, cause)| panic!("original completion closes: {cause:?}"));
    assert_eq!(host.runtime().active_application_attempts_for_test(), 0);
}

#[test]
fn unpinned_stale_live_close_refusal_keeps_evidence_then_closes_without_retry() {
    let host = fixture::install();
    let operation = stale_owner(&host, 21, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let other = host
        .runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .branches()
                .fork(host.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let (operation, denial) = operation
        .close(&request.on_branch(other))
        .err()
        .expect("foreign close returns real no-effect custody");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::FreshRequestMismatch
    ));
    assert_stale(operation.native_retry_outcome().unwrap());
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    operation
        .close(&host.runtime().request(&fresh_principal, &fresh_scope))
        .unwrap_or_else(|(_, cause)| panic!("proven no-effect close: {cause:?}"));
    assert_eq!(read_retention(host.runtime(), host.current_world()), 21);
    assert_eq!(host.contacts.counts(), (2, 2));
    assert_eq!(host.runtime().active_application_attempts_for_test(), 0);
}

#[test]
fn unpinned_stale_retry_observes_current_business_denial_without_false_completion() {
    let host = fixture::install();
    let operation = stale_owner(&host, 19, None);
    let before = observe_head(host.runtime(), host.current_world());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let terminal = match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Terminal(terminal) => terminal,
        _ => panic!("fresh source decision must retain actual business terminal"),
    };
    assert!(matches!(
        terminal.native_outcome(),
        Some(WorthQueryApplicationMutationOutcome::DomainDenied(
            super::schema::SetRetentionDenial::RetentionUnchanged
        ))
    ));
    assert_eq!(host.contacts.counts(), (3, 2));
    assert_eq!(
        observe_head(host.runtime(), host.current_world()).selected_commit(),
        before.selected_commit()
    );
    terminal
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("business terminal close: {cause:?}"));
}

#[test]
fn unpinned_stale_retry_preserves_original_mutable_fact_condition() {
    let host = fixture::install();
    let operation = stale_owner(&host, 21, Some(7));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let before = observe_head(host.runtime(), host.current_world());
    let operation = match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Refused { operation, denial } => {
            assert!(
                matches!(denial, WorthQuerySourceOnlyDenial::Mutation(WorthQueryApplicationRequestMutationDenial::Handler(
                worth_query_host::facade::primary_graph::MutationHandlerExecutionDenial::Attempt(ref cause)))
                if cause.kind() == worth_query_host::facade::primary_graph::WorthQueryApplicationAttemptDenialKind::MutationPreconditionMismatch)
            );
            operation
        }
        _ => panic!("retry cannot replace the original expected retention7 with current21"),
    };
    assert_stale(operation.native_retry_outcome().unwrap());
    assert_eq!(host.contacts.counts(), (3, 2));
    assert_eq!(
        observe_head(host.runtime(), host.current_world()).selected_commit(),
        before.selected_commit()
    );
    assert_eq!(read_retention(host.runtime(), host.current_world()), 21);
    // Fresh selection is permitted; weakening the original caller condition is not.
    operation.close(&request).unwrap_or_else(|(_, cause)| {
        panic!("refused no-effect original condition closes: {cause:?}")
    });
}

#[test]
fn unpinned_stale_retry_lost_answer_uses_keyed_recovery_without_third_handler_run() {
    let host = fixture::install();
    let operation = stale_owner(&host, 21, None);
    let guard = host
        .runtime()
        .hold_post_commit_snapshot_failure_for_test()
        .unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let operation = fixture::progress(operation, &request);
    assert_eq!(
        operation.posture(),
        WorthQuerySourceOnlyPosture::Indeterminate
    );
    assert!(operation.native_retry_outcome().is_none());
    assert_eq!(host.contacts.counts(), (3, 3));
    let operation = match operation.advance(&request) {
        WorthQuerySourceOnlyAdvance::Refused { operation, .. } => operation,
        _ => panic!("held native lost-answer boundary preserves uncertain custody"),
    };
    drop(guard);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(host.installed_schema(), &fresh_scope);
    let fresh = host.runtime().request(&fresh_principal, &fresh_scope);
    let done = fixture::completed(operation, &fresh);
    assert!(matches!(
        done.evidence(),
        WorthQuerySourceCompletion::Recovered {
            initial_posture: Some(U::Indeterminate(_)),
            ..
        }
    ));
    assert_eq!(read_retention(host.runtime(), host.current_world()), 19);
    assert_eq!(host.contacts.counts(), (3, 3));
    done.close(&fresh)
        .unwrap_or_else(|(_, cause)| panic!("actual recovered cleanup: {cause:?}"));
}

#[test]
fn unpinned_unchanged_source_completes_once_and_initial_owner_cannot_close_as_noeffect() {
    let host = fixture::install();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let (operation, denial) = fixture::operation(&host, &request, 19, 0x9176_5019)
        .close(&request)
        .err()
        .expect("initial owner has no native no-effect evidence");
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::NativeObligationRetained
    ));
    assert!(operation.native_retry_outcome().is_none());
    assert_eq!(host.contacts.counts(), (0, 0));
    let done = fixture::completed(operation, &request);
    assert_eq!(host.contacts.counts(), (1, 1));
    assert_eq!(read_retention(host.runtime(), host.current_world()), 19);
    done.close(&request)
        .unwrap_or_else(|(_, cause)| panic!("valid twin close: {cause:?}"));
}
