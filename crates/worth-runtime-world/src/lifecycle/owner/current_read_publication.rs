//! One original publication's current control dependency, discharged at Native.
use crate::branch::{ProductBranchCurrentnessScope, ProductBranchObservation};
use crate::lifecycle::ports::RuntimeWorldObservationService;
use crate::publication::{
    CurrentProductReadDependencyDenial as Denial, NoEffectCause, NoEffectCompositePublication,
    ReservedCompositePublicationAttempt,
};
use worth_relational::facade::mvcc::{
    PreparedRelationalCommitCandidate, RelationalPublicationOutcome,
};

use super::RuntimeWorldOwnerRoot;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Admit the actual owner/cell before lowering. This establishes no
    /// currentness fact: that condition is checked at the actual Native effect.
    pub(super) fn admit_current_read_requirement(
        &self,
        current: &ProductBranchObservation,
        expected: &ProductBranchObservation,
    ) -> Result<(), NoEffectCompositePublication> {
        drop(self.current_read_scope(current).map_err(|denial| {
            NoEffectCompositePublication::new(
                NoEffectCause::CurrentReadDependency(denial),
                Some(expected.clone()),
            )
        })?);
        Ok(())
    }

    pub(super) fn publish_relational_with_current_read(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        attempt: &ReservedCompositePublicationAttempt,
        candidate: PreparedRelationalCommitCandidate,
    ) -> Result<RelationalPublicationOutcome, Denial> {
        let port = self.state.relational.publication_port();
        let Some(current) = attempt.plan().relational().current_read() else {
            return Ok(port.compare_and_publish(execution, candidate));
        };
        let scope = self.current_read_scope(current)?;
        // The scope owns an operation, but holds no cell guard during Native
        // companion preparation or owning cleanup. No Signal/World effect runs
        // inside the purpose-specific cutover receiver.
        let scoped = match port.with_prepared_memory_cutover(execution, candidate, |cutover| {
            scope.perform_memory_publication(cutover)
        }) {
            Ok(scoped) => scoped,
            Err(outcome) => return Ok(outcome),
        };
        let (guarded, publication) = scoped.into_parts();
        match publication {
            // An actual Native fact wins over the caller's callback result.
            Some(outcome) => Ok(outcome),
            None => Err(guarded.err().unwrap_or(Denial::WorldHeadUnavailable)),
        }
    }

    fn current_read_scope<'selected>(
        &self,
        current: &'selected ProductBranchObservation,
    ) -> Result<ProductBranchCurrentnessScope<'selected>, Denial> {
        self.admit_product_branch_currentness(current)
            .map_err(Denial::Branch)
    }
}
