//! Moved whole from `domain_computation::provider_session::protocol::control_stopped`: the items below and every impl they own.

/// The request control condition that stopped provider execution before publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProviderSessionControlStopKind {
    Cancelled,
    TimedOut,
}
