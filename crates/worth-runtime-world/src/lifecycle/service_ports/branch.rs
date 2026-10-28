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

    /// Retire only the saved old head while the saved replacement head is
    /// simultaneously current. Refusal changes neither reference. Success
    /// owns actual retirement work even when the original request stops at
    /// its final handoff; it does not adopt or publish the candidate.
    pub fn retire_replaced_head(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        old: &crate::branch::ProductBranchObservation,
        candidate: &crate::branch::ProductBranchObservation,
    ) -> Result<
        crate::branch::RuntimeWorldRetiredReplacement,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchReplacementDenial>,
    > {
        self.service()?
            .retire_replaced_head(execution, old, candidate)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
}
