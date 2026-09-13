use super::super::ports::RuntimeWorldBranchService;
use super::super::RuntimeWorldOwnerUnavailable;
use std::sync::{Arc, Weak};

/// Weak access to the owning World's branch service.
#[derive(Clone)]
pub struct RuntimeWorldBranchPort {
    owner: Weak<dyn RuntimeWorldBranchService + Send + Sync>,
}
impl RuntimeWorldBranchPort {
    pub(in crate::lifecycle) fn new(
        owner: Weak<dyn RuntimeWorldBranchService + Send + Sync>,
    ) -> Self {
        Self { owner }
    }
    fn service(
        &self,
    ) -> Result<Arc<dyn RuntimeWorldBranchService + Send + Sync>, RuntimeWorldOwnerUnavailable>
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
    pub fn create_product_branch(
        &self,
        source: crate::branch::ProductBranchObservation,
        intent: crate::branch::ProductBranchCreationIntent,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> Result<
        super::super::ports::RuntimeWorldBranchCreationOutcome,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .create_product_branch(super::super::ports::RuntimeWorldBranchCreationRequest::new(
                source,
                intent,
                cancellation,
            ))
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
    pub fn retire_product_branch(
        &self,
        observed: &crate::branch::ProductBranchObservation,
    ) -> Result<
        crate::branch::ProductBranchRetirementReport,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchRetirementDenial>,
    > {
        self.service()?
            .retire_product_branch(observed)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
}
