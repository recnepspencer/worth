use super::super::ports::RuntimeWorldObservationService;
use super::super::RuntimeWorldOwnerUnavailable;
use std::sync::{Arc, Weak};

/// Weak access to the owning World's observation service.
#[derive(Clone)]
pub struct RuntimeWorldObservationPort {
    owner: Weak<dyn RuntimeWorldObservationService + Send + Sync>,
}
impl RuntimeWorldObservationPort {
    pub(in crate::lifecycle) fn new(
        owner: Weak<dyn RuntimeWorldObservationService + Send + Sync>,
    ) -> Self {
        Self { owner }
    }
    fn service(
        &self,
    ) -> Result<Arc<dyn RuntimeWorldObservationService + Send + Sync>, RuntimeWorldOwnerUnavailable>
    {
        let service = self
            .owner
            .upgrade()
            .ok_or_else(RuntimeWorldOwnerUnavailable::new)?;
        if !service.is_available() {
            return Err(RuntimeWorldOwnerUnavailable::new());
        }
        Ok(service)
    }
    pub fn observe_product_branch(
        &self,
        branch: &crate::identity::ProductBranchIdentity,
    ) -> Result<
        crate::branch::ProductBranchObservation,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .observe_product_branch(branch)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
}
