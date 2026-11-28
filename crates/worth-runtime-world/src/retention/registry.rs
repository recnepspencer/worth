//! Exact component-pin registry vocabulary and transferred publication proof.

use crate::identity::RuntimeWorldOwnerIdentity;

mod owner;

#[allow(unused_imports)]
pub(crate) use owner::{ReservedComponentPinPairCapacity, RuntimeWorldRetentionOwner};

/// Why the Runtime World could not issue an exact component dependency claim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetentionObligationDenial {
    ForeignOwner {
        expected: RuntimeWorldOwnerIdentity,
        actual: RuntimeWorldOwnerIdentity,
    },
    InvalidComponentPair,
    HistoryDependency(super::RetentionTransferDenial),
    ObservationCapacityExhausted,
    UniquePinCapacityExhausted {
        maximum_unique_component_pins: Option<usize>,
    },
    InFlightAcquisitionCapacityExhausted {
        maximum_in_flight_reservations: Option<usize>,
    },
    DependencyCountExhausted,
    LeaseIdentityExhausted,
    Relational(worth_relational::facade::branch::RelationalBranchBasisDenial),
    Signal(worth_signal::facade::branch::SignalBranchRetentionAcquisitionDenial),
    OwnerOperationPanicked,
}
