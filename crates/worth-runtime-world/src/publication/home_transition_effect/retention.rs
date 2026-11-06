//! Preserve the exact retention owner refusal while projecting internal scope.
use super::WorldHomeTransitionEffectDenial as Denial;
use crate::retention::{RetentionObligationDenial, RetentionTransferDenial};

impl Denial {
    pub(crate) fn retention(cause: RetentionObligationDenial) -> Self {
        match cause {
            RetentionObligationDenial::ForeignOwner { .. }
            | RetentionObligationDenial::InvalidComponentPair => Self::RetentionScope,
            RetentionObligationDenial::HistoryDependency(cause) => Self::transfer(cause),
            RetentionObligationDenial::ObservationCapacityExhausted
            | RetentionObligationDenial::UniquePinCapacityExhausted { .. }
            | RetentionObligationDenial::InFlightAcquisitionCapacityExhausted { .. }
            | RetentionObligationDenial::DependencyCountExhausted
            | RetentionObligationDenial::LeaseIdentityExhausted => Self::RetentionCapacity,
            RetentionObligationDenial::Relational(cause) => Self::NativeRetention(cause),
            RetentionObligationDenial::Signal(cause) => Self::SignalRetention(cause),
            RetentionObligationDenial::OwnerOperationPanicked => Self::RetentionOwnerFailure,
        }
    }

    pub(crate) fn transfer(cause: RetentionTransferDenial) -> Self {
        match cause {
            RetentionTransferDenial::BasisMismatch
            | RetentionTransferDenial::UnknownPin
            | RetentionTransferDenial::ForeignOwner => Self::RetentionScope,
            RetentionTransferDenial::DependencyCountExhausted => Self::RetentionCapacity,
            #[cfg(test)]
            RetentionTransferDenial::ReleaseDestination => Self::RetentionScope,
        }
    }
}
