//! Records durable commit offers during cold replay-pin admission.

use std::sync::{Arc, Mutex};

use super::ColdPinSource;
use crate::publication::patch::data::PatchStreamPosition;

#[derive(Debug)]
pub(super) struct OfferedSpy(
    pub(super) Arc<Mutex<Vec<PatchStreamPosition>>>,
    pub(super) Arc<ColdPinSource>,
);
impl crate::durability::published_medium::RelationalDurableMedium for OfferedSpy {
    fn publish(
        &self,
        a: &crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact,
    ) -> crate::durability::medium::DurablePublicationFate<String> {
        self.0.lock().unwrap().push(a.position());
        crate::durability::medium::DurablePublicationFate::Committed
    }
    fn cold_source(&self) -> Arc<dyn crate::durability::medium::RelationalDurableColdSource> {
        self.1.clone()
    }
    fn read_transition(
        &self,
        seq: crate::branch::RelationalBranchTransitionSequence,
    ) -> Result<
        crate::facade::durability::RelationalBranchTransitionArtifact,
        crate::durability::medium::RelationalDurableTransitionReadFailure,
    > {
        Err(crate::durability::medium::RelationalDurableTransitionReadFailure::Missing(seq))
    }
    fn publish_transition(
        &self,
        _: &crate::facade::durability::RelationalBranchTransitionArtifact,
    ) -> crate::durability::medium::DurablePublicationFate<String> {
        crate::durability::medium::DurablePublicationFate::Committed
    }
}
