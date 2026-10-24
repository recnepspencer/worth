//! Discharge of the physical-work obligations a killed runtime left behind.
//!
//! An obligation says an effect may or may not have reached media. Once
//! recovery has published its own root and made the record namespace durable,
//! every WAL interval, record range, artifact synchronization, catalog
//! replacement and record removal of the dead runtime has been adjudicated:
//! recovery selected only what it verified from media, and nothing it did not
//! select is reachable. Those obligations are discharged. Checkpoint and WAL
//! reclamation obligations, records of another store, and records that do not
//! decode are retained, so serving still requires inspection for them.

use worth_store_physical_backend::{AdmittedRecoveryFilesystemMedia, ArtifactTreeFailure};
use worth_store_physical_format::physical_work_obligation::PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES;

use crate::physical_runtime::work::admit_recovered_obligation;
use crate::physical_runtime::PhysicalWorkRecoveryTarget;

use super::PhysicalRecoveryCoordination;

/// What one discharge pass did. `retained` obligations still require
/// inspection when the store opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryPhysicalWorkDischarge {
    pub discharged: u64,
    pub retained: u64,
}

impl PhysicalRecoveryCoordination {
    /// Discharges the adjudicated obligations, at most `maximum_records`.
    pub fn discharge_prior_physical_work(
        &self,
        media: &AdmittedRecoveryFilesystemMedia,
        maximum_records: usize,
    ) -> Result<RecoveryPhysicalWorkDischarge, ArtifactTreeFailure> {
        let records = media.list_recovery_physical_work_records(
            maximum_records,
            PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES as u64,
        )?;
        let mut discharge = RecoveryPhysicalWorkDischarge {
            discharged: 0,
            retained: 0,
        };
        for record in records.iter() {
            let adjudicated = admit_recovered_obligation(self.store, record.name(), record.bytes())
                .is_some_and(|locator| adjudicated_by_recovery(locator.target()));
            if adjudicated {
                media.discharge_recovery_physical_work_record(record.name())?;
                discharge.discharged += 1;
            } else {
                discharge.retained += 1;
            }
        }
        Ok(discharge)
    }
}

const fn adjudicated_by_recovery(target: PhysicalWorkRecoveryTarget) -> bool {
    match target {
        PhysicalWorkRecoveryTarget::Range(_)
        | PhysicalWorkRecoveryTarget::WalArtifactInterval { .. }
        | PhysicalWorkRecoveryTarget::ArtifactFileSynchronization(_)
        | PhysicalWorkRecoveryTarget::ArtifactParentSynchronization(_)
        | PhysicalWorkRecoveryTarget::CatalogReplacement(_)
        | PhysicalWorkRecoveryTarget::RecordNamespaceSynchronization
        | PhysicalWorkRecoveryTarget::ArtifactRemoval(_) => true,
        PhysicalWorkRecoveryTarget::Checkpoint { .. }
        | PhysicalWorkRecoveryTarget::WalSegmentReclamation { .. } => false,
    }
}
