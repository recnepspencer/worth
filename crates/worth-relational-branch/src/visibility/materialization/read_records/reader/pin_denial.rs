use crate::indexes::data::DerivedIndexId;

#[derive(Debug)]
pub enum RelationalPinnedReadAdmissionDenial {
    Request(crate::execution::denial_cause::RelationalExecutionDenialCause),
    SnapshotUnavailable,
    IndexNotInstalled(DerivedIndexId),
}
