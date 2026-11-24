//! Refusals from the physical placement and retirement of a selected checkpoint.
use crate::durability::data::recovery_stop::RelationalRecoveryStop;

#[derive(Debug)]
pub enum RelationalCheckpointAcknowledgementDenial {
    CaptureNotBound,
    ForeignSource,
    StaleCapture,
    SelectionNotKnown,
    Storage(crate::storage::data::RelationalStorageUnavailableCause),
    Stopped(worth_execution::MapKernelStop),
    Busy,
    Recovery(RelationalRecoveryStop),
    OwnedRead(super::RelationalDurableOwnedReadRefusal),
    Retirement(super::RelationalDurableMemberRetirementRefusal),
}
