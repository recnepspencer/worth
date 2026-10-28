//! A lease and selected custody transfer together, through serving admission.
use super::handoff::RecoveredPhysicalRuntimeCore;
use crate::physical_runtime::{
    AdmittedPhysicalRuntime, FilesystemMediaAdmission, MediaOwnedPhysicalRuntime,
    PhysicalRecordOpen, RecordStoreOpenOutcome, RecoveredPhysicalCheckpointCustody,
};
use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;

pub struct RecoveredPhysicalServingHandoff {
    media: AdmittedRecoveryFilesystemMedia,
    custody: RecoveredPhysicalCheckpointCustody,
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
    /// A move-only handoff that cannot release the lease while retaining
    /// ordinary authority to open Serving against its selected custody.
    pub fn into_serving_handoff(self) -> Option<RecoveredPhysicalServingHandoff> {
        let (media, custody) = self.into_serving_parts()?;
        Some(RecoveredPhysicalServingHandoff { media, custody })
    }
}

impl RecoveredPhysicalServingHandoff {
    pub fn admit_serving(
        self,
        runtime: AdmittedPhysicalRuntime,
        admission: FilesystemMediaAdmission,
    ) -> RecoveredMediaAdmissionOutcome {
        crate::physical_runtime::media_ownership::admit_recovered(runtime, admission, self.media)
            .map_success(|media| RecoveredMediaOwnedPhysicalRuntime {
                media,
                custody: self.custody,
            })
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
