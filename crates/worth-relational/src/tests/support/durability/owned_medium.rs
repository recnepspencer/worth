use std::any::Any;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

use worth_testkit_relational::durable_medium::{
    bytes::{ByteJournal, MemberBytes},
    OwnedNativeReadSnapshot, OwnedSnapshotDenial,
};

use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact as TransArt;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as CommitArt;
use crate::durability::data::{
    RelationalCheckpointMember, RelationalNativeCheckpoint as NativeCheckpoint,
};
use crate::durability::medium::{
    DurablePublicationFate as Fate, OpaqueCapturedCheckpointSubject as CheckpointSubject,
    RelationalDurableCheckpointSource as CheckpointSrc, RelationalDurableMedium as DurableMedium,
    RelationalDurableOwnedReadRefusal as ReadRefusal,
    RelationalDurableRecoverySource as RecoverySrc,
};
use crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact as InitArt;
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as StorageCause;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScriptedPublication {
    IndeterminateWithoutEffect,
}

#[derive(Debug, Default)]
pub(super) struct OwnedMediumContents {
    pub(super) initial_inputs: Option<InitArt>,
    pub(super) commits: ByteJournal,
    pub(super) transitions: ByteJournal,
    pub(super) members: MemberBytes,
    pub(super) checkpoint: Option<CheckpointSrc>,
    pub(super) selected: Option<CheckpointSubject>,
    pub(super) scripted: VecDeque<ScriptedPublication>,
}

/// Explicitly owned in-crate durable medium reusing `worth-testkit-relational`'s
/// byte journals and immutable read snapshots without any runtime-discovery map.
#[derive(Debug, Default)]
pub(crate) struct OwnedTestDurableMedium {
    contents: Mutex<OwnedMediumContents>,
}

#[derive(Debug)]
pub(super) struct OwnedMediumAttachment(pub(super) Arc<OwnedTestDurableMedium>);

impl OwnedTestDurableMedium {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
    pub(crate) fn attachment(self: &Arc<Self>) -> Box<dyn DurableMedium> {
        Box::new(OwnedMediumAttachment(self.clone()))
    }
    pub(super) fn lock(&self) -> MutexGuard<'_, OwnedMediumContents> {
        self.contents
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    pub(crate) fn committed_count(&self) -> usize {
        self.lock().commits.entries.len()
    }
    pub(crate) fn latest_committed_artifact(&self) -> Option<CommitArt> {
        self.lock()
            .commits
            .entries
            .values()
            .next_back()
            .map(|bytes| CommitArt::from_untrusted_bytes(bytes.clone()).expect("commit frame"))
    }
    pub(crate) fn has_filed_checkpoint(&self) -> bool {
        self.lock().checkpoint.is_some()
    }
    pub(crate) fn flip_checkpoint_byte(&self, offset: usize) {
        let mut contents = self.lock();
        let filed = contents
            .checkpoint
            .take()
            .expect("medium holds a checkpoint");
        let mut bytes = filed.checkpoint().bytes().to_vec();
        bytes[offset] ^= 0xff;
        contents.checkpoint = Some(CheckpointSrc::new(
            filed.covered(),
            filed.covered_transitions(),
            NativeCheckpoint::from_untrusted_bytes(bytes),
        ));
    }
    pub(crate) fn damage_checkpoint_payload(&self) {
        self.flip_checkpoint_byte(
            crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC.len(),
        );
    }
    pub(super) fn commit_bytes_at(&self, position: PatchStreamPosition) -> Option<Box<[u8]>> {
        self.lock().commits.entries.get(&position.0).cloned()
    }
    pub(super) fn replace_commit_bytes(&self, position: PatchStreamPosition, bytes: Box<[u8]>) {
        self.lock().commits.entries.insert(position.0, bytes);
    }
    pub(crate) fn script_next_publication(&self, answer: ScriptedPublication) {
        self.lock().scripted.push_back(answer);
    }
    pub(super) fn publish_commit(&self, artifact: &CommitArt) -> Fate<String> {
        let mut contents = self.lock();
        if let Some(scripted) = contents.scripted.pop_front() {
            return match scripted {
                ScriptedPublication::IndeterminateWithoutEffect => Fate::Indeterminate {
                    detail: "scripted: the append's fate is unknown (not appended)".to_owned(),
                },
            };
        }
        if contents
            .commits
            .publish(artifact.position().0, artifact.as_bytes())
        {
            Fate::Committed
        } else {
            Fate::ProvenNoEffect {
                detail: "not next or identical original position".into(),
            }
        }
    }
    pub(crate) fn file_checkpoint(&self, checkpoint: NativeCheckpoint) {
        let mut contents = self.lock();
        for member in checkpoint.members() {
            if let RelationalCheckpointMember::New { locator, bytes } = member {
                contents
                    .members
                    .insert(locator.sha256, locator.length, bytes.clone());
            }
        }
        contents.selected = checkpoint.publication_subject().cloned();
        contents.checkpoint = Some(CheckpointSrc::new(
            checkpoint.covered_position().expect("captured coverage"),
            checkpoint
                .covered_branch_transitions()
                .expect("captured transitions"),
            NativeCheckpoint::from_untrusted_bytes(checkpoint.bytes().to_vec()),
        ));
    }
    pub(crate) fn recovery_source(self: &Arc<Self>) -> RecoverySrc {
        let contents = self.lock();
        let covered = contents.checkpoint.as_ref().map_or(0, |c| c.covered().0);
        let transitions = contents
            .checkpoint
            .as_ref()
            .map_or(0, |c| c.covered_transitions().0);
        RecoverySrc::new(
            contents.checkpoint.clone(),
            contents
                .commits
                .entries
                .range(covered.saturating_add(1)..)
                .map(|(_, b)| CommitArt::from_untrusted_bytes(b.clone()).expect("commit frame"))
                .collect(),
            contents
                .transitions
                .entries
                .range(transitions.saturating_add(1)..)
                .map(|(_, b)| TransArt::from_untrusted_bytes(b.clone()).expect("transition frame"))
                .collect(),
            self.clone(),
        )
    }
}

pub(super) fn bounded_bytes(bytes: Option<&[u8]>, limit: u64) -> Result<Vec<u8>, StorageCause> {
    let bytes = bytes.ok_or(StorageCause::SourceUnavailable)?;
    if bytes.len() as u64 > limit {
        return Err(StorageCause::LimitExceeded {
            length: bytes.len() as u64,
            limit,
        });
    }
    Ok(bytes.to_vec())
}

pub(super) fn map_owned_denial(cause: OwnedSnapshotDenial) -> ReadRefusal {
    match cause {
        OwnedSnapshotDenial::Request(cause) => ReadRefusal::Request(cause.into()),
        OwnedSnapshotDenial::Buffer(cause) => ReadRefusal::Buffer(cause),
        OwnedSnapshotDenial::Missing => ReadRefusal::Storage(StorageCause::MissingMember),
        OwnedSnapshotDenial::Integrity => ReadRefusal::Storage(StorageCause::IntegrityMismatch),
        OwnedSnapshotDenial::Limit { length, limit } => {
            ReadRefusal::Storage(StorageCause::LimitExceeded { length, limit })
        }
        OwnedSnapshotDenial::Capacity => ReadRefusal::Storage(StorageCause::MemoryExhausted),
    }
}

pub(super) fn downcast_snapshot(
    original: &(dyn Any + Send + Sync),
) -> Result<&OwnedNativeReadSnapshot, ReadRefusal> {
    original
        .downcast_ref::<OwnedNativeReadSnapshot>()
        .ok_or(ReadRefusal::ForeignProtection)
}
