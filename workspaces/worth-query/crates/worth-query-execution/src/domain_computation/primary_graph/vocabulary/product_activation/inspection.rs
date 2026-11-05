//! Moved whole from `domain_computation::primary_graph::product_activation::inspection`: the items below and every impl they own.

/// Why the program carried by a selected product could not be inspected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySelectedProgramInspectionDenial {
    /// The caller's read request stopped.
    ReadExecution(
        Result<
            crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    /// The runtime has no installed program support.
    ProgramSupportUnavailable,
    /// No program activation is published.
    ProgramActivationUnavailable,
    /// The product's program activation record could not be read.
    ProgramActivationUnreadable,
    /// The product's program revision is not on the installed roster.
    ProgramActivationUnrostered,
}
