use std::sync::{Arc, Mutex, MutexGuard};

use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::branch::ProductBranchObservation;
use crate::identity::{
    CompositePublicationAttemptIdentity, ProductUnpublishedOwnerEffectsIdentity,
};
use crate::lifecycle::RuntimeWorldInstant;
use crate::recovery::ReservedProductUnpublishedSlot;

use super::super::{
    CompositeAttemptProgress, ReservedAttemptCapacities, ReservedAttemptCapacityInputs,
};
use super::{ActiveAttemptResources, ActiveHistoryCustody, ActivePinCustody};

pub(crate) struct ActiveAttemptRecord {
    identity: ProductUnpublishedOwnerEffectsIdentity,
    pub(super) attempt: CompositePublicationAttemptIdentity,
    pub(super) expected: ProductBranchObservation,
    pub(super) admitted_at: RuntimeWorldInstant,
    pub(super) deadline: Option<RuntimeWorldInstant>,
    pub(super) publication: Option<Arc<crate::history::CanonicalPublicationEnvelope>>,
    pub(super) state: Mutex<ActiveAttemptState>,
}

pub(super) struct ActiveAttemptState {
    pub(super) progress: Arc<CompositeAttemptProgress>,
    pub(super) successor: Option<AdmittedCompositeRuntimeWorldBasis>,
    pub(super) resources: Option<ActiveAttemptResources>,
    pub(super) abandoned: bool,
    pub(super) cause: crate::recovery::ProductUnpublishedCause,
    pub(super) last_observed: Option<crate::branch::ProductBranchReferenceSnapshot>,
    pub(super) destination: Option<Arc<crate::branch::registry::ProductBranchInstallationWitness>>,
}

impl std::fmt::Debug for ActiveAttemptRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActiveAttemptRecord")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

impl ActiveAttemptRecord {
    pub(super) fn new(
        attempt: CompositePublicationAttemptIdentity,
        expected: ProductBranchObservation,
        deadline: Option<RuntimeWorldInstant>,
        capacities: ReservedAttemptCapacities,
    ) -> (Arc<Self>, ReservedProductUnpublishedSlot) {
        let ReservedAttemptCapacityInputs {
            admitted_at,
            reserved_commit_identity,
            product_unpublished_identity,
            reserved_commit_capacity,
            reserved_recovery_slot,
            reserved_component_pin_pair,
            reserved_publication_capacity,
            history: _,
            operation,
        } = capacities.into_parts();
        let publication = reserved_commit_capacity.publication_envelope().cloned();
        let resources = ActiveAttemptResources {
            conditional_definition: None,
            product_comparison_costs: None,
            commit_identity: reserved_commit_identity,
            commit: None,
            history_custody: ActiveHistoryCustody::Reserved(reserved_commit_capacity),
            pins: ActivePinCustody::Reserved(reserved_component_pin_pair),
            history_pins: None,
            pin_denial: None,
            product_head: None,
            delivery: None,
            creation: None,
            operation: Some(operation),
            publication_capacity: Some(reserved_publication_capacity),
        };
        let record = Arc::new(Self {
            identity: product_unpublished_identity,
            admitted_at,
            attempt,
            expected,
            deadline,
            publication,
            state: Mutex::new(ActiveAttemptState {
                progress: Arc::new(CompositeAttemptProgress::untouched()),
                successor: None,
                resources: Some(resources),
                abandoned: false,
                cause: crate::recovery::ProductUnpublishedCause::CallerAbandoned,
                last_observed: None,
                destination: None,
            }),
        });
        (record, reserved_recovery_slot)
    }

    pub(crate) fn admitted_at(&self) -> RuntimeWorldInstant {
        self.admitted_at
    }
    pub(crate) fn deadline(&self) -> Option<RuntimeWorldInstant> {
        self.deadline
    }

    pub(super) fn state(&self) -> MutexGuard<'_, ActiveAttemptState> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub(crate) const fn metadata_charge_hint() -> usize {
        std::mem::size_of::<Self>()
            + std::mem::size_of::<CompositeAttemptProgress>()
            + crate::branch::registry::ProductBranchInstallationWitness::metadata_charge_hint()
    }

    pub(crate) fn identity(&self) -> &ProductUnpublishedOwnerEffectsIdentity {
        &self.identity
    }

    pub(super) fn replace_progress(
        &self,
        progress: CompositeAttemptProgress,
    ) -> Arc<CompositeAttemptProgress> {
        let progress = Arc::new(progress);
        let old = std::mem::replace(&mut self.state().progress, Arc::clone(&progress));
        drop(old);
        progress
    }

    pub(super) fn set_successor(&self, basis: AdmittedCompositeRuntimeWorldBasis) {
        self.state().successor = Some(basis);
    }

    pub(super) fn progress(&self) -> Arc<CompositeAttemptProgress> {
        Arc::clone(&self.state().progress)
    }

    pub(super) fn product_moved(&self) -> bool {
        self.publication
            .as_ref()
            .is_some_and(|publication| publication.facts().is_some())
            || self
                .state()
                .destination
                .as_ref()
                .is_some_and(|destination| destination.installed_commit().is_some())
    }
    pub(crate) fn is_abandoned(&self) -> bool {
        self.state().abandoned
    }

    /// Called under the catalog lock. The returned admission permits stay live
    /// until the catalog has converted this attempt's accounting atomically.
    pub(crate) fn abandon(
        &self,
    ) -> (
        Option<crate::lifecycle::owner::RuntimeWorldOperationReservation>,
        Option<crate::lifecycle::owner::ReservedPublicationAttemptCapacity>,
        Option<crate::history::PublicationDeliveryClaim>,
        Option<crate::retention::ReservedObservationCapacity>,
    ) {
        let mut state = self.state();
        let resources = state
            .resources
            .as_mut()
            .expect("a resource lease restores before caller Drop");
        let permits = (
            resources.operation.take(),
            resources.publication_capacity.take(),
            resources.delivery.take(),
            resources
                .creation
                .as_mut()
                .and_then(|c| c.observation_capacity.take()),
        );
        state.abandoned = true;
        permits
    }
}
