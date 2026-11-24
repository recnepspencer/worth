//! Publish through the test medium attached to an admitted runtime.

use super::*;

impl DurableMedium for OwnedMediumAttachment {
    fn publish_initial_inputs(
        &self,
        artifact: &InitArt,
        request: &Req<'_, '_>,
    ) -> Fate<InitDetail> {
        if let Err(cause) = request.consult() {
            return Fate::ProvenNoEffect {
                detail: InitDetail::Request(cause.into()),
            };
        }
        let mut state = self.0.lock();
        if state
            .initial_inputs
            .as_ref()
            .is_some_and(|old| old.as_bytes() == artifact.as_bytes())
        {
            return Fate::Committed;
        }
        let revision = state
            .initial_inputs
            .as_ref()
            .map_or(0, |old| old.revision());
        if artifact.revision() != revision + 1 {
            return Fate::ProvenNoEffect {
                detail: InitDetail::Physical("initial input revision conflict".into()),
            };
        }
        state.initial_inputs = Some(artifact.clone());
        Fate::Committed
    }
    fn checkpoint_selection(&self, subject: &CheckpointSubject) -> SelectionObs {
        match self.0.lock().selected.as_ref() {
            Some(selected) if selected.same_capture(subject) => {
                SelectionObs::Selected(CheckpointSelectionIdentity {
                    generation: 1,
                    manifest_sha256: Sha256::digest(subject.bytes()).into(),
                })
            }
            _ => SelectionObs::NotSelected,
        }
    }
    fn cold_source(&self) -> Arc<dyn ColdSource> {
        self.0.clone()
    }
    fn read_transition(&self, sequence: Seq) -> Result<TransArt, TransReadErr> {
        let bytes = self
            .0
            .lock()
            .transitions
            .entries
            .get(&sequence.0)
            .cloned()
            .ok_or(TransReadErr::Missing(sequence))?;
        TransArt::from_untrusted_bytes(bytes).map_err(TransReadErr::Frame)
    }
    fn publish(&self, artifact: &CommitArt) -> Fate<String> {
        self.0.publish_commit(artifact)
    }
    fn publish_transition(&self, artifact: &TransArt) -> Fate<String> {
        self.0.publish_transition(artifact)
    }
}
