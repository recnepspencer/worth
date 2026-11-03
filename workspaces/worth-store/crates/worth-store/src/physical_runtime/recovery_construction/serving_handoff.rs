//! A lease and selected custody transfer together, through serving admission.
use super::handoff::RecoveredPhysicalRuntimeCore;
use crate::physical_runtime::{
    AdmittedPhysicalRuntime, FilesystemMediaAdmission, MediaOwnedPhysicalRuntime,
    PhysicalRecordOpen, RecordStoreOpenOutcome, RecoveredPhysicalCheckpointCustody,
};
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;

pub struct RecoveredPhysicalServingHandoff {
    pub(super) media: AdmittedRecoveryFilesystemMedia,
    pub(super) custody: RecoveredPhysicalCheckpointCustody,
}

/// A qualified serving media owner still carrying the selected recovery custody.
pub struct RecoveredMediaOwnedPhysicalRuntime {
    media: MediaOwnedPhysicalRuntime,
    custody: RecoveredPhysicalCheckpointCustody,
}

pub type RecoveredMediaAdmissionOutcome = worth_proof::ProofOutcome<
    RecoveredMediaOwnedPhysicalRuntime,
    crate::physical_runtime::MediaAdmissionDenial,
    crate::physical_runtime::MediaAdmissionDeferred,
    crate::physical_runtime::MediaAdmissionStale,
    crate::physical_runtime::MediaAdmissionRebindRequired,
    crate::physical_runtime::MediaAdmissionInspectionRequired,
>;

impl RecoveredPhysicalRuntimeCore {
    /// Move the continuously held media owner and its one-shot selected custody
    /// together. Neither can be reconstructed from checkpoint bytes.
    pub fn into_serving_handoff(self) -> Option<RecoveredPhysicalServingHandoff> {
        let Self {
            serving_custody_backing,
            residency,
            checkpoint_ownership,
            store,
            recovery_allocation,
            root,
            media,
            reopen,
            head_v2_custody,
            no_release_custody,
            pending_wal_release_custody,
            effective_release_heads,
            historical_release_custody,
            tier_custody,
            selected_wal,
            selected_controls,
            ..
        } = self;
        drop(reopen);
        // Ownership agrees with custody by construction: the generation-zero
        // custody is minted only under this coordination's absence witness,
        // and every other custody requires the selected checkpoint.
        match (head_v2_custody, no_release_custody, pending_wal_release_custody, historical_release_custody, tier_custody, effective_release_heads) {
            (None, Some(no_release), None, None, tier, None) => Some(RecoveredPhysicalServingHandoff {
                custody: crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_no_release(
                    serving_custody_backing, residency, checkpoint_ownership, store, recovery_allocation, root, no_release, tier, selected_wal?, selected_controls?,
                ),
                media,
            }),
            (None, None, Some(pending), None, tier, Some(effective)) => Some(RecoveredPhysicalServingHandoff {
                custody: crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_pending_wal_release(
                    serving_custody_backing, residency, checkpoint_ownership, store, recovery_allocation, root, pending, effective, tier, selected_wal?, selected_controls?,
                ),
                media,
            }),
            (None, None, None, Some(historical), tier, Some(effective)) => Some(RecoveredPhysicalServingHandoff {
                custody: crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_ordered_historical_release(
                    serving_custody_backing, residency, checkpoint_ownership, store, recovery_allocation, root, historical, effective, tier, selected_wal?, selected_controls?,
                ),
                media,
            }),
            (Some(head_v2), None, None, None, tier, None) => Some(RecoveredPhysicalServingHandoff {
                custody: crate::physical_runtime::RecoveredPhysicalCheckpointCustody::from_verified_head_v2(
                    serving_custody_backing, residency, checkpoint_ownership, store, recovery_allocation, root, head_v2, tier, selected_wal?, selected_controls?,
                ),
                media,
            }),
            _ => None,
        }
    }
}

impl RecoveredPhysicalServingHandoff {
    pub fn admit_serving(
        self,
        runtime: AdmittedPhysicalRuntime,
        admission: FilesystemMediaAdmission,
    ) -> RecoveredMediaAdmissionOutcome {
        use worth_proof::TransitionOutcome;
        let outcome = match crate::physical_runtime::media_ownership::admit_recovered(
            runtime, admission, self.media,
        )
        .into_raw()
        {
            TransitionOutcome::Success(media) => {
                TransitionOutcome::Success(RecoveredMediaOwnedPhysicalRuntime {
                    media,
                    custody: self.custody,
                })
            }
            TransitionOutcome::Denied(denial) => TransitionOutcome::Denied(denial),
            TransitionOutcome::Deferred(deferred) => TransitionOutcome::Deferred(deferred),
            TransitionOutcome::Stale(stale) => TransitionOutcome::Stale(stale),
            TransitionOutcome::RebindRequired(rebind) => TransitionOutcome::RebindRequired(rebind),
            TransitionOutcome::Failed(failure) => TransitionOutcome::Failed(failure),
        };
        outcome.into()
    }
}

impl RecoveredMediaOwnedPhysicalRuntime {
    pub fn physical_durability_admission_basis(
        &self,
    ) -> Result<
        worth_store_physical_backend::PhysicalDurabilityAdmissionBasis,
        worth_store_physical_backend::BackendCapabilityAdmissionDenial,
    > {
        self.media.physical_durability_admission_basis()
    }
    /// Opens only with the custody recovered under this continuously held lease.
    pub fn open_record_store(self, request: PhysicalRecordOpen) -> RecordStoreOpenOutcome {
        self.media
            .open_record_store(request.with_recovered_checkpoint_custody(self.custody))
    }
}
