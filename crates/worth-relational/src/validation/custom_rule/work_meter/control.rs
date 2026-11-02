//! Original lease or effective serial/ancestor controls, without resource authority.
pub(super) enum PreparationControl {
    Lease(worth_execution::ExecutionLeaseStatus),
    Stage(worth_execution::ExecutionWorkLoan),
}
impl PreparationControl {
    pub(super) fn is_cancelled(&self) -> bool {
        match self {
            Self::Lease(status) => status.is_cancelled(),
            Self::Stage(_) => false,
        }
    }
    pub(super) fn deadline_elapsed(&self) -> bool {
        match self {
            Self::Lease(status) => status.deadline_elapsed(),
            Self::Stage(_) => false,
        }
    }
}
