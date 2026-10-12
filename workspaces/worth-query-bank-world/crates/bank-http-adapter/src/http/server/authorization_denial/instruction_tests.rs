//! A lapse is cleared by the request, a refusal by nothing it can change.

use bank_server::BankAuthorizationDenialKind as Authorization;

use super::super::super::protocol::{BankHttpDenialKind as Kind, BankHttpNextAction as Next};
use super::authorization_denial;

#[test]
fn a_lapse_names_the_action_that_clears_it() {
    for (lapse, kind, next) in [
        (Authorization::Cancelled, Kind::Cancelled, Next::Retry),
        (
            Authorization::DeadlineExceeded,
            Kind::DeadlineExceeded,
            Next::Retry,
        ),
        (
            Authorization::ExpiredAuthentication,
            Kind::Unauthenticated,
            Next::Authenticate,
        ),
    ] {
        let denial = authorization_denial(lapse);
        assert_eq!((denial.kind, denial.next_action), (kind, next), "{lapse:?}");
    }
    assert_eq!(
        authorization_denial(Authorization::PermissionDenied).next_action,
        Next::None
    );
}

#[test]
fn capacity_is_retried_but_spent_identity_asks_for_the_operator() {
    for capacity in [
        Authorization::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots: 4,
        },
        Authorization::RetentionCapacityExhausted,
    ] {
        let denial = authorization_denial(capacity);
        assert_eq!(
            (denial.kind, denial.next_action),
            (Kind::Unavailable, Next::Retry),
            "{capacity:?}"
        );
    }
    for identity in [
        Authorization::SnapshotIdentityExhausted,
        Authorization::RetentionIdentityExhausted,
        Authorization::AdmissionIdentityExhausted,
    ] {
        let denial = authorization_denial(identity);
        assert_eq!(
            (denial.kind, denial.next_action),
            (Kind::Unavailable, Next::ContactOperator),
            "{identity:?}"
        );
    }
}

#[test]
fn an_uninstalled_policy_requires_operator_repair() {
    let denial = authorization_denial(Authorization::PolicyNotInstalled);
    assert_eq!(
        (denial.kind, denial.next_action),
        (Kind::Unavailable, Next::ContactOperator)
    );
}

#[test]
fn an_authorization_read_stop_keeps_its_http_disposition() {
    use worth_query_host::facade::primary_graph::{
        RelationalExecutionDenialCause as Cause,
        WorthQueryOperationAuthorizationDenialKind as Query,
    };
    for (cause, kind, next) in [
        (Cause::Cancelled, Kind::Cancelled, Next::Retry),
        (Cause::DeadlineElapsed, Kind::DeadlineExceeded, Next::Retry),
        (
            Cause::WorkExhausted,
            Kind::ResourceExhausted,
            Next::NarrowRequest,
        ),
        (
            Cause::DeclaredMemoryExhausted {
                requested: 7,
                admitted: 3,
            },
            Kind::ResourceExhausted,
            Next::NarrowRequest,
        ),
        (
            Cause::NoActiveExecutionScope,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (
            Cause::WorkerFailed,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (Cause::NestedStopped, Kind::Unavailable, Next::Retry),
        (
            Cause::UncheckedCustomKernel,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (
            Cause::EquivalenceContractUnavailable,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
        (
            Cause::ExpectedIdentitiesNotCanonical,
            Kind::InternalDenied,
            Next::ContactOperator,
        ),
    ] {
        assert_eq!(
            authorization_denial(Query::RelationalExecution(cause).into()),
            super::BankHttpDenial::new(kind, next)
        );
    }
}
