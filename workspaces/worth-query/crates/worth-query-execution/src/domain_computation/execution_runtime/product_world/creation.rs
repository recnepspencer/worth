use worth_runtime_world::facade::{
    ProductBranchCreationIntent, RuntimeWorldBranchAdmissionDenial,
    RuntimeWorldBranchCreationOutcome, RuntimeWorldCancellationToken, RuntimeWorldServiceDenial,
};

use super::{activation::WorthQueryProductActivationDenial, WorthQueryProductRuntime};
use crate::basis::WorthQueryProductBranchLease;

#[derive(Debug)]
pub enum WorthQueryProductBranchCreationDenial {
    CoordinationCapacityExhausted,
    CoordinationUnavailable,
    World(RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>),
}

impl WorthQueryProductRuntime {
    /// World owns creation and the returned occurrence. Query reserves only
    /// bounded activation coordination before any component owner can move.
    pub(crate) fn create_product_branch(
        &self,
        source: &WorthQueryProductBranchLease,
        intent: ProductBranchCreationIntent,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> Result<RuntimeWorldBranchCreationOutcome, WorthQueryProductBranchCreationDenial> {
        if source.observation().owner_identity() != self.owner.owner_identity() {
            return Err(WorthQueryProductBranchCreationDenial::World(
                RuntimeWorldServiceDenial::Denied(RuntimeWorldBranchAdmissionDenial::ForeignOwner),
            ));
        }
        let reservation = self.activations.reserve().map_err(|denial| match denial {
            WorthQueryProductActivationDenial::CapacityExhausted => {
                WorthQueryProductBranchCreationDenial::CoordinationCapacityExhausted
            }
            _ => WorthQueryProductBranchCreationDenial::CoordinationUnavailable,
        })?;
        let outcome = self
            .owner
            .branch_port()
            .create_product_branch(source.observation().clone(), intent, cancellation)
            .map_err(WorthQueryProductBranchCreationDenial::World)?;
        if let RuntimeWorldBranchCreationOutcome::Performed(observation) = &outcome {
            reservation.commit(observation);
        }
        Ok(outcome)
    }
}
