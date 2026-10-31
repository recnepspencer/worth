//! Owner-held custody of an admitted attempt before its terminal is known.
//!
//! The catalog holds the record. A phase holds the one caller capability;
//! losing that capability exposes the existing effects without owner calls.

mod conditional_definition;
mod creation;
mod durable_publication;
pub(crate) use conditional_definition::ConditionalDefinitionAttemptCustody;
#[cfg(test)]
mod creation_rehearsal;
mod head;
mod lease;
mod materialization;
mod movement;
pub(crate) use movement::AttemptProductMovementFailure;
mod operation;
mod record;
mod resources;
mod retention;

use std::sync::Arc;

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::branch::ProductBranchObservation;
use crate::identity::CompositePublicationAttemptIdentity;
use crate::lifecycle::RuntimeWorldInstant;
use crate::recovery::ReservedProductUnpublishedSlot;

use super::{CompositeAttemptProgress, ReservedAttemptCapacities};

use creation::ActiveCreationResources;
pub(crate) use lease::ActiveAttemptResourceLease;
pub(crate) use record::ActiveAttemptRecord;
pub(crate) use resources::ActiveAttemptResources;
use resources::{ActiveHistoryCustody, ActivePinCustody};
pub(crate) use retention::RetainedCommitDisposition;

/// The linear caller capability to one preinstalled owner record. Resource
/// leases borrow this capability and restore their custody before it drops.
pub(crate) struct ActiveAttemptCustody {
    record: Arc<ActiveAttemptRecord>,
    slot: Option<ReservedProductUnpublishedSlot>,
}

impl std::fmt::Debug for ActiveAttemptCustody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveAttemptCustody")
            .field("identity", self.record.identity())
            .finish_non_exhaustive()
    }
}

impl ActiveAttemptCustody {
    pub(crate) fn reserved_commit_identity(&self) -> crate::identity::CompositeCommitIdentity {
        self.record
            .state()
            .resources
            .as_ref()
            .expect("the prepared caller owns its original resources")
            .commit_identity
            .clone()
    }

    pub(crate) fn register(
        attempt: CompositePublicationAttemptIdentity,
        expected: ProductBranchObservation,
        deadline: Option<RuntimeWorldInstant>,
        capacities: ReservedAttemptCapacities,
    ) -> Self {
        let (record, slot) = ActiveAttemptRecord::new(attempt, expected, deadline, capacities);
        slot.register_active(Arc::clone(&record));
        Self {
            record,
            slot: Some(slot),
        }
    }

    pub(crate) fn retain_durable_open(
        &mut self,
        open: Option<Arc<crate::durability::WorldDurableOpenCustody>>,
    ) {
        self.lease_resources().resources_mut().durable_open = open;
    }

    pub(crate) fn has_durable_open(&self) -> bool {
        self.record
            .state()
            .resources
            .as_ref()
            .is_some_and(|resources| resources.durable_open.is_some())
    }

    pub(crate) fn prepare_durable_occurrence(
        &mut self,
        original: &crate::durability::WorldDurableArtifact,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), crate::durability::WorldDurableMediumDenial> {
        let mut lease = self.lease_resources();
        lease.resources_mut().durable_occurrence = Some(
            crate::durability::WorldOccurrenceCustody::prepare(original, execution)?,
        );
        Ok(())
    }

    /// A durable preparing record needs no invented successor or owner effect
    /// to remain recoverable. Convert the original catalog slot directly.
    pub(crate) fn retain_open(mut self) -> crate::recovery::ProductUnpublishedOwnerEffects {
        self.record.state().cause = crate::recovery::ProductUnpublishedCause::OpenAttemptRetained;
        self.slot
            .take()
            .expect("original open owns its slot")
            .retain_active(self.record.identity())
    }

    pub(crate) fn record_progress(
        &mut self,
        progress: CompositeAttemptProgress,
    ) -> Arc<CompositeAttemptProgress> {
        self.record.replace_progress(progress)
    }

    pub(crate) fn record_successor(&mut self, basis: AdmittedCompositeRuntimeWorldBasis) {
        self.record.set_successor(basis);
    }

    pub(crate) fn progress(&self) -> Arc<CompositeAttemptProgress> {
        self.record.progress()
    }

    pub(crate) fn unpublished_recovery_handle(
        &self,
    ) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        self.slot
            .as_ref()
            .expect("a prepared attempt retains its recovery reservation")
            .recovery_handle(self.record.identity())
    }
}

impl Drop for ActiveAttemptCustody {
    fn drop(&mut self) {
        let Some(slot) = self.slot.take() else { return };
        if self.record.product_moved()
            || (self.record.progress().owner_effect_count() == 0 && !self.has_durable_open())
        {
            slot.remove_active(self.record.identity());
            return;
        }
        slot.abandon_active(self.record.identity());
    }
}
