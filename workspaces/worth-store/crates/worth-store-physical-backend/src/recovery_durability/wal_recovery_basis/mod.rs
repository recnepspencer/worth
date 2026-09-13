mod append_receipt;
mod durability_observation;

#[cfg(feature = "certification-test-authority")]
pub use append_receipt::WalAppendFailureObservation;
pub use append_receipt::{WalAppendObservationScope, WalAppendReceipt, WalFrameDigest};
pub use durability_observation::{
    WalDurabilityObservation, WalDurabilityObservationBasis, WalDurabilityObservationDenial,
    WalDurabilityObservationDenialKind,
};
