use crate::history::data::coordinates::CommitId;
use crate::runtime::operation_control::interruption_event::RelationalInterruptionEvent;

/// The original reason native companion admission or preparation stopped.
///
/// Execution retains the actual request refusal; counter overflow carries no
/// inferred requirement. Structural variants name
/// pending topology, a mismatched source or position, a foreign cell, or changed
/// registration, without implying resource exhaustion. Execution and interruption
/// variants retain the native cause and, when supplied, its boundary.
///
/// This is diagnostic data, not publication, currentness or cleanup authority.
/// The receiving owner supplies the reporting stage and decides recovery;
/// a stop alone neither identifies that stage nor promises a successful retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanionPreflightStop {
    Execution(crate::execution::denial_cause::RelationalExecutionDenialCause),
    TopologyPending,
    SelectedSourceMismatch,
    SelectedPositionUnavailable { commit_id: CommitId },
    ForeignCell,
    RegistrationChanged,
    ManagedSourceNotPrepared,
    ManagedSourceChanged,
    WorkCounterOverflow,
    PreparationMemoryCounterOverflow,
    Interrupted(RelationalInterruptionEvent),
}
