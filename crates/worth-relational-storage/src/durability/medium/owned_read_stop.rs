//! Translate a selected original member read refusal into recovery evidence.
use crate::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use crate::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};

pub fn owned_stop(cause: Refusal, subject: RelationalStorageSubject) -> RelationalRecoveryStop {
    match cause {
        Refusal::Request(cause) => RelationalRecoveryStop::Request(cause),
        Refusal::Buffer(worth_execution::ExecutionRequestArrayDenial::Request(cause)) => {
            RelationalRecoveryStop::Request(cause.into())
        }
        Refusal::Buffer(worth_execution::ExecutionRequestArrayDenial::Allocation(cause)) => {
            RelationalRecoveryStop::Allocation(cause)
        }
        Refusal::Storage(cause) => RelationalStorageUnavailable { subject, cause }.into(),
        Refusal::UnsupportedProvider => DurabilityError::new(
            RecoveryFailureClass::UnsupportedLegacySemantics,
            "original protected canonical source unsupported",
        )
        .into(),
        Refusal::ForeignSource | Refusal::ForeignProtection | Refusal::Busy | Refusal::Bounds => {
            RelationalStorageUnavailable {
                subject,
                cause: RelationalStorageUnavailableCause::SourceUnavailable,
            }
            .into()
        }
        Refusal::CanonicalFrame(_) => DurabilityError::new(
            RecoveryFailureClass::CorruptCanonicalArtifact,
            "original canonical frame refused",
        )
        .into(),
        Refusal::TransitionFrame(_) => DurabilityError::new(
            RecoveryFailureClass::CorruptBranchTransitionArtifact,
            "original transition frame refused",
        )
        .into(),
    }
}
