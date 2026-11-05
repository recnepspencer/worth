//! Each owner-read refusal of a callback asks for a retry only when one can succeed.

use super::map_owner_read;
use crate::domain_computation::primary_graph::{
    WorthQueryCommittedDispatchOutboxReadDenial as Read, WorthQueryInboundAdmissionDenial as Denial,
};

#[test]
fn only_a_read_that_can_later_succeed_asks_for_a_retry() {
    for read in [
        Read::PendingPublication,
        Read::CommittedIndexUnavailable,
        Read::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots: 2,
        },
    ] {
        assert_eq!(
            map_owner_read(read),
            Denial::RetryBeforeAcceptance,
            "{read:?}"
        );
    }
    assert_eq!(map_owner_read(Read::Missing), Denial::UnknownCorrelation);
}

#[test]
fn a_gone_basis_or_spent_identity_keeps_its_exact_cause() {
    for read in [
        Read::ExactCommitUnavailable,
        Read::SnapshotIdentityExhausted,
        Read::ForeignRuntime,
        Read::AmbiguousCorrelation,
        Read::WrongRecordKind,
        Read::NotAuthoritative,
        Read::Malformed,
        Read::CommitMismatch,
        Read::RecordMismatch,
    ] {
        assert_eq!(
            map_owner_read(read),
            Denial::OwnerReadDenied(read),
            "{read:?}"
        );
    }
}
