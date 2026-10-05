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
