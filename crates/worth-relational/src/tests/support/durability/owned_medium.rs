use std::any::Any;
use std::collections::{BTreeSet, VecDeque};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, MutexGuard};

use worth_testkit_relational::durable_medium::{
    bytes::{ByteJournal, MemberBytes},
    OwnedNativeReadSnapshot, OwnedSnapshotDenial,
};

use crate::durability::branch_transition_artifact::{
    RelationalBranchTransitionArtifact as TransArt,
    RelationalBranchTransitionArtifactDigest as TransDigest,
    RelationalBranchTransitionArtifactFrameRefusal as TransFrameRefusal,
};
use crate::durability::canonical_commit_artifact::{
    RelationalCanonicalCommitArtifact as CommitArt,
    RelationalCanonicalCommitArtifactDigest as CommitDigest,
    RelationalCanonicalCommitArtifactFrameRefusal as CommitFrameRefusal,
};
use crate::durability::data::{
    RelationalCheckpointMember, RelationalNativeCheckpoint as NativeCheckpoint,
};
use crate::durability::medium::{
    DurablePublicationFate as Fate, RelationalDurableOwnedReadRefusal as ReadRefusal,
};
use crate::durability::published_medium::{
    OpaqueCapturedCheckpointSubject as CheckpointSubject,
    RelationalDurableCheckpointSource as CheckpointSrc, RelationalDurableMedium as DurableMedium,
    RelationalDurableRecoverySource as RecoverySrc,
};
use crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact as InitArt;
use crate::history::data::CommitId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as StorageCause;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScriptedPublication {
    ProvenNoEffect,
    IndeterminateWithoutEffect,
    IndeterminateAfterEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OfferedCommitMeta {
    position: PatchStreamPosition,
    commit_id: CommitId,
    payload_digest: CommitDigest,
}

impl OfferedCommitMeta {
    pub(crate) fn position(&self) -> PatchStreamPosition {
        self.position
    }
    pub(crate) fn commit_id(&self) -> CommitId {
        self.commit_id
    }
    pub(crate) fn payload_digest(&self) -> CommitDigest {
        self.payload_digest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OfferedTransitionMeta {
    follows: PatchStreamPosition,
    payload_digest: TransDigest,
}

impl OfferedTransitionMeta {
    pub(crate) fn follows(&self) -> PatchStreamPosition {
        self.follows
    }
    pub(crate) fn payload_digest(&self) -> TransDigest {
        self.payload_digest
    }
}

#[derive(Debug)]
pub(crate) enum TestRecoverySourceRefusal {
    Commit(CommitFrameRefusal),
    Transition(TransFrameRefusal),
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
    pub(super) scripted_transitions: VecDeque<ScriptedPublication>,
    pub(super) offered_commits: Vec<OfferedCommitMeta>,
    pub(super) offered_transitions: Vec<OfferedTransitionMeta>,
    pub(super) retain_transition_wires: bool,
    pub(super) retained_transition_wires: Vec<TransArt>,
    pub(super) forbidden_members: BTreeSet<[u8; 32]>,
    pub(super) commit_index_page_digests: BTreeSet<[u8; 32]>,
    pub(super) member_reads: Vec<[u8; 32]>,
}

/// Explicitly owned in-crate durable medium reusing `worth-testkit-relational`'s
/// byte journals and immutable read snapshots without any runtime-discovery map.
#[derive(Debug, Default)]
pub(crate) struct OwnedTestDurableMedium {
    contents: Mutex<OwnedMediumContents>,
    pub(super) observe_member_reads: AtomicBool,
    pub(super) has_forbidden_members: AtomicBool,
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
    pub(super) fn commit_bytes_at(&self, position: PatchStreamPosition) -> Option<Box<[u8]>> {
        self.lock().commits.entries.get(&position.0).cloned()
    }
    pub(super) fn replace_commit_bytes(&self, position: PatchStreamPosition, bytes: Box<[u8]>) {
        self.lock().commits.entries.insert(position.0, bytes);
    }
    pub(crate) fn offered_count(&self) -> usize {
        self.lock().offered_commits.len()
    }
    pub(crate) fn offered_commits(&self) -> Vec<OfferedCommitMeta> {
        self.lock().offered_commits.clone()
    }
    pub(crate) fn offered_transitions(&self) -> Vec<OfferedTransitionMeta> {
        self.lock().offered_transitions.clone()
    }
    pub(crate) fn head(&self) -> PatchStreamPosition {
        PatchStreamPosition(
            self.lock()
                .commits
                .entries
                .keys()
                .next_back()
                .copied()
                .unwrap_or(0),
        )
    }
    pub(crate) fn positions(&self) -> Vec<PatchStreamPosition> {
        self.lock()
            .commits
            .entries
            .keys()
            .copied()
            .map(PatchStreamPosition)
            .collect()
    }
    pub(crate) fn artifact_bytes(&self, position: PatchStreamPosition) -> Option<Box<[u8]>> {
        self.lock().commits.entries.get(&position.0).cloned()
    }
    pub(crate) fn script_next_publication(&self, answer: ScriptedPublication) {
        self.lock().scripted.push_back(answer);
    }
    pub(crate) fn script_next_transition(&self, answer: ScriptedPublication) {
        self.lock().scripted_transitions.push_back(answer);
    }
    pub(super) fn publish_commit(&self, artifact: &CommitArt) -> Fate<String> {
        let mut contents = self.lock();
        contents.offered_commits.push(OfferedCommitMeta {
            position: artifact.position(),
            commit_id: artifact.commit_id(),
            payload_digest: artifact.payload_digest(),
        });
        if let Some(scripted) = contents.scripted.pop_front() {
            return apply_scripted(
                scripted,
                || {
                    contents
                        .commits
                        .publish(artifact.position().0, artifact.as_bytes())
                },
                "commit",
            );
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
    pub(super) fn publish_transition(&self, artifact: &TransArt) -> Fate<String> {
        let mut contents = self.lock();
        contents.offered_transitions.push(OfferedTransitionMeta {
            follows: artifact.follows(),
            payload_digest: artifact.payload_digest(),
        });
        if contents.retain_transition_wires {
            contents.retained_transition_wires.push(artifact.clone());
        }
        if let Some(scripted) = contents.scripted_transitions.pop_front() {
            return apply_scripted(
                scripted,
                || {
                    contents
                        .transitions
                        .publish(artifact.sequence().0, artifact.as_bytes())
                },
                "transition",
            );
        }
        if contents
            .transitions
            .publish(artifact.sequence().0, artifact.as_bytes())
        {
            Fate::Committed
        } else {
            Fate::ProvenNoEffect {
                detail: "not next or identical original transition".into(),
            }
        }
    }
    pub(crate) fn file_checkpoint(&self, checkpoint: NativeCheckpoint) {
        let mut contents = self.lock();
        for member in checkpoint.members() {
            if let RelationalCheckpointMember::New { locator, bytes } = member {
                if bytes
                    .strip_prefix(b"WORTH-REL-CHUNK1\n")
                    .and_then(|rest| rest.first())
                    == Some(&4)
                {
                    contents.commit_index_page_digests.insert(locator.sha256);
                }
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
    pub(crate) fn try_recovery_source(
        self: &Arc<Self>,
    ) -> Result<RecoverySrc, TestRecoverySourceRefusal> {
        let contents = self.lock();
        let covered = contents.checkpoint.as_ref().map_or(0, |c| c.covered().0);
        let transitions = contents
            .checkpoint
            .as_ref()
            .map_or(0, |c| c.covered_transitions().0);
        let tail = contents
            .commits
            .entries
            .range(covered.saturating_add(1)..)
            .map(|(_, b)| {
                CommitArt::from_untrusted_bytes(b.clone())
                    .map_err(TestRecoverySourceRefusal::Commit)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let transition_tail = contents
            .transitions
            .entries
            .range(transitions.saturating_add(1)..)
            .map(|(_, b)| {
                TransArt::from_untrusted_bytes(b.clone())
                    .map_err(TestRecoverySourceRefusal::Transition)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RecoverySrc::new(
            contents.checkpoint.clone(),
            tail,
            transition_tail,
            self.clone(),
        ))
    }
    pub(crate) fn recovery_source(self: &Arc<Self>) -> RecoverySrc {
        self.try_recovery_source()
            .expect("valid test medium frames")
    }
}

fn apply_scripted(
    scripted: ScriptedPublication,
    append: impl FnOnce() -> bool,
    kind: &str,
) -> Fate<String> {
    match scripted {
        ScriptedPublication::ProvenNoEffect => Fate::ProvenNoEffect {
            detail: format!("scripted: the {kind} publication had no durable effect"),
        },
        ScriptedPublication::IndeterminateWithoutEffect => Fate::Indeterminate {
            detail: format!("scripted: the {kind} append's fate is unknown (not appended)"),
        },
        ScriptedPublication::IndeterminateAfterEffect => {
            assert!(
                append(),
                "scripted indeterminate-after-effect must append cleanly"
            );
            Fate::Indeterminate {
                detail: format!("scripted: the {kind} append's fate is unknown (appended)"),
            }
        }
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
