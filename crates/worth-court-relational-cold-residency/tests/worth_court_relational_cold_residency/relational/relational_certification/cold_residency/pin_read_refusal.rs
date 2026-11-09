//! Preserve execution failure from an actual owner-issued pin request.
use worth_relational::facade::{
    runtime::RelationalPinnedReadAdmissionDenial as Denial,
    transactions::RelationalExecutionDenialCause,
};
pub(super) fn pin_denial(denial: Denial) -> RelationalExecutionDenialCause {
    match denial {
        Denial::Request(cause) => cause,
        Denial::SnapshotUnavailable => panic!("original snapshot exists"),
        Denial::IndexNotInstalled(index) => panic!("original index installed: {index:?}"),
    }
}
