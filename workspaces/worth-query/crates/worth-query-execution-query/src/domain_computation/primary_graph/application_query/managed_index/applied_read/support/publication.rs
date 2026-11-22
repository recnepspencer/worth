//! Only the sole final publication can position an admitted candidate Source.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead;
use worth_relational::facade::runtime::{
    ManagedIndexSourceObservation, ManagedIndexSourcePreparationDenial as Stop,
};
impl AppliedManagedSourceSupport {
    pub fn seal_published(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        after: &SnapshotHandle,
    ) -> Result<(), Stop> {
        // The retained Native issuer checks registration and exact proposal
        // runtime/commit/version. This is never a pin-to-position conversion.
        let observed = self.source.observe_published(execution, runtime, after)?;
        if let Some(prior) = self.published.get() {
            return if prior.native().same_source(&observed) {
                Ok(())
            } else {
                Err(Stop::ForeignSource)
            };
        }
        self._allocation
            .claim_arc::<ManagedIndexSourceObservation>(execution)
            .map_err(request_stop)?;
        self._allocation
            .claim_arc::<ManagedSourceRead>(execution)
            .map_err(request_stop)?;
        let observation = Arc::new(ManagedSourceRead::new(
            *self
                .association
                .access_contract()
                .graph_obligations()
                .identity()
                .bytes(),
            self.source.scope(),
            Arc::new(observed),
            Arc::clone(&self._allocation),
            None,
            self.parameters,
            Arc::clone(self.authorization()),
        ));
        match self.published.set(observation) {
            Ok(()) => Ok(()),
            Err(observation)
                if self
                    .published
                    .get()
                    .is_some_and(|prior| prior.native().same_source(observation.native())) =>
            {
                Ok(())
            }
            Err(_) => Err(Stop::ForeignSource),
        }
    }
}
fn request_stop(
    cause: worth_relational::facade::transactions::RelationalExecutionDenialCause,
) -> Stop {
    Stop::Source(worth_relational::facade::runtime::DerivedRelationReadDenial::Request(cause))
}
