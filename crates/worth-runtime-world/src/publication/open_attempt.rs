//! Original preparing custody across the durable-open / Native-preparation seam.
use super::{ActiveAttemptCustody, ResolvedExpectedProductHead};
use crate::durability::{
    WorldDurableArtifact, WorldDurableMediumBinding, WorldDurableMediumDenial,
    WorldDurablePublicationStop, WorldOpenAttemptRecord,
};
use crate::identity::CompositePublicationAttemptIdentity;
use std::sync::Arc;
use worth_execution::{ExecutionArray, ExecutionRequest};
mod native_preparation;
pub use native_preparation::{
    WorldNativePreparationDenial, WorldNativePreparationRefusal,
    WorldPreparedPublicationWithoutSignal,
};
#[derive(Debug)]
pub enum WorldOpenAttemptRefusal {
    NotHomeBound,
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
    pub(crate) binding: Arc<WorldDurableMediumBinding>,
    pub(crate) durable_custody: Arc<crate::durability::WorldDurableOpenCustody>,
    pub(crate) subject: WorldDurableArtifact,
    pub(crate) record: WorldOpenAttemptRecord,
    pub(crate) association_bytes: ExecutionArray<u8>,
    pub(crate) identity: CompositePublicationAttemptIdentity,
    pub(crate) resolved: ResolvedExpectedProductHead,
    pub(crate) custody: ActiveAttemptCustody,
    pub(crate) deadline: Option<crate::lifecycle::RuntimeWorldInstant>,
}
impl WorldPendingOpenAttempt {
    pub fn advance(mut self, execution: &ExecutionRequest<'_, '_>) -> WorldOpenAttemptOutcome {
        match self.binding.publish(&self.subject, execution) {
            Ok(committed) => {
                self.durable_custody.acknowledge(committed);
                WorldOpenAttemptOutcome::Opened(WorldOpenedPublicationAttempt { original: self })
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
                self.custody.retain_durable_open(None);
                WorldOpenAttemptOutcome::Refused(WorldOpenAttemptRefusal::Durable(cause))
            }
        }
    }
}
/// A committed original World open; the caller may now bind its association to
/// the consumed validated Native proposal. It is not an owner-effect success.
#[derive(Debug)]
pub struct WorldOpenedPublicationAttempt {
    pub(crate) original: WorldPendingOpenAttempt,
}
impl WorldOpenedPublicationAttempt {
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
