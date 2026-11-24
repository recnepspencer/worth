use crate::execution::denial_cause::RelationalExecutionDenialCause;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaintenanceRefusal {
    MissingPredecessor,
    UnavailableChange,
    Execution(RelationalExecutionDenialCause),
}
