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
    pub(crate) branches: Vec<Vec<u8>>,
    pub(crate) occurrences: Vec<Vec<u8>>,
    pub(crate) custody: Vec<Vec<u8>>,
}
impl WorldDurableRecoverySource {
    /// Untrusted owner bytes. Only World readmission can turn them into facts.
    pub fn from_owner_bytes(
        branches: Vec<Vec<u8>>,
        occurrences: Vec<Vec<u8>>,
        custody: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            branches,
            occurrences,
            custody,
        }
    }
    pub(crate) fn stream(&self, journal: WorldDurableJournal) -> &[Vec<u8>] {
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
