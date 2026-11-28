mod claim_counts;
pub(crate) mod component_obligation;
pub(crate) mod frontier_pin;
pub(crate) mod historical_cursor;
mod obligation_transfer;
mod observation_capacity;
pub(crate) use observation_capacity::ReservedObservationCapacity;
mod registry;
mod unique_component_pin;

pub(crate) use claim_counts::ComponentClaimKind;
pub(crate) use component_obligation::{
    HistoryRetentionObligation, ObservationRetentionObligation, ProductHeadRetentionObligation,
    PublicationRetentionObligation, RetainedPartialRetentionObligation,
};
pub(crate) use obligation_transfer::{ProductHeadRetentionTransfer, RetentionTransferReceipt};
#[allow(unused_imports)]
pub(crate) use registry::{ReservedComponentPinPairCapacity, RuntimeWorldRetentionOwner};
#[allow(unused_imports)]
pub(crate) use unique_component_pin::{ExactComponentBasisKey, ExactComponentPinRequest};

pub use obligation_transfer::RetentionTransferDenial;
pub use registry::RetentionObligationDenial;
