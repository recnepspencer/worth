//! A callback refused on its owner read tells the sender whether to retry.

use axum::http::StatusCode;
use worth_query_host::facade::primary_graph::{
    WorthQueryCommittedDispatchOutboxReadDenial as Read, WorthQueryInboundAdmissionDenial as Denial,
};

use super::status;

#[test]
fn a_gone_basis_is_not_retried_and_owner_faults_are_the_servers() {
    let cases = [
        (Read::ExactCommitUnavailable, StatusCode::GONE),
        (
            Read::SnapshotIdentityExhausted,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (Read::ForeignRuntime, StatusCode::INTERNAL_SERVER_ERROR),
        (
            Read::AmbiguousCorrelation,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
        (Read::WrongRecordKind, StatusCode::INTERNAL_SERVER_ERROR),
        (Read::NotAuthoritative, StatusCode::INTERNAL_SERVER_ERROR),
        (Read::Malformed, StatusCode::INTERNAL_SERVER_ERROR),
        (Read::CommitMismatch, StatusCode::INTERNAL_SERVER_ERROR),
        (Read::RecordMismatch, StatusCode::INTERNAL_SERVER_ERROR),
        (Read::PendingPublication, StatusCode::SERVICE_UNAVAILABLE),
        (
            Read::CommittedIndexUnavailable,
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        (
            Read::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 2,
            },
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        (Read::Missing, StatusCode::CONFLICT),
    ];
    for (read, expected) in cases {
        assert_eq!(status(Denial::OwnerReadDenied(read)), expected, "{read:?}");
    }
    assert_eq!(
        status(Denial::RetryBeforeAcceptance),
        StatusCode::SERVICE_UNAVAILABLE
    );
}
