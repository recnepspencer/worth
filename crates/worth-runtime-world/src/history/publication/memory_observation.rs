//! Root-free original World meaning, sealed by its actual successful movement.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use worth_foundational::facade::MemoryProductBranchReference;
use worth_relational::facade::{
    branch::ObservedRelationalMemoryComponent, durability::ObservedRelationalMemoryCommit,
};

#[derive(Debug)]
pub(crate) struct MemoryPublicationRecord {
    pub(crate) before: MemoryProductBranchReference,
    pub(crate) after: MemoryProductBranchReference,
    pub(crate) attempt: crate::identity::CompositePublicationAttemptIdentity,
    pub(crate) before_component: ObservedRelationalMemoryComponent,
    pub(crate) after_component: ObservedRelationalMemoryComponent,
    pub(crate) committed: AtomicBool,
    pub(crate) _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl MemoryPublicationRecord {
    pub(crate) fn mark_committed(&self, movement: &crate::branch::ProductBranchReferenceMovement) {
        for (actual, recorded) in [
            (movement.before(), &self.before),
            (movement.after(), &self.after),
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
        &self.record.before
    }
    pub fn after(&self) -> &MemoryProductBranchReference {
        &self.record.after
    }
    pub fn before_component(&self) -> &ObservedRelationalMemoryComponent {
        &self.record.before_component
    }
    pub fn after_component(&self) -> &ObservedRelationalMemoryComponent {
        &self.record.after_component
    }
    pub fn attempt_identity(&self) -> &crate::identity::CompositePublicationAttemptIdentity {
        &self.record.attempt
    }
    pub fn native(&self) -> &ObservedRelationalMemoryCommit {
        &self.native
    }
}
