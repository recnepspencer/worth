use super::super::ports::RuntimeWorldBranchService;
use super::super::RuntimeWorldOwnerUnavailable;
use std::sync::{Arc, Weak};

/// Weak access to the owning World's branch service.
#[derive(Clone)]
pub struct RuntimeWorldBranchPort {
    owner: Weak<dyn RuntimeWorldBranchService + Send + Sync>,
}
impl RuntimeWorldBranchPort {
    /// Burn a descriptive name coordinate from the actual branch-incarnation
    /// issuer. Creation reserves its destination later from that same issuer;
    /// its durable original therefore bounds every name used for owner effects.
    /// This coordinate grants no permission to create or move a branch.
    pub fn reserve_creation_name_ordinal(
        &self,
    ) -> Result<
        u64,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .reserve_creation_name_ordinal()
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    /// Lend the same durable World's final source while the original Home
    /// owner publishes its descriptor. The callback must not reenter this cell.
    pub fn with_home_program_binding<R>(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        expected: &crate::branch::ProductBranchObservation,
        required: &worth_proof::HomeProgramTransitionRequired,
        successor: &worth_proof::AdmittedHomeProgramTransitionSuccessor<
            &worth_foundational::facade::ApplicationHomeBinding,
        >,
        handoff: impl for<'owner> FnOnce(super::super::PreparedWorldHomeProgramBinding<'owner>) -> R,
    ) -> Result<R, super::super::WorldHomeProgramBindingDenial> {
        let service = self
            .service()
            .map_err(|_| super::super::WorldHomeProgramBindingDenial::OwnerUnavailable)?;
        let mut handoff = Some(handoff);
        let mut result = None;
        service.with_home_program_binding(
            request,
            expected,
            required,
            successor,
            &mut |prepared| {
                result = Some(handoff
                    .take()
                    .expect("the original final-source guard lends once")(
                    prepared
                ));
            },
        )?;
        Ok(result.expect("a successful final-source guard invokes its handoff"))
    }

    /// Resume the SAME durable creation. Existing inspection views must be dropped.
    pub fn resume_creation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: &crate::recovery::ProductUnpublishedRecoveryHandle,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> Result<
        super::super::ports::RuntimeWorldBranchCreationOutcome,
        crate::recovery::RuntimeWorldRecoveryDenial,
    > {
        let owner = self
            .service()
            .map_err(crate::recovery::RuntimeWorldRecoveryDenial::OwnerUnavailable)?;
        owner.resume_creation(execution, handle, cancellation)
    }

    /// Admit exact retained truth without claiming its source head is current.
    /// No destination is reserved and no component fork runs during admission.
    pub fn admit_retained_branch_source(
        &self,
        source: &crate::branch::ProductBranchObservation,
    ) -> Result<
        crate::branch::RetainedProductBranchCreationSource,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .admit_retained_branch_source(source)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    /// Create independent component forks from the admitted retained source.
    /// This process-local profile requires ForkExact for both components and
    /// Signal's original exact-source admission. Partial effects retain the
    /// ordinary World recovery and cleanup owners.
    pub fn create_product_branch_from_retained_source(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: crate::branch::RetainedProductBranchCreationSource,
        intent: crate::branch::ProductBranchCreationIntent,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> Result<
        super::super::ports::RuntimeWorldBranchCreationOutcome,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .create_product_branch(
                super::super::ports::RuntimeWorldBranchCreationRequest::retained(
                    execution,
                    source,
                    intent,
                    cancellation,
                ),
            )
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: crate::branch::ProductBranchObservation,
        intent: crate::branch::ProductBranchCreationIntent,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> Result<
        super::super::ports::RuntimeWorldBranchCreationOutcome,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .create_product_branch(super::super::ports::RuntimeWorldBranchCreationRequest::new(
                execution,
                source,
                intent,
                cancellation,
            ))
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
    /// Directory creation consumes the selected branch owner's actual programme proof.
    pub fn create_home_product_branch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: super::super::RuntimeWorldHomeBranchCreationRequest,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> Result<
        super::super::ports::RuntimeWorldBranchCreationOutcome,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .create_product_branch(
                super::super::ports::RuntimeWorldBranchCreationRequest::home(
                    execution,
                    request,
                    cancellation,
                ),
            )
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
