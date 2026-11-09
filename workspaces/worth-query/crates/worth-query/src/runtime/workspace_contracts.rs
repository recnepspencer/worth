use super::{
    WorthQueryLiveView, WorthQueryRuntimeDownstreamDelivery,
    WorthQueryRuntimeDownstreamDeliveryContract, WorthQueryRuntimeError, WorthQueryWorkspace,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

impl WorthQueryWorkspace {
    pub fn public_downstream_delivery_contract(
        &self,
    ) -> Result<WorthQueryRuntimeDownstreamDeliveryContract, IdentityDerivationDenial> {
        Ok(self.runtime.public_downstream_delivery_contract()?)
    }

    pub fn downstream_delivery<T>(
        &self,
        view: &WorthQueryLiveView<T>,
    ) -> Result<Option<WorthQueryRuntimeDownstreamDelivery>, WorthQueryRuntimeError> {
        self.runtime.downstream_delivery(view)
    }
}
