use super::*;
use sha2::{Digest, Sha256};
#[derive(Debug)]
pub(super) struct Attachment(pub(super) Arc<TestDurableMedium>);
impl RelationalDurableMedium for Attachment {
    fn checkpoint_selection(
        &self,
        subject: &OpaqueCapturedCheckpointSubject,
    ) -> CheckpointSelectionObservation {
        match self.0.lock().selected.as_ref() {
            Some(selected) if selected.same_capture(subject) => {
                CheckpointSelectionObservation::Selected(CheckpointSelectionIdentity {
                    generation: 1,
                    manifest_sha256: Sha256::digest(subject.bytes()).into(),
                })
            }
            _ => CheckpointSelectionObservation::NotSelected,
        }
    }
    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        self.0.clone()
    }
    fn read_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        let contents = self.0.lock();
        let bytes = contents
            .transitions
            .entries
            .get(&sequence.0)
            .ok_or(RelationalDurableTransitionReadFailure::Missing(sequence))?;
        RelationalBranchTransitionArtifact::from_untrusted_bytes(bytes.clone())
            .map_err(RelationalDurableTransitionReadFailure::Frame)
    }
    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> DurablePublicationFate<String> {
        let mut contents = self.0.lock();
        contents.offered.push(artifact.clone());
        if contents
            .commits
            .publish(artifact.position().0, artifact.as_bytes())
        {
            DurablePublicationFate::Committed
        } else {
            DurablePublicationFate::ProvenNoEffect {
                detail: "not next or identical original position".into(),
            }
        }
    }
    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        if self
            .0
            .lock()
            .transitions
            .publish(artifact.sequence().0, artifact.as_bytes())
        {
            DurablePublicationFate::Committed
        } else {
            DurablePublicationFate::ProvenNoEffect {
                detail: "not next or identical original transition".into(),
            }
        }
    }
}
