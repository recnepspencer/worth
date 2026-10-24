//! The in-crate durable medium Relational's own durability tests publish to.
//!
//! It is a test double of Worth Store's Relational home, not a second durable
//! path: it keeps artifact bytes by position in memory, accepts only the next
//! position (or an identical retry), holds the latest native checkpoint with
//! the position it was filed under, and hands both back as a
//! `RelationalDurableRecoverySource` the way a home does on reopen.

mod journal;
mod runtime_attachment;
mod scripted_publication;
mod tamper;

use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) use runtime_attachment::{
    attached_durable_medium, attached_test_medium, bind_durable_medium,
};
pub(crate) use scripted_publication::ScriptedPublication;
pub(crate) use tamper::ARTIFACT_HEADER_COMMIT_ID_OFFSET;

use crate::durability::canonical_commit_artifact::{
    RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactFrameRefusal,
};
use crate::durability::data::RelationalNativeCheckpoint;
use crate::durability::medium::{
    RelationalDurableCheckpointSource, RelationalDurableMedium, RelationalDurablePublication,
    RelationalDurableRecoverySource,
};
use crate::publication::patch::data::PatchStreamPosition;
use journal::TestDurableJournal;

#[derive(Debug, Default)]
pub struct TestDurableMedium {
    journal: Mutex<TestDurableJournal>,
}

impl TestDurableMedium {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// The last position this medium made durable, or zero.
    pub(crate) fn head(&self) -> PatchStreamPosition {
        self.journal().head()
    }

    /// Every position this medium holds, ascending.
    pub(crate) fn positions(&self) -> Vec<PatchStreamPosition> {
        self.journal().positions()
    }

    /// The exact bytes held at `position`.
    pub(crate) fn artifact_bytes(&self, position: PatchStreamPosition) -> Option<Box<[u8]>> {
        self.journal().bytes_at(position).map(Box::from)
    }

    /// Every artifact a publication offered, in the order offered, whatever
    /// the answer was.
    pub(crate) fn offered(&self) -> Vec<RelationalCanonicalCommitArtifact> {
        self.journal().offered.clone()
    }

    /// File a captured native checkpoint under the position it covers,
    /// replacing any earlier one, as a home's `checkpoint()` does.
    pub(crate) fn file_checkpoint(&self, checkpoint: RelationalNativeCheckpoint) {
        let covered = checkpoint
            .covered_position()
            .expect("a checkpoint this process captured knows its coverage");
        self.file_checkpoint_under(covered, checkpoint);
    }

    /// File checkpoint bytes under an explicit position.
    pub(crate) fn file_checkpoint_under(
        &self,
        covered: PatchStreamPosition,
        checkpoint: RelationalNativeCheckpoint,
    ) {
        self.journal().checkpoint =
            Some(RelationalDurableCheckpointSource::new(covered, checkpoint));
    }

    /// What a home would hand Relational on reopen: the filed checkpoint and
    /// every artifact after its coverage. A frame-damaged artifact refuses
    /// here, as it would at the home's own read.
    pub(crate) fn recovery_source(
        &self,
    ) -> Result<RelationalDurableRecoverySource, RelationalCanonicalCommitArtifactFrameRefusal>
    {
        let journal = self.journal();
        let checkpoint = journal.checkpoint.clone();
        let covered = checkpoint.as_ref().map_or(
            PatchStreamPosition(0),
            RelationalDurableCheckpointSource::covered,
        );
        let tail = journal
            .artifacts_after(covered)
            .map(|bytes| RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.to_vec()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RelationalDurableRecoverySource::new(checkpoint, tail))
    }

    /// Answer the next publication as scripted instead of by position.
    pub(crate) fn script_next_publication(&self, answer: ScriptedPublication) {
        self.journal().scripted.push_back(answer);
    }

    fn journal(&self) -> MutexGuard<'_, TestDurableJournal> {
        self.journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl RelationalDurableMedium for TestDurableMedium {
    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> RelationalDurablePublication {
        let mut journal = self.journal();
        journal.offered.push(artifact.clone());
        match journal.scripted.pop_front() {
            Some(answer) => answer.answer(&mut journal, artifact),
            None => journal.publish_at_head(artifact),
        }
    }

    fn in_crate_test_medium(&self) -> Option<&TestDurableMedium> {
        Some(self)
    }
}
