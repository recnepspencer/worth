mod component_obligation;
mod dependency_counts;
mod obligation_transfer;
mod observation_capacity;
pub(crate) use observation_capacity::ReservedObservationCapacity;
mod registry;
mod unique_component_pin;

pub(crate) use component_obligation::{
    HistoryRetentionObligation, ObservationRetentionObligation, ProductHeadRetentionObligation,
    PublicationRetentionObligation, RetainedPartialRetentionObligation,
};
pub use dependency_counts::{ComponentBasisDependencyClass, ComponentBasisDependencyCounts};
#[cfg(test)]
pub(crate) use obligation_transfer::ComponentBasisObligationTransferDestination;
pub(crate) use obligation_transfer::{
    ProductHeadRetentionTransfer, RetentionTransferDenial, RetentionTransferReceipt,
};
#[allow(unused_imports)]
pub(crate) use registry::{
    ReservedComponentPinPairCapacity, RetentionObligationDenial, RuntimeWorldRetentionOwner,
};
#[allow(unused_imports)]
pub(crate) use unique_component_pin::{ExactComponentBasisKey, ExactComponentPinRequest};

pub use registry::{RetentionCostSnapshot, RetentionReclamationReport};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_one_freezes_independent_keys_and_closed_vocabulary() {
        assert_eq!(ComponentBasisDependencyClass::ALL.len(), 6);
        assert!(matches!(
            ComponentBasisObligationTransferDestination::Release,
            ComponentBasisObligationTransferDestination::Release
        ));
        let _ = std::mem::size_of::<ObservationRetentionObligation>();
        let _ = std::mem::size_of::<PublicationRetentionObligation>();
        let _ = std::mem::size_of::<RetainedPartialRetentionObligation>();
        let _ = std::mem::size_of::<RetentionTransferReceipt>();
    }
}
