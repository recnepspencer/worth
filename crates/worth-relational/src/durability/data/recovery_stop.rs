use crate::execution::RelationalExecutionDenialCause;

use super::DurabilityError;

/// An in-process recovery refusal. Request control is never durable evidence.
///
/// Durable failures retain their existing serialization contract:
/// ```
/// fn durable<T: serde::Serialize>() {}
/// durable::<worth_relational::facade::durability::DurabilityError>();
/// ```
/// A recovery stop cannot be persisted as a recovery failure:
/// ```compile_fail,E0277
/// fn durable<T: serde::Serialize>() {}
/// durable::<worth_relational::facade::durability::RelationalRecoveryStop>();
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalRecoveryStop {
    Request(RelationalExecutionDenialCause),
    Allocation(worth_execution::ExecutionAllocationDenial),
    Durability(DurabilityError),
}

impl From<DurabilityError> for RelationalRecoveryStop {
    fn from(error: DurabilityError) -> Self {
        Self::Durability(error)
    }
}

impl std::fmt::Display for RelationalRecoveryStop {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allocation(cause) => write!(formatter, "recovery allocation stopped: {cause}"),
            Self::Request(cause) => write!(formatter, "recovery request stopped: {cause:?}"),
            Self::Durability(error) => write!(formatter, "recovery durability: {error:?}"),
        }
    }
}

impl std::error::Error for RelationalRecoveryStop {}

impl From<super::RelationalNativeCheckpointCaptureDenial> for RelationalRecoveryStop {
    fn from(denial: super::RelationalNativeCheckpointCaptureDenial) -> Self {
        match denial {
            super::RelationalNativeCheckpointCaptureDenial::Durability(error) => error.into(),
            super::RelationalNativeCheckpointCaptureDenial::Allocation(error) => {
                crate::symbols::data::field_text_recovery_stop(
                    crate::symbols::data::RelationalFieldTextAdmissionDenial::Allocation(error),
                )
            }
        }
    }
}
