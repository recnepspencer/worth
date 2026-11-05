//! Moved whole from `domain_computation::primary_graph::product_operation::conditional_definition`: the items below and every impl they own.

/// Why a conditional definition was not admitted for an application change.
/// Admission is a check, so nothing was published.
#[derive(Debug)]
pub enum WorthQueryApplicationConditionalDefinitionAdmissionDenial {
    /// The operation has no admitted mutation product to bind a successor to.
    MissingProductObservation,
    /// The installed registry has no matching typed conditional binding.
    ConditionalBindingUnavailable { identity: &'static str },
    /// The clock handle's conditional operation is not live in this
    /// application.
    ForeignConditionalOperation,
    /// The bridge refused the conditional signal basis of the selected product.
    BridgePreparation(worth_runtime_bridge::facade::BridgeConditionalDenial),
}

impl std::fmt::Display for WorthQueryApplicationConditionalDefinitionAdmissionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "conditional definition admission refused: {self:?}"
        )
    }
}

impl std::error::Error for WorthQueryApplicationConditionalDefinitionAdmissionDenial {}
