use std::sync::Arc;

use crate::recovery::{ProductUnpublishedOwnerEffectsRecord, RetainedAttemptFacts};

use super::ActiveAttemptRecord;

impl ActiveAttemptRecord {
    /// Called while the recovery catalog exclusively selects an abandoned row.
    /// This is a representation change only: existing evidence and original
    /// custody move to the retained row, without history installation, pin
    /// acquisition, dependency transfer, or any component-owner call.
    pub(crate) fn materialize_abandoned(
        &self,
        catalog_affinity: usize,
    ) -> Option<Arc<ProductUnpublishedOwnerEffectsRecord>> {
        let mut state = self.state();
        if !state.abandoned {
            return None;
        }
        let (progress, owner_results) = state
            .progress
            .retained_image()
            .into_recovery_results()
            .ok()?;
        let facts = RetainedAttemptFacts {
            admitted_at: self.admitted_at,
            identity: self.identity().clone(),
            attempt_identity: self.attempt.clone(),
            expected_head: self.expected.clone(),
            last_observed_head: state.last_observed.clone(),
            progress,
            owner_results,
            destination: state.destination.as_ref().map(|witness| {
                let (branch, incarnation) = witness.destination();
                (branch.clone(), incarnation)
            }),
        };
        let resources = state
            .resources
            .take()
            .expect("abandoned custody is restored before materialization");
        Some(ProductUnpublishedOwnerEffectsRecord::from_abandoned(
            facts,
            state.successor.clone(),
            resources,
            catalog_affinity,
            self.deadline,
            state.cause,
        ))
    }
}
