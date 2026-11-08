//! Store-free owner port, also present on wasm32.
use super::{SignalBranchAdvanceArtifact, SignalDefinitionTransitionArtifact};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_foundational::facade::{
    ApplicationBuildIdentity, DurableSignalOwnerIdentity, HomeIncarnation, SignalDurableJournal,
};
mod completion;
pub use completion::SignalPublicationCompletion;

#[derive(Clone, Debug)]
pub struct SignalDurableHomeBinding {
    owner: DurableSignalOwnerIdentity,
    incarnation: HomeIncarnation,
    program_revision: [u8; 32],
    build: ApplicationBuildIdentity,
}
impl SignalDurableHomeBinding {
    /// Trusted admitted adapter data, never a filesystem attachment witness.
    pub const fn from_admitted_adapter(
        owner: DurableSignalOwnerIdentity,
        incarnation: HomeIncarnation,
        program_revision: [u8; 32],
        build: ApplicationBuildIdentity,
    ) -> Self {
        Self {
            owner,
            incarnation,
            program_revision,
            build,
        }
    }
    pub const fn owner(&self) -> DurableSignalOwnerIdentity {
        self.owner
    }
    pub const fn incarnation(&self) -> HomeIncarnation {
        self.incarnation
    }
    pub const fn program_revision(&self) -> [u8; 32] {
        self.program_revision
    }
    pub const fn build_identity(&self) -> ApplicationBuildIdentity {
        self.build
    }
}
#[derive(Debug)]
pub enum SignalDurableMediumDenial {
    Request(worth_execution::WorkCeilingDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
    Frame(super::SignalDurableFrameDenial),
    ForeignOwner,
    HomeIncarnation(worth_proof::HomeIncarnationDenial),
    OwnerAlreadyReadmitted(super::SignalLiveOwnerIdentity),
    OwnerAdmissionBusy,
    OwnerAdmissionPoisoned,
    OwnerAdmissionCapacity,
    Unavailable,
    PublicationBusy,
    PublicationUnresolved,
    PublicationRetryOwnerMismatch {
        expected: super::SignalLiveOwnerIdentity,
        observed: Option<super::SignalLiveOwnerIdentity>,
    },
    PublicationRetrySubjectMismatch {
        expected_journal: SignalDurableJournal,
        expected_digest: [u8; 32],
        observed_journal: SignalDurableJournal,
        observed_digest: [u8; 32],
    },
    CheckpointBounds(Box<super::SignalOwnerReadmissionDenial>),
    CheckpointSourceBehind {
        journal: SignalDurableJournal,
        checkpoint: u64,
        observed: u64,
    },
    Storage(String),
}
#[derive(Debug)]
pub enum SignalPublicationObservation {
    Pending,
    Finished(SignalPublicationCompletion),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignalPublicationTaskPosture {
    Busy,
    Unsettled,
    Terminal,
}

/// Original protected source shared by both Signal journals. Reads cannot move
/// to a later physical root partway through owner reconstruction.
pub trait SignalDurableReadSource: std::fmt::Debug + Send + Sync {
    fn owner(&self) -> DurableSignalOwnerIdentity;
    fn identity_contribution(&self) -> super::SignalRecordedIdentityContribution;
    fn frontier(
        &self,
        journal: SignalDurableJournal,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, SignalDurableMediumDenial>;
    fn read(
        &self,
        journal: SignalDurableJournal,
        position: u64,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, SignalDurableMediumDenial>;
}
/// Owner artifacts are opaque to the adapter. Repeated calls advance the SAME
/// original publication; an unresolved task survives the invoking request.
pub trait SignalDurableMedium: std::fmt::Debug + Send + Sync {
    fn binding(&self) -> &SignalDurableHomeBinding;
    /// The same actual registry must be returned for this medium's lifetime.
    fn owner_admission_registry(&self) -> &super::SignalDurableOwnerAdmissionRegistry;
    fn publication_task_posture(&self) -> SignalPublicationTaskPosture;
    fn capture_recovery_source(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn SignalDurableReadSource>, SignalDurableMediumDenial>;
    fn advance_definition(
        &self,
        artifact: &SignalDefinitionTransitionArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation;
    fn advance_branch(
        &self,
        artifact: &SignalBranchAdvanceArtifact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation;
    /// Explicitly retry the unchanged logical frame after an actual no-effect
    /// answer. Ordinary advance keeps its terminal answer. The adapter checks
    /// this concrete proof AND its actual current task before retiring any
    /// physical custody; committed, pending and unresolved tasks stay original.
    fn retry_definition_after_no_effect(
        &self,
        artifact: &SignalDefinitionTransitionArtifact,
        proof: &worth_proof::ProvenNoEffectPublication<super::SignalNoEffectOriginal>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation;
    fn retry_branch_after_no_effect(
        &self,
        artifact: &SignalBranchAdvanceArtifact,
        proof: &worth_proof::ProvenNoEffectPublication<super::SignalNoEffectOriginal>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> SignalPublicationObservation;
    /// Quiesced owner teardown releases this exact serving share. It refuses
    /// unsettled physical work; it never infers absence from a stopped request.
    fn retire(&self, execution: &ExecutionRequest<'_, '_>)
        -> Result<(), SignalDurableMediumDenial>;
}
