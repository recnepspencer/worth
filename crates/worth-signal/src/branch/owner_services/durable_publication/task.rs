//! A request can stop while its original owner task and callback result remain retained.
use super::super::SignalDurableBranchHandle;
use super::SignalDurableOwnerEffect;
use crate::durability::{
    SignalAuthoredImageDenial, SignalBranchAdvanceArtifact, SignalDurableFrameDenial,
    SignalDurableMediumDenial, SignalOwnerReadmissionDenial,
};
use std::sync::Weak;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{DurablePublicationFate, SignalBranchAdvanceIdentity};

#[derive(Debug)]
pub enum SignalDurableAdvanceDenial {
    CompositePublicationRequired,
    CheckpointCaptureInProgress {
        owner: worth_foundational::facade::DurableSignalOwnerIdentity,
    },
    CheckpointOriginalsPending {
        originals: worth_execution::ExecutionArray<SignalBranchAdvanceIdentity>,
    },
    CompositeOwnerNotAttached,
    CompositeOwnerAlreadyAttached,
    CompositeAttachmentPending {
        originals: worth_execution::ExecutionArray<SignalBranchAdvanceIdentity>,
    },
    CompositeAdmission(worth_proof::CompositeSignalPublicationAdmissionDenial),
    Backing(worth_execution::ExecutionRequestArrayDenial),
    OwnerUnavailable,
    PublicationStatePoisoned,
    Source(super::super::durable_source_port::SignalDurableSourceDenial),
    Readmission(SignalOwnerReadmissionDenial),
    Mutation(crate::branch::SignalBranchAdvanceDenial),
    Fork(crate::branch::SignalBranchForkOperationDenial),
    Frame(SignalDurableFrameDenial),
    Image(SignalAuthoredImageDenial),
    Medium(SignalDurableMediumDenial),
    Request(worth_execution::WorkCeilingDenial),
    BranchReserved {
        original: SignalBranchAdvanceIdentity,
    },
    TaskExecuting {
        original: SignalBranchAdvanceIdentity,
    },
    TaskDriving {
        original: SignalBranchAdvanceIdentity,
    },
    TaskCompleting {
        original: SignalBranchAdvanceIdentity,
    },
    CompletionDelivered {
        original: SignalBranchAdvanceIdentity,
    },
    UnknownTask {
        original: SignalBranchAdvanceIdentity,
    },
    ForeignTask,
    PublicationUnsettled {
        original: SignalBranchAdvanceIdentity,
    },
    ConditionalDefinitionPublicationRequired,
    ConditionalDefinitionMismatch,
    ConditionalInstallation(
        super::super::conditional_execution::SignalConditionalInstallationExtensionDenial,
    ),
    ConditionalContract(crate::data::conditional_execution::SignalConditionalContractDenial),
    DefinitionSectionRefused,
    DefinitionSourceMismatch {
        expected: worth_foundational::facade::SignalDefinitionTransitionReference,
        expected_branch: worth_foundational::facade::DurableSignalBranchIdentity,
        observed: worth_foundational::facade::SignalDefinitionTransitionReference,
        observed_branch: worth_foundational::facade::DurableSignalBranchIdentity,
    },
    WrongCommittedSubject {
        expected_journal: worth_foundational::facade::SignalDurableJournal,
        expected_digest: [u8; 32],
        observed: Option<(
            worth_foundational::facade::SignalDurableJournal,
            u64,
            [u8; 32],
        )>,
    },
}
/// This output is constructed only after the concrete committed subject moved
/// into its actual branch cell. Its fields cannot be authored by a caller.
pub struct SignalDurableAdvanceOutput {
    pub(super) projection: worth_foundational::facade::SignalDefinitionProjectionOutcome,
    pub(super) branch: SignalDurableBranchHandle,
    pub(super) effect: SignalDurableOwnerEffect,
    pub(super) evidence: crate::durability::SignalCommittedAdvanceEvidence,
}
impl SignalDurableAdvanceOutput {
    pub fn projection_outcome(
        &self,
    ) -> worth_foundational::facade::SignalDefinitionProjectionOutcome {
        self.projection
    }
    pub fn branch(&self) -> &SignalDurableBranchHandle {
        &self.branch
    }
    pub fn artifact(&self) -> &SignalBranchAdvanceArtifact {
        self.evidence.artifact()
    }
    pub fn evidence(&self) -> &crate::durability::SignalCommittedAdvanceEvidence {
        &self.evidence
    }
    pub fn effect(&self) -> &SignalDurableOwnerEffect {
        &self.effect
    }
    pub fn into_parts(
        self,
    ) -> (
        SignalDurableBranchHandle,
        SignalDurableOwnerEffect,
        crate::durability::SignalCommittedAdvanceEvidence,
    ) {
        (self.branch, self.effect, self.evidence)
    }
}
enum SignalDurableAdvanceTerminal {
    Committed(SignalDurableAdvanceOutput),
    ProvenNoEffect(SignalDurableAdvanceDenial),
    Indeterminate(SignalDurableAdvanceDenial),
}
pub struct SignalDurableAdvanceCompletion {
    original: SignalBranchAdvanceIdentity,
    terminal: SignalDurableAdvanceTerminal,
}
impl SignalDurableAdvanceCompletion {
    pub(super) fn committed(
        original: SignalBranchAdvanceIdentity,
        output: SignalDurableAdvanceOutput,
    ) -> Self {
        Self {
            original,
            terminal: SignalDurableAdvanceTerminal::Committed(output),
        }
    }
    pub(super) fn proven_no_effect(
        original: SignalBranchAdvanceIdentity,
        detail: SignalDurableAdvanceDenial,
    ) -> Self {
        Self {
            original,
            terminal: SignalDurableAdvanceTerminal::ProvenNoEffect(detail),
        }
    }
    pub(super) fn indeterminate(
        original: SignalBranchAdvanceIdentity,
        detail: SignalDurableAdvanceDenial,
    ) -> Self {
        Self {
            original,
            terminal: SignalDurableAdvanceTerminal::Indeterminate(detail),
        }
    }
    pub const fn original(&self) -> SignalBranchAdvanceIdentity {
        self.original
    }
    pub fn fate(&self) -> DurablePublicationFate<&SignalDurableAdvanceDenial> {
        match &self.terminal {
            SignalDurableAdvanceTerminal::Committed(_) => DurablePublicationFate::Committed,
            SignalDurableAdvanceTerminal::ProvenNoEffect(detail) => {
                DurablePublicationFate::ProvenNoEffect { detail }
            }
            SignalDurableAdvanceTerminal::Indeterminate(detail) => {
                DurablePublicationFate::Indeterminate { detail }
            }
        }
    }
    pub fn into_committed(self) -> Result<SignalDurableAdvanceOutput, Self> {
        match self.terminal {
            SignalDurableAdvanceTerminal::Committed(output) => Ok(output),
            terminal @ (SignalDurableAdvanceTerminal::ProvenNoEffect(_)
            | SignalDurableAdvanceTerminal::Indeterminate(_)) => Err(Self {
                original: self.original,
                terminal,
            }),
        }
    }
}
pub(super) enum SignalDurableAdvanceAnswer {
    Pending,
    Finished(SignalDurableAdvanceCompletion),
}
pub(super) trait SignalDurableAdvanceDriver: Send + Sync {
    fn drive_original(
        &self,
        original: SignalBranchAdvanceIdentity,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalDurableAdvanceAnswer, SignalDurableAdvanceDenial>;
}
#[must_use = "drive the original task to a terminal answer; dropping this handle does not cancel its owner task"]
pub struct SignalDurableAdvanceTask {
    pub(super) original: SignalBranchAdvanceIdentity,
    pub(super) driver: Weak<dyn SignalDurableAdvanceDriver>,
}
pub enum SignalDurableAdvanceProgress {
    Pending(SignalDurableAdvanceTask),
    Finished(SignalDurableAdvanceCompletion),
}
pub struct SignalDurableAdvanceDriveRefusal {
    task: SignalDurableAdvanceTask,
    denial: SignalDurableAdvanceDenial,
}
impl SignalDurableAdvanceDriveRefusal {
    pub fn denial(&self) -> &SignalDurableAdvanceDenial {
        &self.denial
    }
    pub fn into_parts(self) -> (SignalDurableAdvanceTask, SignalDurableAdvanceDenial) {
        (self.task, self.denial)
    }
}
impl SignalDurableAdvanceTask {
    pub const fn original(&self) -> SignalBranchAdvanceIdentity {
        self.original
    }
    pub fn drive(
        self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalDurableAdvanceProgress, SignalDurableAdvanceDriveRefusal> {
        let Some(driver) = self.driver.upgrade() else {
            return Err(SignalDurableAdvanceDriveRefusal {
                task: self,
                denial: SignalDurableAdvanceDenial::OwnerUnavailable,
            });
        };
        match driver.drive_original(self.original, execution) {
            Ok(SignalDurableAdvanceAnswer::Pending) => {
                Ok(SignalDurableAdvanceProgress::Pending(self))
            }
            Ok(SignalDurableAdvanceAnswer::Finished(completion)) => {
                Ok(SignalDurableAdvanceProgress::Finished(completion))
            }
            Err(denial) => Err(SignalDurableAdvanceDriveRefusal { task: self, denial }),
        }
    }
}
