//! Each idempotency cause reaches the request kind whose instruction fits it.

use worth_query_execution::facade::primary_graph::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolutionDenialKind as Resolution,
    WorthQueryOperationAuthorizationDenialKind as Authorization,
};

use super::{idempotency_kind, WorthQueryApplicationRequestMutationDenialKind as Request};

#[test]
fn every_idempotency_cause_keeps_its_own_request_kind() {
    let cases = [
        (
            Resolution::Authorization(Authorization::Stopped(Err(worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind::TimedOut))),
            Request::Authorization(Authorization::Stopped(Err(worth_query_execution::facade::primary_graph::WorthQueryProviderSessionControlStopKind::TimedOut))),
        ),
        (
            Resolution::Authorization(Authorization::Refused(AuthorizationRefusal::PermissionDenied)),
            Request::Authorization(Authorization::Refused(AuthorizationRefusal::PermissionDenied)),
        ),
        (
            Resolution::ForeignAdmission,
            Request::IdempotencyForeignAdmission,
        ),
        (
            Resolution::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 2,
            },
            Request::IdempotencyUnavailable,
        ),
        (
            Resolution::RetentionCapacityExhausted,
            Request::IdempotencyUnavailable,
        ),
        (
            Resolution::ProviderUnavailable,
            Request::IdempotencyUnavailable,
        ),
        (
            Resolution::RetentionIdentityExhausted,
            Request::IdempotencyIdentityExhausted,
        ),
        (
            Resolution::SnapshotIdentityExhausted,
            Request::IdempotencyIdentityExhausted,
        ),
        (
            Resolution::RecordedIntentUnverifiable,
            Request::IdempotencyIntentUnverifiable,
        ),
    ];
    for (resolution, expected) in cases {
        assert_eq!(idempotency_kind(resolution), expected, "{resolution:?}");
    }
}
