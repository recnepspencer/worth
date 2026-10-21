use super::super::ports::RuntimeWorldObservationService;
use super::super::RuntimeWorldOwnerUnavailable;
use std::sync::{Arc, Weak};

/// Weak access to the owning World's observation service.
#[derive(Clone)]
pub struct RuntimeWorldObservationPort {
    owner: Weak<dyn RuntimeWorldObservationService + Send + Sync>,
}
impl RuntimeWorldObservationPort {
    /// Install prepared derived state while the exact issued head stays current.
    /// The callback receives currentness evidence but no mutation authority.
    /// Perform allocation/admission beforehand and return cleanup values from
    /// the callback; do not call this World owner inside the guarded section.
    pub fn while_product_branch_current<Argument, Output>(
        &self,
        expected: &crate::branch::ProductBranchObservation,
        argument: Argument,
        install: impl for<'guard> FnOnce(Argument, crate::branch::CurrentProductHead<'guard>) -> Output,
    ) -> Result<Output, crate::branch::ProductBranchCurrentnessFailure<Argument>> {
        let service = match self.service() {
            Ok(service) => service,
            Err(denial) => {
                return Err(
                    crate::branch::ProductBranchCurrentnessFailure::AdmissionDenied {
                        denial: denial.into(),
                        argument,
                    },
                )
            }
        };
        let scope = match service.admit_product_branch_currentness(expected) {
            Ok(scope) => scope,
            Err(denial) => {
                return Err(
                    crate::branch::ProductBranchCurrentnessFailure::AdmissionDenied {
                        denial: super::super::RuntimeWorldServiceDenial::Denied(denial),
                        argument,
                    },
                )
            }
        };
        scope.while_current(argument, install)
    }

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

    /// Resolves one owner-issued live occurrence without exposing or
    /// reconstructing its name-keyed product identity.
    pub fn observe_product_branch_occurrence(
        &self,
        occurrence: crate::identity::ProductBranchIncarnation,
    ) -> Result<
        crate::branch::ProductBranchObservation,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .observe_product_branch_occurrence(occurrence)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    pub fn trace_product_branch_ancestry(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        occurrence: crate::identity::ProductBranchIncarnation,
        maximum: Option<std::num::NonZeroUsize>,
    ) -> Result<
        crate::branch::ProductBranchHistoryTraversal,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .trace_product_branch_ancestry(execution, occurrence, maximum)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    pub fn continue_product_branch_ancestry(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        previous: &crate::branch::ProductBranchHistoryTraversal,
        maximum: Option<std::num::NonZeroUsize>,
    ) -> Result<
        crate::branch::ProductBranchHistoryTraversal,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .continue_product_branch_ancestry(execution, previous, maximum)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }

    pub fn observe_product_branch_history_entry(
        &self,
        history: &crate::branch::ProductBranchHistoryTraversal,
        index: usize,
    ) -> Result<
        crate::branch::ProductBranchObservation,
        super::super::RuntimeWorldServiceDenial<crate::branch::RuntimeWorldBranchAdmissionDenial>,
    > {
        self.service()?
            .observe_product_branch_history_entry(history, index)
            .map_err(super::super::RuntimeWorldServiceDenial::Denied)
    }
}
