//! Exact current control input for one original Relational publication.

/// A current-read requirement is a pre-effect condition, not write or Signal
/// authority. A performed Relational occurrence discharges it; settlement and
/// repair retain that original occurrence without requiring the head again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentProductReadDependencyDenial {
    AlreadyRequired,
    UnsupportedComponentPlan,
    Branch(crate::branch::RuntimeWorldBranchAdmissionDenial),
    WorldHeadUnavailable,
    Native(worth_relational::facade::mvcc::RelationalPublicationReadDependencyDenial),
    DurabilityUnsupported,
}
