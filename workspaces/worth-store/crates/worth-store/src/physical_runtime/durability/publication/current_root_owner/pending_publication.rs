use super::PhysicalCurrentRootOwner;
use crate::physical_runtime::{
    durability::retention::{PendingPublicationLease, PhysicalPublicationAdmissionDenial},
    PhysicalMutationIdentity,
};

impl PhysicalCurrentRootOwner {
    /// The root lock serializes pending admission with reclaim-fence insertion.
    /// Only the one registered fenced mutation can enter WAL/data work.
    pub(in crate::physical_runtime) fn register_pending_publication(
        &self,
        identity: PhysicalMutationIdentity,
    ) -> Result<PendingPublicationLease, PhysicalPublicationAdmissionDenial> {
        let _root = self.lock_publication_state();
        let fence = self.lock_reclaim();
        if fence.as_ref().is_some_and(|fence| !fence.accepts(identity)) {
            return Err(PhysicalPublicationAdmissionDenial::ReclaimFenced);
        }
        self.publication.register_exclusive_pending(identity)
    }
}
