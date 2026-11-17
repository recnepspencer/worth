//! Admission of an owned publication waits on the active physical predecessor.
use super::*;
use crate::physical_runtime::durability::PendingPublicationLease;

impl RecordPublicationDirector {
    pub(super) fn admit_publication_original(
        &self,
        attempt: &PhysicalMutationAttempt,
    ) -> Result<PendingPublicationLease, PhysicalPublicationAdmissionDenial> {
        loop {
            match self
                .root_owner
                .register_pending_publication(attempt.identity())
            {
                Err(PhysicalPublicationAdmissionDenial::ScopeConflict(predecessor))
                    if attempt.retains_publication_original() =>
                {
                    self.mutations.reach_checkpoint(
                        PhysicalMutationCheckpoint::PublicationAdmissionContended,
                    );
                    if !self.mutations.await_publication_predecessor(predecessor) {
                        return Err(PhysicalPublicationAdmissionDenial::ScopeConflict(
                            predecessor,
                        ));
                    }
                }
                admission => return admission,
            }
        }
    }
}
