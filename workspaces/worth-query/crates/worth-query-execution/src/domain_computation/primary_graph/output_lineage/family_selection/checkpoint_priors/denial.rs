//! Internal selector diagnostics; checkpoint wire grammar is unchanged.
use worth_relational::facade::mvcc::CompanionPreflightStop;

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum CheckpointPriorSelectionDenial {
    Admission {
        phase: &'static str,
        stop: CompanionPreflightStop,
    },
    Allocation {
        phase: &'static str,
        error: std::collections::TryReserveError,
    },
    CheckedArithmetic {
        phase: &'static str,
    },
    Structural(CheckpointPriorStructure),
}

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum CheckpointPriorStructure {
    DuplicateInstalledBinding,
    DuplicatePartition,
    AmbiguousPublicationHeads,
    UnexpectedOutputRole,
    MissingOperationBinding,
    MissingSourcePartition,
}

impl CheckpointPriorSelectionDenial {
    pub(super) fn admission(phase: &'static str, stop: CompanionPreflightStop) -> Self {
        Self::Admission { phase, stop }
    }

    pub(super) fn allocation(
        phase: &'static str,
        error: std::collections::TryReserveError,
    ) -> Self {
        Self::Allocation { phase, error }
    }

    pub(super) const fn arithmetic(phase: &'static str) -> Self {
        Self::CheckedArithmetic { phase }
    }

    pub(in crate::domain_computation::primary_graph) fn into_capture_denial(
        self,
    ) -> worth_relational::facade::durability::DurabilityError {
        use worth_relational::facade::durability::{DurabilityError, RecoveryFailureClass};
        let detail = match self {
            Self::Admission { phase, stop } => {
                format!("checkpoint native output head selection at {phase}: {stop:?}")
            }
            Self::Allocation { phase, error } => {
                return DurabilityError::new(
                    RecoveryFailureClass::CheckpointAllocationUnavailable,
                    format!("checkpoint native output head selection at {phase}: {error:?}"),
                )
            }
            Self::CheckedArithmetic { phase } => format!(
                "checkpoint native output head selection checked arithmetic failed at {phase}"
            ),
            Self::Structural(reason) => {
                format!("checkpoint native output head selection rejected: {reason:?}")
            }
        };
        // This existing class covers unavailable authoritative head selection;
        // the detail distinguishes admission from a structural closure failure.
        DurabilityError::new(
            RecoveryFailureClass::MissingAuthoritativeParentClosure,
            detail,
        )
    }
}
