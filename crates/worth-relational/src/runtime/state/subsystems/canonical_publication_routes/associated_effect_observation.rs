//! Settlement descriptions require the same actual registered subject.
use super::RelationalCanonicalPublicationRoutes;
use crate::durability::authority::RelationalAssociatedEffectSettlement as Settlement;
use crate::history::{RelationalCommitEnvelopeResidency, RelationalCommitIndexEntry};
use std::sync::Arc;

impl RelationalCanonicalPublicationRoutes {
    pub(crate) fn performed_effect_residencies(
        &self,
    ) -> Vec<(
        crate::history::data::CommitId,
        Arc<RelationalCommitEnvelopeResidency>,
    )> {
        self.by_commit
            .iter()
            .filter(|route| route.is_performed())
            .map(|route| (route.commit.commit_id, route.envelope.clone()))
            .collect()
    }

    pub(crate) fn associated_effect_settlement(
        &self,
        entry: &RelationalCommitIndexEntry,
        residency: &Arc<RelationalCommitEnvelopeResidency>,
    ) -> Option<Settlement> {
        if let Some(route) = self.by_commit.get(&entry.commit.commit_id) {
            if !route.is_performed()
                || route.commit != entry.commit
                || route.roots != entry.root
                || route.position() != Some(entry.position)
                || !Arc::ptr_eq(&route.envelope, residency)
            {
                return None;
            }
            return Some(if route.is_settled() {
                Settlement::PerformedSettled
            } else {
                Settlement::PerformedPending
            });
        }
        let catalog = self
            .checkpoint_catalog
            .read()
            .unwrap_or_else(|error| error.into_inner());
        let artifact = catalog.get(&entry.commit.commit_id)?;
        let actual = artifact.envelope_residency();
        (Arc::ptr_eq(&actual, residency) && actual.journal_index().as_deref() == Some(entry))
            .then_some(Settlement::CheckpointReadmitted)
    }
}
