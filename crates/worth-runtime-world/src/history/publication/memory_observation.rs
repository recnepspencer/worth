//! Root-free original World meaning, sealed by its actual successful movement.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use worth_foundational::facade::MemoryProductBranchReference;
use worth_foundational::facade::RecordedCompositePublicationAttemptIdentity;
use worth_relational::facade::{
    branch::ObservedRelationalMemoryComponent, durability::ObservedRelationalMemoryCommit,
};

#[derive(Debug)]
pub(crate) struct MemoryPublicationRecord {
    pub(crate) meaning: Arc<MemoryPublicationMeaning>,
    pub(crate) receiver: MemoryPublicationReceiver,
    pub(crate) before_component: ObservedRelationalMemoryComponent,
    pub(crate) after_component: ObservedRelationalMemoryComponent,
    pub(crate) committed: AtomicBool,
    pub(crate) _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Debug)]
pub(crate) struct MemoryPublicationMeaning {
    pub(crate) before: MemoryProductBranchReference,
    pub(crate) after: MemoryProductBranchReference,
    pub(crate) attempt: RecordedCompositePublicationAttemptIdentity,
    pub(crate) _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
#[derive(Debug, Clone)]
pub(crate) enum MemoryPublicationReceiver {
    Live,
    Restored(Arc<MemoryPublicationReceiverScope>),
    Archived,
}
#[derive(Debug)]
pub(crate) struct MemoryPublicationReceiverScope {
    pub(crate) reference: MemoryProductBranchReference,
    pub(crate) _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl MemoryPublicationRecord {
    pub(crate) fn matches_receiver(
        &self,
        observation: &crate::branch::ProductBranchObservation,
    ) -> bool {
        let reference = match &self.receiver {
            MemoryPublicationReceiver::Live => &self.meaning.after,
            MemoryPublicationReceiver::Restored(scope) => &scope.reference,
            MemoryPublicationReceiver::Archived => return false,
        };
        reference.owner() == observation.owner_identity().get()
            && reference.branch() == observation.branch_identity().name().as_str()
            && reference.incarnation() == observation.lifecycle_incarnation().ordinal()
    }
    pub(crate) fn mark_committed(&self, movement: &crate::branch::ProductBranchReferenceMovement) {
        for (actual, recorded) in [
            (movement.before(), &self.meaning.before),
            (movement.after(), &self.meaning.after),
        ] {
            assert_eq!(
                actual.branch_identity().owner_identity().get(),
                recorded.owner()
            );
            assert_eq!(actual.branch_identity().name().as_str(), recorded.branch());
            assert_eq!(
                actual.lifecycle_incarnation().ordinal(),
                recorded.incarnation()
            );
            assert_eq!(actual.reference_generation().get(), recorded.generation());
            assert_eq!(
                actual.selected_commit().ordinal(),
                recorded.selected_commit()
            );
        }
        self.committed.store(true, Ordering::Release);
    }
}
/// A request-local read of the actual Native original and compact World record.
/// The retained World record owns no live basis, root, result or history pin.
#[derive(Debug)]
pub struct WorldObservedMemoryPublication {
    pub(crate) record: Arc<MemoryPublicationRecord>,
    pub(crate) native: ObservedRelationalMemoryCommit,
}
impl WorldObservedMemoryPublication {
    pub fn before(&self) -> &MemoryProductBranchReference {
        &self.record.meaning.before
    }
    pub fn after(&self) -> &MemoryProductBranchReference {
        &self.record.meaning.after
    }
    pub fn before_component(&self) -> &ObservedRelationalMemoryComponent {
        &self.record.before_component
    }
    pub fn after_component(&self) -> &ObservedRelationalMemoryComponent {
        &self.record.after_component
    }
    pub fn attempt_identity(&self) -> RecordedCompositePublicationAttemptIdentity {
        self.record.meaning.attempt
    }
    pub fn matches_receiver(&self, observation: &crate::branch::ProductBranchObservation) -> bool {
        self.record.matches_receiver(observation)
    }
    pub fn native(&self) -> &ObservedRelationalMemoryCommit {
        &self.native
    }
}
