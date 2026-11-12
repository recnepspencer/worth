use super::{WorldDurableArtifact, WorldDurableFrameDenial};
use std::fmt::Debug;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_foundational::facade::{DurableWorldOwnerIdentity, HomeIncarnation};
mod completion;
pub use completion::WorldPublicationCompletion;

#[derive(Clone, Debug)]
pub struct WorldDurableHomeBinding {
    owner: DurableWorldOwnerIdentity,
    incarnation: HomeIncarnation,
    program_revision: [u8; 32],
    transition: Option<worth_proof::HomeProgramTransitionAuthority>,
    program_source: Option<worth_proof::InstalledBranchProgramSourceAuthority>,
    transition_attempt_frontier: Option<u64>,
    pub(in crate::durability) lineage: Option<std::sync::Arc<super::WorldHomeProgramLineage>>,
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
            transition: None,
            program_source: None,
            transition_attempt_frontier: None,
            lineage: None,
        }
    }
    /// Installed by the actual Home program-source owner, never a per-request verifier.
    pub fn with_branch_program_source_authority(
        mut self,
        authority: worth_proof::InstalledBranchProgramSourceAuthority,
    ) -> Self {
        self.program_source = Some(authority);
        self
    }
    pub(crate) fn branch_program_source_authority(
        &self,
    ) -> Option<&worth_proof::InstalledBranchProgramSourceAuthority> {
        self.program_source.as_ref()
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

    pub(crate) fn prepare_program_history(
        mut self,
        steps: impl ExactSizeIterator<
            Item = worth_foundational::facade::HomeProgramTransitionProgramStep,
        >,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::WorldReadmissionDenial> {
        let lineage = self
            .lineage
            .take()
            .ok_or(super::WorldReadmissionDenial::ProgramRevisionMismatch)?;
        self.lineage = Some(lineage.with_program_steps(steps, request)?);
        Ok(self)
    }

    pub(crate) fn successor(
        &self,
        program_revision: [u8; 32],
        through: worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::WorldReadmissionDenial> {
        if through.owner() != self.owner {
            return Err(super::WorldReadmissionDenial::ForeignOwner);
        }
        let boundary = worth_foundational::facade::HomeProgramRevisionBoundary::from_recorded(
            worth_foundational::facade::CanonicalDigestId::new(self.program_revision),
            worth_foundational::facade::CanonicalDigestId::new(program_revision),
            through,
        );
        let lineage =
            super::WorldHomeProgramLineage::extend(self.lineage.as_deref(), boundary, request)?;
        let mut successor = self.clone();
        successor.program_revision = program_revision;
        successor.lineage = Some(lineage);
        Ok(successor)
    }

    /// Source DATA from the protected Home intent chain. This grants neither
    /// a World writer nor permission to fabricate an original publication.
    pub fn with_recorded_program_lineage(
        mut self,
        lineage: std::sync::Arc<super::WorldHomeProgramLineage>,
    ) -> Result<Self, super::WorldReadmissionDenial> {
        if lineage.owner() != self.owner || lineage.current().bytes() != &self.program_revision {
            return Err(super::WorldReadmissionDenial::ProgramRevisionMismatch);
        }
        self.lineage = Some(lineage);
        Ok(self)
    }

    pub(crate) fn admits_checkpoint_revision(
        &self,
        revision: [u8; 32],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<bool, super::WorldReadmissionDenial> {
        match &self.lineage {
            None => Ok(revision == self.program_revision),
            Some(lineage) => lineage.admits_checkpoint(revision, request),
        }
    }

    pub(crate) fn admits_root_revision(&self, revision: [u8; 32]) -> bool {
        self.lineage
            .as_ref()
            .map_or(self.program_revision == revision, |lineage| {
                lineage.initial().bytes() == &revision
            })
    }

    /// The actual Home composition attaches this verifier once. An individual
    /// transition caller cannot substitute its own admission authority.
    pub fn with_home_transition_authority(
        mut self,
        transition: worth_proof::HomeProgramTransitionAuthority,
    ) -> Self {
        self.transition = Some(transition);
        self
    }

    /// Supplemental original reservations admitted from the actual Home intent journal.
    /// This does not replace or rewrite checkpoint/source frontier facts.
    pub fn with_home_transition_attempt_frontier(mut self, frontier: Option<u64>) -> Self {
        self.transition_attempt_frontier = frontier;
        self
    }
    pub(crate) fn transition_attempt_frontier(&self) -> Option<u64> {
        self.transition_attempt_frontier
    }

    pub(crate) fn transition_authority(
        &self,
    ) -> Option<&worth_proof::HomeProgramTransitionAuthority> {
        self.transition.as_ref()
    }
}
#[derive(Debug)]
pub enum WorldDurableMediumDenial {
    NameAllocation,
    OccurrenceReopenRequired,
    Allocation(worth_execution::ExecutionAllocationDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
    Ordered(worth_execution::ExecutionOrderedMapDenial),
    Request(WorkCeilingDenial),
    Frame(WorldDurableFrameDenial),
    NativeDescription(Box<worth_relational::facade::branch::RelationalRecordedComponentRefusal>),
    Unavailable,
    PublicationBusy,
    PublicationUnresolved,
    StaleCheckpoint,
    CheckpointSelectionRequired,
    CheckpointOriginRetention,
    Storage(String),
}
impl From<WorldDurableFrameDenial> for WorldDurableMediumDenial {
    fn from(cause: WorldDurableFrameDenial) -> Self {
        Self::Frame(cause)
    }
}
#[derive(Debug)]
pub enum WorldPublicationObservation {
    Pending,
    /// Selection is known, but the actual owner-issued committed carrier has
    /// not been readmitted. This does not describe an unknown physical fate.
    EvidenceRequired,
    Finished(WorldPublicationCompletion),
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
    /// Retire this exact adapter's physical serving share without waiting or
    /// destroying its original allocation. The Home's Native/physical keepers
    /// remain borrowed throughout; a stopped request preserves active custody.
    fn retire_for_home(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), WorldDurableMediumDenial>;

    /// Acquire one actual protected physical W root without payload I/O.
    /// Called only while the actual durable publication reservation is held.
    /// The same original request funds metadata and nonwaiting admission.
    fn capture_checkpoint_source(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn super::checkpoint::WorldCheckpointReadSource>, WorldDurableMediumDenial>;
    /// Query/advance the actual retained publication for this original subject.
    /// Implementations must never pair a caller-supplied fate with another receipt.
    fn advance_publication(
        &self,
        subject: &WorldDurableArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> WorldPublicationObservation;
}
