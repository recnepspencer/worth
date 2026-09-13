use crate::history::{CompositeHistoryCatalog, ReservedCompositeCommitCapacity};
use crate::identity::{CompositeCommitIdentity, ProductUnpublishedOwnerEffectsIdentity};
use crate::lifecycle::owner::{
    ReservedPublicationAttemptCapacity, RuntimeWorldOperationReservation,
};
use crate::recovery::ReservedProductUnpublishedSlot;
use crate::retention::ReservedComponentPinPairCapacity;
use crate::retention::ReservedObservationCapacity;

/// The bounded reservations every terminal-governed attempt acquires before
/// its first owner effect. Publication and branch creation reserve the same
/// resources under the same rules, so they share one bundle rather than two
/// parallel field lists.
#[derive(Debug)]
#[must_use = "reserved attempt capacities are consumed by a terminal or released on drop"]
pub(crate) struct ReservedAttemptCapacities {
    admitted_at: crate::lifecycle::RuntimeWorldInstant,
    reserved_commit_identity: CompositeCommitIdentity,
    product_unpublished_identity: ProductUnpublishedOwnerEffectsIdentity,
    reserved_commit_capacity: ReservedCompositeCommitCapacity,
    reserved_recovery_slot: ReservedProductUnpublishedSlot,
    reserved_component_pin_pair: ReservedComponentPinPairCapacity,
    reserved_successor_observation_capacity: Option<ReservedObservationCapacity>,
    reserved_successor_observation_pin_pair: Option<ReservedComponentPinPairCapacity>,
    reserved_publication_capacity: ReservedPublicationAttemptCapacity,
    history: CompositeHistoryCatalog,
    operation: RuntimeWorldOperationReservation,
}

/// The eight reservations named apart. The bundle is assembled from this
/// record and disassembled back into it, so a terminal that takes the bundle
/// apart names each resource it consumes rather than counting tuple slots.
pub(crate) struct ReservedAttemptCapacityInputs {
    pub(crate) admitted_at: crate::lifecycle::RuntimeWorldInstant,
    pub(crate) reserved_commit_identity: CompositeCommitIdentity,
    pub(crate) product_unpublished_identity: ProductUnpublishedOwnerEffectsIdentity,
    pub(crate) reserved_commit_capacity: ReservedCompositeCommitCapacity,
    pub(crate) reserved_recovery_slot: ReservedProductUnpublishedSlot,
    pub(crate) reserved_component_pin_pair: ReservedComponentPinPairCapacity,
    pub(crate) reserved_successor_observation_capacity: Option<ReservedObservationCapacity>,
    pub(crate) reserved_successor_observation_pin_pair: Option<ReservedComponentPinPairCapacity>,
    pub(crate) reserved_publication_capacity: ReservedPublicationAttemptCapacity,
    pub(crate) history: CompositeHistoryCatalog,
    pub(crate) operation: RuntimeWorldOperationReservation,
}

impl ReservedAttemptCapacities {
    pub(crate) fn new(inputs: ReservedAttemptCapacityInputs) -> Self {
        let ReservedAttemptCapacityInputs {
            admitted_at,
            reserved_commit_identity,
            product_unpublished_identity,
            reserved_commit_capacity,
            reserved_recovery_slot,
            reserved_component_pin_pair,
            reserved_successor_observation_capacity,
            reserved_successor_observation_pin_pair,
            reserved_publication_capacity,
            history,
            operation,
        } = inputs;
        Self {
            admitted_at,
            reserved_commit_identity,
            product_unpublished_identity,
            reserved_commit_capacity,
            reserved_recovery_slot,
            reserved_component_pin_pair,
            reserved_successor_observation_capacity,
            reserved_successor_observation_pin_pair,
            reserved_publication_capacity,
            history,
            operation,
        }
    }

    pub(crate) fn into_parts(self) -> ReservedAttemptCapacityInputs {
        ReservedAttemptCapacityInputs {
            admitted_at: self.admitted_at,
            reserved_commit_identity: self.reserved_commit_identity,
            product_unpublished_identity: self.product_unpublished_identity,
            reserved_commit_capacity: self.reserved_commit_capacity,
            reserved_recovery_slot: self.reserved_recovery_slot,
            reserved_component_pin_pair: self.reserved_component_pin_pair,
            reserved_successor_observation_capacity: self.reserved_successor_observation_capacity,
            reserved_successor_observation_pin_pair: self.reserved_successor_observation_pin_pair,
            reserved_publication_capacity: self.reserved_publication_capacity,
            history: self.history,
            operation: self.operation,
        }
    }
}
