//! The journal model pins sealed publications or explicitly untrusted DATA cuts.
use super::*;
mod mutated_data;
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use worth_execution::ExecutionRequestArrayBuilder;

#[derive(Clone, Debug)]
struct Stored {
    journal: WorldDurableJournal,
    position: u64,
    digest: [u8; 32],
    bytes: Vec<u8>,
}
#[derive(Debug)]
pub(in super::super) struct Recorder {
    binding: WorldDurableHomeBinding,
    program_source: worth_proof::InstalledBranchProgramSourceOwner,
    pub(in super::super) subjects: Mutex<Vec<WorldDurableArtifact>>,
    rows: Arc<Mutex<Vec<Stored>>>,
    refused: std::sync::atomic::AtomicBool,
    held: std::sync::atomic::AtomicBool,
    persist: bool,
    crashed: std::sync::atomic::AtomicBool,
    surviving: Mutex<Option<u32>>,
    undecided: Mutex<Option<bool>>,
}
impl Recorder {
    /// The next World write reports an indeterminate fate. `landed` decides
    /// whether it actually became durable; the writer cannot tell.
    pub(in super::super) fn undecide_next(&self, landed: bool) {
        *self.undecided.lock().unwrap() = Some(landed);
    }
    /// Process death at the World medium: no later World write becomes
    /// durable. Native keeps whatever its own medium already made durable.
    pub(in super::super) fn crash(&self) {
        self.crashed
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
    /// Process death after `writes` more durable World writes.
    pub(in super::super) fn crash_after(&self, writes: u32) {
        *self.surviving.lock().unwrap() = Some(writes);
    }
    pub(in super::super) fn refuse_next_publication(&self) {
        self.refused
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
    pub(in super::super) fn hold_publication(&self, held: bool) {
        self.held.store(held, std::sync::atomic::Ordering::SeqCst);
    }
    pub(super) fn new(persist: bool) -> Arc<Self> {
        let program_source = worth_proof::InstalledBranchProgramSourceOwner::fresh();
        let authority = program_source.authority();
        Arc::new(Self {
            binding: WorldDurableHomeBinding::from_admitted_adapter(
                DurableWorldOwnerIdentity::from_recorded(
                    HomeIdentity::from_observed_bytes([7; 16]).unwrap(),
                    WorldOwnerSlotIdentity::from_recorded(0),
                ),
                HomeIncarnation::initial(),
                [3; 32],
            )
            .with_branch_program_source_authority(authority),
            program_source,
            subjects: Mutex::new(Vec::new()),
            rows: Arc::default(),
            refused: std::sync::atomic::AtomicBool::new(false),
            held: std::sync::atomic::AtomicBool::new(false),
            persist,
            crashed: std::sync::atomic::AtomicBool::new(false),
            surviving: Mutex::new(None),
            undecided: Mutex::new(None),
        })
    }
    pub(super) fn successor(&self) -> Arc<Self> {
        let program_source = worth_proof::InstalledBranchProgramSourceOwner::fresh();
        let authority = program_source.authority();
        Arc::new(Self {
            binding: WorldDurableHomeBinding::from_admitted_adapter(
                self.binding.owner(),
                self.binding.incarnation().successor().unwrap(),
                self.binding.program_revision(),
            )
            .with_branch_program_source_authority(authority),
            program_source,
            subjects: Mutex::new(Vec::new()),
            rows: self.rows.clone(),
            refused: std::sync::atomic::AtomicBool::new(false),
            held: std::sync::atomic::AtomicBool::new(false),
            persist: self.persist,
            crashed: std::sync::atomic::AtomicBool::new(false),
            surviving: Mutex::new(None),
            undecided: Mutex::new(None),
        })
    }
    /// This model installs one unchanged declared program; it has no adoption table.
    pub(in super::super) fn program_source(
        &self,
        expected: &ProductBranchObservation,
    ) -> worth_proof::AdmittedBranchProgramSource {
        let revision = self.binding.program_revision();
        self.program_source.admit_source(
            revision,
            world_branch_program_source_commitment(expected, revision).unwrap(),
        )
    }
    pub(in super::super) fn heads(&self) -> [u64; 3] {
        heads(&self.rows.lock().unwrap())
    }
    /// Read-only protocol fixture, not a writer or a Native admission.
    pub(in super::super) fn recorded_data_cut(
        selected: worth_execution::ExecutionImmutableBytes,
        tail: worth_execution::ExecutionImmutableBytes,
    ) -> Box<dyn WorldCheckpointReadSource> {
        Box::new(JournalSource {
            selected: Some(selected),
            rows: vec![Stored {
                journal: WorldDurableJournal::Branches,
                position: 4,
                digest: Sha256::digest(tail.as_ref()).into(),
                bytes: tail.as_ref().to_vec(),
            }],
        })
    }
    pub(in super::super) fn data_snapshot(
        &self,
        selected: Option<worth_execution::ExecutionImmutableBytes>,
        mutation: impl FnOnce(&mut Vec<u8>),
    ) -> Box<dyn WorldCheckpointReadSource> {
        let mut rows = self.rows.lock().unwrap().clone();
        if let Some(row) = rows
            .iter_mut()
            .rev()
            .find(|row| row.journal == WorldDurableJournal::Occurrences)
        {
            mutation(&mut row.bytes);
            row.digest = Sha256::digest(&row.bytes).into();
        }
        Box::new(JournalSource { selected, rows })
    }
    /// Untrusted DATA: rewrite the latest stored row of one journal in place,
    /// as damage between processes. `None` removes that row.
    pub(in super::super) fn rewrite_latest(
        &self,
        journal: WorldDurableJournal,
        change: impl FnOnce(Vec<u8>) -> Option<Vec<u8>>,
    ) {
        let mut rows = self.rows.lock().unwrap();
        let index = rows.iter().rposition(|row| row.journal == journal).unwrap();
        match change(rows[index].bytes.clone()) {
            Some(bytes) => {
                rows[index].digest = Sha256::digest(&bytes).into();
                rows[index].bytes = bytes;
            }
            None => {
                rows.remove(index);
            }
        }
    }
    pub(in super::super) fn snapshot_for_court(&self) -> Box<dyn WorldCheckpointReadSource> {
        self.snapshot()
    }
    pub(super) fn snapshot(&self) -> Box<dyn WorldCheckpointReadSource> {
        Box::new(JournalSource {
            selected: None,
            rows: self.rows.lock().unwrap().clone(),
        })
    }
}
impl RuntimeWorldDurableMedium for Recorder {
    fn binding(&self) -> &WorldDurableHomeBinding {
        &self.binding
    }
    fn publication_task_posture(&self) -> WorldPublicationTaskPosture {
        WorldPublicationTaskPosture::Terminal
    }
    fn retire_for_home(
        &self,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldDurableMediumDenial> {
        Ok(())
    }
    fn capture_checkpoint_source(
        &self,
        _: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn WorldCheckpointReadSource>, WorldDurableMediumDenial> {
        Ok(self.snapshot())
    }
    fn advance_publication(
        &self,
        subject: &WorldDurableArtifact,
        _: &ExecutionRequest<'_, '_>,
    ) -> WorldPublicationObservation {
        self.subjects.lock().unwrap().push(subject.clone());
        if self.held.load(std::sync::atomic::Ordering::SeqCst) {
            return WorldPublicationObservation::Pending;
        }
        if let Some(left) = self.surviving.lock().unwrap().as_mut() {
            match left.checked_sub(1) {
                Some(next) => *left = next,
                None => self.crash(),
            }
        }
        let undecided = self.undecided.lock().unwrap().take();
        if undecided == Some(false) {
            return WorldPublicationObservation::Finished(
                WorldPublicationCompletion::indeterminate(WorldDurableMediumDenial::Unavailable),
            );
        }
        if self
            .refused
            .swap(false, std::sync::atomic::Ordering::SeqCst)
            || !self.persist
            || self.crashed.load(std::sync::atomic::Ordering::SeqCst)
        {
            return WorldPublicationObservation::Finished(
                WorldPublicationCompletion::proven_no_effect(WorldDurableMediumDenial::Unavailable),
            );
        }
        let mut rows = self.rows.lock().unwrap();
        let journal = subject.journal();
        let position = rows.iter().filter(|r| r.journal == journal).count() as u64 + 1;
        let digest = Sha256::digest(subject.as_bytes()).into();
        rows.push(Stored {
            journal,
            position,
            digest,
            bytes: subject.as_bytes().to_vec(),
        });
        if undecided == Some(true) {
            return WorldPublicationObservation::Finished(
                WorldPublicationCompletion::indeterminate(WorldDurableMediumDenial::Unavailable),
            );
        }
        WorldPublicationObservation::Finished(WorldPublicationCompletion::committed(
            journal,
            std::num::NonZeroU64::new(position).unwrap(),
            CanonicalDigestId::new(digest),
        ))
    }
}
#[derive(Debug)]
struct JournalSource {
    selected: Option<worth_execution::ExecutionImmutableBytes>,
    rows: Vec<Stored>,
}
impl WorldCheckpointReadSource for JournalSource {
    fn read(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<WorldCheckpointReadView, WorldDurableMediumDenial> {
        request
            .consult()
            .map_err(WorldDurableMediumDenial::Request)?;
        WorldCheckpointReadView::from_protected_home_source(
            self.selected
                .as_ref()
                .map(|bytes| (bytes.clone(), 0..bytes.len())),
            heads(&self.rows),
        )
    }
    fn read_journal_record(
        &self,
        request: &ExecutionRequest<'_, '_>,
        journal: WorldDurableJournal,
        position: u64,
    ) -> Result<WorldCheckpointJournalRecord, WorldDurableMediumDenial> {
        let row = self
            .rows
            .iter()
            .find(|r| r.journal == journal && r.position == position)
            .ok_or(WorldDurableMediumDenial::Unavailable)?;
        let mut bytes = ExecutionRequestArrayBuilder::allocate(*request, row.bytes.len())
            .map_err(WorldDurableMediumDenial::Backing)?;
        bytes
            .extend_from_slice(&row.bytes)
            .map_err(WorldDurableMediumDenial::Backing)?;
        Ok(WorldCheckpointJournalRecord::from_protected_world_journal(
            position,
            row.digest,
            bytes
                .seal()
                .map_err(WorldDurableMediumDenial::Backing)?
                .into_immutable_bytes(),
        ))
    }
    fn observe_selection(
        &self,
        _: &WorldCheckpointSubject,
    ) -> Result<Option<WorldCheckpointSelection>, WorldDurableMediumDenial> {
        Ok(None)
    }
}
fn heads(rows: &[Stored]) -> [u64; 3] {
    let mut result = [0; 3];
    for row in rows {
        result[match row.journal {
            WorldDurableJournal::Branches => 0,
            WorldDurableJournal::Custody => 1,
            WorldDurableJournal::Occurrences => 2,
        }] = row.position;
    }
    result
}
