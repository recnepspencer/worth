use super::{WorldDurableArtifact, WorldDurableFrameDenial};
use std::fmt::Debug;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_foundational::facade::{
    DurablePublicationFate, DurableWorldOwnerIdentity, HomeIncarnation, WorldDurableJournal,
};

#[derive(Clone, Debug)]
pub struct WorldDurableHomeBinding {
    owner: DurableWorldOwnerIdentity,
    incarnation: HomeIncarnation,
    program_revision: [u8; 32],
}
impl WorldDurableHomeBinding {
    /// Data provided by the trusted admitted adapter contract, not physical
    /// authentication. Only the restricted Home composition attaches it.
    pub fn from_admitted_adapter(
        owner: DurableWorldOwnerIdentity,
        incarnation: HomeIncarnation,
        program_revision: [u8; 32],
    ) -> Self {
        Self {
            owner,
            incarnation,
            program_revision,
        }
    }
    pub const fn owner(&self) -> DurableWorldOwnerIdentity {
        self.owner
    }
    pub const fn incarnation(&self) -> HomeIncarnation {
        self.incarnation
    }
    pub const fn program_revision(&self) -> [u8; 32] {
        self.program_revision
    }
}
#[derive(Debug)]
pub enum WorldDurableMediumDenial {
    Allocation(worth_execution::ExecutionAllocationDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
    Request(WorkCeilingDenial),
    Frame(WorldDurableFrameDenial),
    Unavailable,
    PublicationBusy,
    PublicationUnresolved,
    Storage(String),
}
#[derive(Debug)]
pub struct WorldDurableRecoverySource {
    pub(crate) branches: Vec<WorldRecoveredJournalRecord>,
    pub(crate) occurrences: Vec<WorldRecoveredJournalRecord>,
    pub(crate) custody: Vec<WorldRecoveredJournalRecord>,
}
/// Source coordinates supplied by the admitted Home adapter. These values are
/// untrusted input until World validates the owner frame and contiguous stream.
#[derive(Debug, Clone)]
pub struct WorldRecoveredJournalRecord {
    pub(crate) position: u64,
    pub(crate) digest: [u8; 32],
    pub(crate) bytes: Vec<u8>,
}
impl WorldRecoveredJournalRecord {
    pub fn from_owner_record(position: u64, digest: [u8; 32], bytes: Vec<u8>) -> Self {
        Self {
            position,
            digest,
            bytes,
        }
    }
}
impl WorldDurableRecoverySource {
    /// Untrusted owner bytes. Only World readmission can turn them into facts.
    pub fn from_owner_records(
        branches: Vec<WorldRecoveredJournalRecord>,
        occurrences: Vec<WorldRecoveredJournalRecord>,
        custody: Vec<WorldRecoveredJournalRecord>,
    ) -> Self {
        Self {
            branches,
            occurrences,
            custody,
        }
    }
    pub(crate) fn stream(&self, journal: WorldDurableJournal) -> &[WorldRecoveredJournalRecord] {
        match journal {
            WorldDurableJournal::Branches => &self.branches,
            WorldDurableJournal::Occurrences => &self.occurrences,
            WorldDurableJournal::Custody => &self.custody,
        }
    }
}
#[derive(Debug)]
pub enum WorldPublicationObservation {
    Pending,
    /// Selection is known, but the actual owner-issued committed carrier has
    /// not been readmitted. This does not describe an unknown physical fate.
    EvidenceRequired,
    Finished(DurablePublicationFate<WorldDurableMediumDenial>),
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorldPublicationTaskPosture {
    Busy,
    Unsettled,
    Terminal,
}
/// Trusted owner port implemented by Store Application's private adapter.
/// An arbitrary implementation does not authenticate filesystem ownership.
pub trait RuntimeWorldDurableMedium: Debug + Send + Sync {
    fn binding(&self) -> &WorldDurableHomeBinding;
    /// A no-I/O snapshot of the actual retained task slots. Terminal includes
    /// terminal indeterminate custody: it permits teardown, never serving or
    /// a NoEffect conclusion. Home retains those tasks through physical close.
    fn publication_task_posture(&self) -> WorldPublicationTaskPosture;
    fn recover(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<WorldDurableRecoverySource, WorldDurableMediumDenial>;
    /// Query/advance the actual retained publication for this original subject.
    /// Implementations must never pair a caller-supplied fate with another receipt.
    fn advance_publication(
        &self,
        subject: &WorldDurableArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> WorldPublicationObservation;
}
