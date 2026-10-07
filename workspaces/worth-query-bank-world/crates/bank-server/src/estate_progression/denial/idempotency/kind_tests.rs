//! An idempotency refusal keeps its exact Bank cause, authorization included.

use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolutionDenialKind as Query,
    WorthQueryOperationAuthorizationDenialKind as QueryAuthorization,
};

use super::{from_kind, BankEstateIdempotencyResolutionDenial as Bank};
use crate::BankAuthorizationDenialKind as Authorization;

#[test]
fn an_authorization_refusal_keeps_its_exact_kind() {
    for (query, bank) in [
        (QueryAuthorization::Cancelled, Authorization::Cancelled),
        (
            QueryAuthorization::DeadlineExceeded,
            Authorization::DeadlineExceeded,
        ),
        (
            QueryAuthorization::ExpiredAuthentication,
            Authorization::ExpiredAuthentication,
        ),
        (
            QueryAuthorization::PermissionDenied,
            Authorization::PermissionDenied,
        ),
    ] {
        let Bank::Authorization(denial) = from_kind(Query::Authorization(query), 2) else {
            panic!("{query:?} left the authorization cause");
        };
        assert_eq!(denial.kind(), bank);
        assert_eq!(denial.contributing_cause_count(), 2);
    }
}

#[test]
fn every_other_refusal_keeps_its_own_cause() {
    for (query, bank) in [
        (Query::ForeignAdmission, Bank::ForeignAdmission),
        (
            Query::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 3,
            },
            Bank::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 3,
            },
        ),
        (
            Query::RetentionCapacityExhausted,
            Bank::RetentionCapacityExhausted,
        ),
        (
            Query::RetentionIdentityExhausted,
            Bank::RetentionIdentityExhausted,
        ),
        (
            Query::SnapshotIdentityExhausted,
            Bank::SnapshotIdentityExhausted,
        ),
        (Query::ProviderUnavailable, Bank::ProviderUnavailable),
        (
            Query::RecordedIntentUnverifiable,
            Bank::RecordedIntentUnverifiable,
        ),
    ] {
        assert_eq!(from_kind(query, 0), bank, "{query:?}");
    }
}

#[test]
fn pending_execution_refusal_preserves_exact_kind() {
    use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind;
    let kind = WorthQueryProviderSessionDenialKind::ExecutionWorkerPanicked {
        partition_identity: Some(3),
    };
    assert_eq!(
        from_kind(Query::ExecutionDenied(kind), 0),
        Bank::ExecutionDenied(kind)
    );
}
