//! Original preparing custody across the durable-open / Native-preparation seam.
mod original_program;
use super::{ActiveAttemptCustody, ResolvedExpectedProductHead};
use crate::durability::{
    WorldDurableArtifact, WorldDurableMediumBinding, WorldDurableMediumDenial,
    WorldDurablePublicationStop, WorldOpenAttemptRecord,
};
use crate::identity::CompositePublicationAttemptIdentity;
use std::sync::Arc;
use worth_execution::{ExecutionArray, ExecutionRequest};
mod native_preparation;
mod signal_preparation;
pub use native_preparation::{
    WorldNativePreparationDenial, WorldNativePreparationRefusal,
    WorldPreparedPublicationWithoutSignal, WorldQueryNativePreparation,
};
pub use signal_preparation::{WorldPreparedPublicationWithSignal, WorldQuerySignalPreparation};
#[derive(Debug)]
pub enum WorldOpenAttemptRefusal {
    TransitionAdmission(worth_proof::HomeProgramTransitionAdmissionDenial),
    NotHomeBound,
    ProgramSourceOwnerRequired,
    ProgramSource(worth_proof::BranchProgramSourceAdmissionDenial),
    Reservation(super::NoEffectCompositePublication),
    Durable(WorldDurableMediumDenial),
    IdentityScope,
}
#[derive(Debug)]
pub enum WorldOpenAttemptOutcome {
    Opened(WorldOpenedPublicationAttempt),
    Pending(WorldPendingOpenAttempt),
    Unresolved(WorldPendingOpenAttempt),
    EvidenceRequired(WorldPendingOpenAttempt),
    Refused(WorldOpenAttemptRefusal),
}
/// The same issued identities and capacities remain installed before any I/O.
/// Dropping observation preserves a submitted original open in its owner catalog.
#[derive(Debug)]
pub struct WorldPendingOpenAttempt {
    pub(crate) original: WorldOpenAttemptOriginal,
}

/// Same identities and capacities, independent of submitted/readmitted phase.
#[derive(Debug)]
pub(crate) struct WorldOpenAttemptOriginal {
    pub(crate) binding: Arc<WorldDurableMediumBinding>,
    pub(crate) durable_custody: Arc<crate::durability::WorldDurableOpenCustody>,
    pub(crate) subject: WorldDurableArtifact,
    pub(crate) association_bytes: ExecutionArray<u8>,
    pub(crate) identity: CompositePublicationAttemptIdentity,
    pub(crate) resolved: ResolvedExpectedProductHead,
    pub(crate) custody: ActiveAttemptCustody,
    pub(crate) deadline: Option<crate::lifecycle::RuntimeWorldInstant>,
}
impl WorldOpenAttemptOriginal {
    pub(crate) fn open_record(&self) -> &WorldOpenAttemptRecord {
        self.subject
            .open_attempt()
            .expect("the opened phase retains an actual original open")
    }
}
impl WorldPendingOpenAttempt {
    /// Transfer this same pending original into the World's retained catalog.
    /// Its durable fate remains unresolved until owner settlement decides it.
    pub fn retain(self) -> crate::recovery::ProductUnpublishedOwnerEffects {
        self.original.custody.retain_open()
    }
    /// Keep this same conditional-definition original as its typed unpublished outcome.
    pub fn retain_conditional_definition(
        self,
    ) -> crate::publication::RuntimeWorldUnpublishedConditionalDefinition {
        crate::publication::RuntimeWorldUnpublishedConditionalDefinition::from_effects(
            self.retain(),
        )
    }
    pub fn unpublished_recovery_handle(&self) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        self.original.custody.unpublished_recovery_handle()
    }
    pub fn advance(mut self, execution: &ExecutionRequest<'_, '_>) -> WorldOpenAttemptOutcome {
        match self
            .original
            .binding
            .publish(&self.original.subject, execution)
        {
            Ok(committed) => {
                self.original
                    .durable_custody
                    .acknowledge(committed.into_witness());
                WorldOpenAttemptOutcome::Opened(WorldOpenedPublicationAttempt {
                    original: self.original,
                })
            }
            Err(WorldDurablePublicationStop::Pending | WorldDurablePublicationStop::Busy) => {
                WorldOpenAttemptOutcome::Pending(self)
            }
            Err(WorldDurablePublicationStop::Unresolved) => {
                WorldOpenAttemptOutcome::Unresolved(self)
            }
            Err(WorldDurablePublicationStop::EvidenceRequired) => {
                WorldOpenAttemptOutcome::EvidenceRequired(self)
            }
            Err(WorldDurablePublicationStop::Refused(cause)) => {
                self.original.custody.retain_durable_open(None);
                WorldOpenAttemptOutcome::Refused(WorldOpenAttemptRefusal::Durable(
                    cause.into_cause(),
                ))
            }
        }
    }
}
/// A committed original World open; the caller may now bind its association to
/// the consumed validated Native proposal. It is not an owner-effect success.
#[derive(Debug)]
pub struct WorldOpenedPublicationAttempt {
    pub(crate) original: WorldOpenAttemptOriginal,
}
impl WorldOpenedPublicationAttempt {
    pub fn reserved_commit_identity(&self) -> crate::identity::CompositeCommitIdentity {
        self.original.custody.reserved_commit_identity()
    }

    pub fn caller_association_bytes(&self) -> &[u8] {
        self.original.association_bytes.elements()
    }
    pub fn attempt_identity(&self) -> &CompositePublicationAttemptIdentity {
        &self.original.identity
    }
    pub fn expected_head(&self) -> &crate::branch::ProductBranchObservation {
        self.original.resolved.expected()
    }
    pub fn unpublished_recovery_handle(&self) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        self.original.custody.unpublished_recovery_handle()
    }
}
