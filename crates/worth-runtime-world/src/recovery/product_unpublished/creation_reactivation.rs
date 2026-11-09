//! Transfer complete retained creation custody back to its original live driver.
use super::*;
use crate::publication::ActiveAttemptRecord;
impl ProductUnpublishedOwnerEffectsRecord {
    pub(crate) fn is_durable_creation(&self) -> bool {
        self.retention
            .creation
            .as_ref()
            .is_some_and(|c| c.durable.is_some())
    }
    pub(crate) fn into_active_creation(
        mut self,
        operation: crate::lifecycle::owner::RuntimeWorldOperationReservation,
        capacity: crate::lifecycle::owner::ReservedPublicationAttemptCapacity,
    ) -> Arc<ActiveAttemptRecord> {
        assert!(self.is_durable_creation());
        self.retention
            .reactivate_creation_operation(operation, capacity);
        let destination = self
            .retention
            .creation
            .as_ref()
            .expect("creation")
            .durable
            .as_ref()
            .expect("durable original")
            .destination_witness();
        ActiveAttemptRecord::from_retained_creation(crate::publication::ReactivatedCreationFacts {
            identity: self.identity,
            attempt: self.attempt_identity,
            expected: self.expected_head,
            admitted_at: self.admitted_at,
            deadline: self.deadline,
            progress: self.progress,
            successor: self.successor_basis,
            resources: self.retention,
            cause: self.cause,
            before_effect_refusal: self.before_effect_refusal,
            last_observed: self.last_observed_head,
            destination,
        })
    }
}
