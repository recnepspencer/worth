//! Original externally owned medium for public Native durability courts.
//! Typed adapters use Native facades; byte storage is also reused by unit tests.
mod attachment;
pub mod bytes;
mod cold_source;
mod owned_snapshot;
use bytes::{ByteJournal, MemberBytes};
pub use owned_snapshot::{OwnedNativeReadSnapshot, OwnedSnapshotDenial};
use std::sync::{Arc, Mutex, MutexGuard};
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence, durability::*, publication::PatchStreamPosition,
};

#[derive(Debug, Default)]
pub struct TestDurableMedium {
    contents: Mutex<Contents>,
}
#[derive(Debug, Default)]
struct Contents {
    initial_inputs: Option<RelationalSourceInitialInputsArtifact>,
    commits: ByteJournal,
    transitions: ByteJournal,
    members: MemberBytes,
    offered: Vec<RelationalCanonicalCommitArtifact>,
    checkpoint: Option<RelationalDurableCheckpointSource>,
    selected: Option<OpaqueCapturedCheckpointSubject>,
}
impl TestDurableMedium {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    pub fn attachment(self: &Arc<Self>) -> Box<dyn RelationalDurableMedium> {
        Box::new(attachment::Attachment(self.clone()))
    }
    fn lock(&self) -> MutexGuard<'_, Contents> {
        self.contents
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    pub fn offered(&self) -> Vec<RelationalCanonicalCommitArtifact> {
        self.lock().offered.clone()
    }
    pub fn artifact_bytes(&self, position: PatchStreamPosition) -> Option<Box<[u8]>> {
        self.lock().commits.entries.get(&position.0).cloned()
    }
    pub fn flip_artifact_byte(&self, position: PatchStreamPosition, offset: usize) {
        self.lock()
            .commits
            .entries
            .get_mut(&position.0)
            .expect("actual held artifact")[offset] ^= 0xff;
    }
    pub fn drop_artifact(&self, position: PatchStreamPosition) {
        self.lock()
            .commits
            .entries
            .remove(&position.0)
            .expect("actual held artifact");
    }
    pub fn file_checkpoint(&self, checkpoint: RelationalNativeCheckpoint) {
        let mut contents = self.lock();
        for member in checkpoint.members() {
            if let RelationalCheckpointMember::New { locator, bytes } = member {
                contents
                    .members
                    .insert(locator.sha256, locator.length, bytes.clone());
            }
        }
        contents.selected = checkpoint.publication_subject().cloned();
        contents.checkpoint = Some(RelationalDurableCheckpointSource::new(
            checkpoint
                .covered_position()
                .expect("actual captured coverage"),
            checkpoint
                .covered_branch_transitions()
                .expect("actual captured transitions"),
            RelationalNativeCheckpoint::from_untrusted_bytes(checkpoint.bytes().to_vec()),
        ));
    }
    pub fn recovery_source(self: &Arc<Self>) -> RelationalDurableRecoverySource {
        let contents = self.lock();
        let covered = contents
            .checkpoint
            .as_ref()
            .map_or(0, |checkpoint| checkpoint.covered().0);
        let transitions = contents
            .checkpoint
            .as_ref()
            .map_or(0, |checkpoint| checkpoint.covered_transitions().0);
        RelationalDurableRecoverySource::new(
            contents.checkpoint.clone(),
            contents
                .commits
                .entries
                .range(covered.saturating_add(1)..)
                .map(|(_, bytes)| {
                    RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone())
                        .expect("stored actual commit frame")
                })
                .collect(),
            contents
                .transitions
                .entries
                .range(transitions.saturating_add(1)..)
                .map(|(_, bytes)| {
                    RelationalBranchTransitionArtifact::from_untrusted_bytes(bytes.clone())
                        .expect("stored actual transition frame")
                })
                .collect(),
            self.clone(),
        )
    }
}
