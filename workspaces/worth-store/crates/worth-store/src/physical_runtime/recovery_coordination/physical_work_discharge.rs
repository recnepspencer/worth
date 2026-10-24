//! The physical-work obligations a killed runtime left behind, as recovery
//! sees them: each record admitted through the same integrity admission the
//! serving inventory uses, or marked as evidence that did not admit.
//!
//! Recovery decides which obligations its own publication adjudicated; this
//! owner only lists them and removes the ones recovery names.

use worth_store_physical_backend::{AdmittedRecoveryFilesystemMedia, ArtifactTreeFailure};
use worth_store_physical_format::physical_work_obligation::PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES;

use crate::physical_runtime::work::admit_recovered_obligation;
use crate::physical_runtime::PhysicalWorkRecoveryLocator;

use super::PhysicalRecoveryCoordination;

/// One obligation record. `locator` is `None` when the record did not admit;
/// such evidence must be retained for inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPriorObligation {
    name: String,
    locator: Option<PhysicalWorkRecoveryLocator>,
}

impl RecoveryPriorObligation {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn locator(&self) -> Option<PhysicalWorkRecoveryLocator> {
        self.locator
    }
}

impl PhysicalRecoveryCoordination {
    /// Lists at most `maximum_records` obligation records. A non-record name
    /// or an oversized record refuses the listing as damaged evidence.
    pub fn prior_physical_work_obligations(
        &self,
        media: &AdmittedRecoveryFilesystemMedia,
        maximum_records: usize,
    ) -> Result<Vec<RecoveryPriorObligation>, ArtifactTreeFailure> {
        let records = media.list_recovery_physical_work_records(
            maximum_records,
            PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES as u64,
        )?;
        Ok(records
            .iter()
            .map(|record| RecoveryPriorObligation {
                name: record.name().to_owned(),
                locator: admit_recovered_obligation(self.store, record.name(), record.bytes()),
            })
            .collect())
    }

    /// Durably removes one obligation that recovery has adjudicated.
    pub fn discharge_prior_physical_work_obligation(
        &self,
        media: &AdmittedRecoveryFilesystemMedia,
        obligation: &RecoveryPriorObligation,
    ) -> Result<(), ArtifactTreeFailure> {
        media.discharge_recovery_physical_work_record(obligation.name())
    }
}
