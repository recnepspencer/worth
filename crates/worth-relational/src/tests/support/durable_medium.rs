//! The in-crate durable medium Relational's own durability tests publish to.
//!
//! It is a test double of Worth Store's Relational home, not a second durable
//! path: it keeps commit artifact bytes by position and branch transition
//! artifact bytes by sequence in memory, accepts only the next position or
//! sequence (or an identical retry), holds the latest native checkpoint with
//! the coverage it was filed under, and hands all three back as a
//! `RelationalDurableRecoverySource` the way a home does on reopen.
//!
//! `TestDurableMedium` is the durable contents, standing in for a home's
//! Store; it outlives runtimes. A runtime publishes through an `attachment()`,
//! as each opening of a home builds its own journal medium over the Store.
//! The attachment is the medium a runtime owns, and is moved into exactly one
//! runtime.

mod journal;
mod runtime_attachment;
mod scripted_publication;
mod tamper;
mod transition_contents;
mod transition_journal;

use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) use runtime_attachment::{
    attached_durable_medium, attached_test_medium, bind_durable_medium, TestDurableMediumAttachment,
};
pub(crate) use scripted_publication::ScriptedPublication;
pub(crate) use tamper::ARTIFACT_HEADER_COMMIT_ID_OFFSET;

use crate::branch::RelationalBranchTransitionSequence;
use crate::durability::branch_transition_artifact::{
    RelationalBranchTransitionArtifact, RelationalBranchTransitionArtifactFrameRefusal,
};
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
use transition_journal::TestTransitionJournal;

#[derive(Debug, Default)]
pub struct TestDurableMedium {
    journal: Mutex<TestDurableJournal>,
    transitions: Mutex<TestTransitionJournal>,
}

/// Why the test medium could not hand back a recovery source: an artifact
/// whose frame is damaged refuses at the medium's own read, as a home's does.
#[derive(Debug)]
pub(crate) enum TestRecoverySourceRefusal {
    Commit(RelationalCanonicalCommitArtifactFrameRefusal),
    Transition(RelationalBranchTransitionArtifactFrameRefusal),
}

impl TestDurableMedium {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// A medium publishing to these durable contents, for one runtime to own.
    pub(crate) fn attachment(self: &Arc<Self>) -> Box<dyn RelationalDurableMedium> {
        Box::new(TestDurableMediumAttachment::new(Arc::clone(self)))
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

    /// File a captured native checkpoint under the coverage it records, as a
    /// home's `checkpoint()` does.
    pub(crate) fn file_checkpoint(&self, checkpoint: RelationalNativeCheckpoint) {
        let covered = checkpoint
            .covered_position()
            .expect("a checkpoint this process captured knows its coverage");
        let covered_transitions = checkpoint
            .covered_branch_transitions()
            .expect("a checkpoint this process captured knows its transition coverage");
        self.file_checkpoint_under(covered, covered_transitions, checkpoint);
    }

    /// File checkpoint bytes under an explicit coverage.
    pub(crate) fn file_checkpoint_under(
        &self,
        covered: PatchStreamPosition,
        covered_transitions: RelationalBranchTransitionSequence,
        checkpoint: RelationalNativeCheckpoint,
    ) {
        self.journal().checkpoint = Some(RelationalDurableCheckpointSource::new(
            covered,
            covered_transitions,
            checkpoint,
        ));
    }

    /// What a home would hand Relational on reopen: the filed checkpoint and
    /// every commit and transition after its coverage. A frame-damaged
    /// artifact refuses here, as it would at the home's own read.
    pub(crate) fn recovery_source(
        self: &Arc<Self>,
    ) -> Result<RelationalDurableRecoverySource, TestRecoverySourceRefusal> {
        let journal = self.journal();
        let checkpoint = journal.checkpoint.clone();
        let covered = checkpoint.as_ref().map_or(
            PatchStreamPosition(0),
            RelationalDurableCheckpointSource::covered,
        );
        let covered_transitions = checkpoint.as_ref().map_or(
            RelationalBranchTransitionSequence(0),
            RelationalDurableCheckpointSource::covered_transitions,
        );
        let tail = journal
            .artifacts_after(covered)
            .map(|bytes| RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.to_vec()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(TestRecoverySourceRefusal::Commit)?;
        let transitions = self
            .transition_journal()
            .artifacts_after(covered_transitions)
            .map(|bytes| RelationalBranchTransitionArtifact::from_untrusted_bytes(bytes.to_vec()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(TestRecoverySourceRefusal::Transition)?;
        Ok(RelationalDurableRecoverySource::new(
            checkpoint,
            tail,
            transitions,
            self.clone(),
        ))
    }

    /// Answer the next commit publication as scripted instead of by position.
    pub(crate) fn script_next_publication(&self, answer: ScriptedPublication) {
        self.journal().scripted.push_back(answer);
    }

    fn journal(&self) -> MutexGuard<'_, TestDurableJournal> {
        self.journal
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn transition_journal(&self) -> MutexGuard<'_, TestTransitionJournal> {
        self.transitions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Answer one commit publication: as scripted, otherwise by position.
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

    /// Answer one transition publication: as scripted, otherwise by sequence.
    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> RelationalDurablePublication {
        self.transition_journal().publish(artifact)
    }
}

mod cold_source;
