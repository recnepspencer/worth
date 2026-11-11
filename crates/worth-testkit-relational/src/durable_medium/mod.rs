//! Original externally owned medium for public Native durability courts.
//! Typed adapters use Native facades; byte storage is also reused by unit tests.
mod attachment;
pub mod changed_member;
pub mod member_placement;
mod scripted_publication;
pub use scripted_publication::ScriptedPublication;
pub mod bytes;
mod cold_source;
pub mod immutable_members;
mod member_faults;
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
    member_cancellation: Mutex<Option<worth_execution::CancellationSource>>,
    canonical_member_reads: std::sync::atomic::AtomicU64,
    canonical_journal_reads: std::sync::atomic::AtomicU64,
    member_faults: Mutex<member_faults::MemberFaultState>,
}
#[derive(Debug, Default)]
struct Contents {
    initial_inputs: Option<RelationalSourceInitialInputsArtifact>,
    commits: ByteJournal,
    transitions: ByteJournal,
    members: MemberBytes,
    offered: Vec<RelationalCanonicalCommitArtifact>,
    offered_transitions: Vec<RelationalBranchTransitionArtifact>,
    scripted: std::collections::VecDeque<ScriptedPublication>,
    scripted_transitions: std::collections::VecDeque<ScriptedPublication>,
    checkpoint: Option<RelationalDurableCheckpointSource>,
    selected: Option<OpaqueCapturedCheckpointSubject>,
}
impl TestDurableMedium {
    pub fn cancel_next_member_read_for_test(
        &self,
        cancellation: worth_execution::CancellationSource,
    ) {
        let mut armed = self.member_cancellation.lock().unwrap();
        assert!(armed.is_none());
        *armed = Some(cancellation);
    }
    pub fn canonical_journal_reads(&self) -> u64 {
        self.canonical_journal_reads
            .load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn canonical_member_reads(&self) -> u64 {
        self.canonical_member_reads
            .load(std::sync::atomic::Ordering::Relaxed)
    }
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
    pub fn script_next_publication(&self, answer: ScriptedPublication) {
        self.lock().scripted.push_back(answer);
    }
    pub fn script_next_transition(&self, answer: ScriptedPublication) {
        self.lock().scripted_transitions.push_back(answer);
    }
    pub fn offered_transitions(&self) -> Vec<RelationalBranchTransitionArtifact> {
        self.lock().offered_transitions.clone()
    }
    pub fn committed_artifacts(
        &self,
    ) -> Result<Vec<RelationalCanonicalCommitArtifact>, RelationalCanonicalCommitArtifactFrameRefusal>
    {
        self.lock()
            .commits
            .entries
            .values()
            .map(|bytes| RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()))
            .collect()
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
