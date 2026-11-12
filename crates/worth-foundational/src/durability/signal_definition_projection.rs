//! A committed Signal effect and installation of its derived projection are distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalDefinitionProjectionOutcome {
    NotRequired,
    Installed,
    UnwoundAfterCommit,
}
