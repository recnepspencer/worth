//! The explicit memory profile preserves its original creation route.
use super::*;
impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn create_memory_product_branch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: ProductBranchObservation,
        intent: crate::branch::ProductBranchCreationIntent,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
        retained: Option<worth_relational::facade::branch::AdmittedRelationalRetainedForkSource>,
    ) -> Result<RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchAdmissionDenial> {
        self.owner_identity()
            .admit_branch_authority(source.owner_identity())?;
        if self.state.durable.is_some() {
            return Err(RuntimeWorldBranchAdmissionDenial::HomeCreationRequestRequired);
        }

        if source.owner_identity() != self.owner_identity()
            || source.branch_identity().owner_identity() != self.owner_identity()
        {
            return Err(RuntimeWorldBranchAdmissionDenial::ForeignOwner);
        }
        if !self.branch_service_is_available() {
            return Err(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
        }
        let plans = intent
            .plans()
            .ok_or(RuntimeWorldBranchAdmissionDenial::PlansOmitted)?;
        if cancellation.is_cancelled() {
            return Err(RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect);
        }
        if retained.is_some() {
            if !matches!(
                plans.relational(),
                crate::branch::RelationalBranchCreationPlan::ForkExact { .. }
            ) || !matches!(
                plans.signal(),
                crate::branch::SignalBranchCreationPlan::ForkExact { .. }
            ) {
                return Err(RuntimeWorldBranchAdmissionDenial::UnsupportedRetainedSourcePlan);
            }
            self.admit_retained_source_lifecycle(&source)?;
        }
        if plans.is_exact_reuse() {
            return self
                .create_reused_branch(source, intent.name().clone(), cancellation)
                .map(RuntimeWorldBranchCreationOutcome::Performed);
        }
        creation::create_forked_branch(self, execution, source, intent, cancellation, retained)
    }
}
