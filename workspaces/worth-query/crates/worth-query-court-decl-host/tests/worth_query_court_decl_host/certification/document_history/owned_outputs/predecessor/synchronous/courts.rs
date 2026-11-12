//! The coupled handler uses its independently admitted C read control.
use super::super::{courts, install, PROBE};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
fn run(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install(worker);
    courts::reset(false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    courts::settle(&request, &host).unwrap();
    for value in [2, 3] {
        let outcome = request
            .advanced()
            .mutate(Intent {
                key: "a".into(),
                value,
            })
            .without_source()
            .idempotency(&format!("coupled-predecessor-{value}"))
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{outcome:?}"
        );
        PROBE.with(|probe| {
            assert_eq!(
                probe.borrow().values.last(),
                Some(&Ok(Some(value - 1))),
                "the first predecessor consumed committed 1; the next consumed applied 2"
            )
        });
        let output = request
            .query(query::first_output::Read("predecessor:a".into()))
            .execute()
            .unwrap();
        assert_eq!(output.rows()[0].value, value + 9000);
        let contacts = host.producer_contacts_on_this_thread_for_test();
        courts::settle(&request, &host).unwrap();
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    }
}
#[test]
fn coupled_predecessor_reads_committed_then_applied_inputs() {
    run(false);
}
#[test]
fn worker_only_coupled_predecessor_reads_committed_then_applied_inputs() {
    run(true);
}

enum ReadRefusal {
    Permission,
    Scope,
    Undeclared,
}
fn refusal(cause: ReadRefusal) {
    let wrong_scope = matches!(cause, ReadRefusal::Scope);
    let undeclared = matches!(cause, ReadRefusal::Undeclared);
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install(false);
    courts::reset(false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    courts::settle(&request, &host).unwrap();
    if matches!(cause, ReadRefusal::Permission) {
        let revoked = request
            .advanced()
            .mutate(member_edit::Intent {
                key: "m".into(),
                value: 6,
            })
            .without_source()
            .idempotency(&"revoke-coupled-read".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                revoked,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{revoked:?}"
        );
    }
    courts::reset(wrong_scope);
    PROBE.with(|probe| probe.borrow_mut().undeclared = undeclared);
    let checkpoint = checkpoint(&host);
    let before = host.current_world();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"denied-coupled-read".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("the handler deliberately swallowed the read refusal, but no effect is legal: receipt {:?}", outcome.receipt().map(|receipt| receipt.commit_reference()))
    };
    let producer = denial
        .coupled_producer_denial()
        .expect("the actual producer refusal");
    let (expected, subject) = match cause {
        ReadRefusal::Permission => (
            WorthQueryOutputDemandDenialKind::RequestAuthorization(
                WorthQueryOperationAuthorizationRefusalKind::PermissionDenied,
            ),
            "ReadOldInputs",
        ),
        ReadRefusal::Scope => (
            WorthQueryOutputDemandDenialKind::RequestAuthorization(
                WorthQueryOperationAuthorizationRefusalKind::ScopeMismatch,
            ),
            "ReadWrongScope",
        ),
        ReadRefusal::Undeclared => (
            WorthQueryOutputDemandDenialKind::PredecessorRead(Box::new(
                WorthQueryPredecessorReadDenial::UndeclaredPredecessorRead,
            )),
            "UndeclaredPredecessorRead",
        ),
    };
    assert_eq!(producer.kind(), expected);
    assert!(producer.subject().contains(subject), "{producer:?}");
    if matches!(cause, ReadRefusal::Permission) {
        assert!(
            producer.subject().contains("InputReadPolicy"),
            "{producer:?}"
        );
    }
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "the outer producer's authorization succeeded"
    );
    assert_eq!(host.current_world(), before);
    assert_model_unchanged(&host, &checkpoint);
}

pub(super) fn checkpoint(host: &super::super::Host) -> WorthQueryApplicationCheckpoint {
    host.with_host_advancement(|phase| {
        host.capture_application_checkpoint(
            &phase,
            WorthQueryCheckpointCapturePolicy::SystemAllocation,
        )
        .unwrap()
    })
    .unwrap()
}
pub(super) fn assert_model_unchanged(
    host: &super::super::Host,
    before: &WorthQueryApplicationCheckpoint,
) {
    let after = checkpoint(host);
    host.with_host_advancement(|phase| {
        let old = host
            .runtime()
            .inspect_application_checkpoint(&phase, before.bytes())
            .unwrap();
        let now = host
            .runtime()
            .inspect_application_checkpoint(&phase, after.bytes())
            .unwrap();
        assert_eq!(now.model, old.model);
        assert_eq!(now.live_records, old.live_records);
        assert_eq!(
            now.current_publication_records,
            old.current_publication_records
        );
    })
    .unwrap();
}

#[test]
fn coupled_producer_grant_cannot_replace_the_read_permission() {
    refusal(ReadRefusal::Permission);
}
#[test]
fn coupled_predecessor_scope_mismatch_cannot_be_swallowed() {
    refusal(ReadRefusal::Scope);
}

#[test]
fn coupled_undeclared_predecessor_read_cannot_be_swallowed() {
    refusal(ReadRefusal::Undeclared);
}
