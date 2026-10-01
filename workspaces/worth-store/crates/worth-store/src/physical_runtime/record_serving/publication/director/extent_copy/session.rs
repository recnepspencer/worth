use super::evidence::CopyWriteAccumulator;
use crate::physical_runtime::durability::DurableMaintenanceReceipt;
use crate::physical_runtime::record_serving::{
    access::extent_rewrite_source::ExtentRewriteCursor, arena::ArenaReservation,
    AdmittedRecordPlacementPolicy,
};
use crate::physical_runtime::{stability::PhysicalRootReadLease, PreparedPhysicalMutation};
use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DurableExtentRecordPlacement, PhysicalExtentCopyIntent, PhysicalExtentCopyRecord,
    RecordFrameCoordinate,
};

pub(in crate::physical_runtime::record_serving::publication::director) struct ExtentCopySession {
    pub(super) producer: CopyProducer,
    pub(super) placement: AdmittedRecordPlacementPolicy,
    pub(super) prepared: PreparedPhysicalMutation,
    pub(super) operation: [u8; 32],
    pub(super) source_root: u64,
    pub(super) source: DurableExtentRecordPlacement,
    pub(super) target_tier: worth_store_physical_format::PhysicalTierClass,
    pub(super) source_lease: PhysicalRootReadLease,
    pub(super) reservation: ArenaReservation,
    pub(super) allocation: worth_store_buffer_pool::ForegroundWriteAllocationGrant,
    pub(super) cursor: ExtentRewriteCursor,
    pub(super) digest: Sha256,
    pub(super) completed: u64,
    pub(super) intent: Option<PhysicalExtentCopyIntent>,
    pub(super) durable: Option<DurableMaintenanceReceipt>,
    pub(super) writes: Option<CopyWriteAccumulator>,
    pub(super) next_ordinal: u32,
    pub(super) pending: Option<(RecordFrameCoordinate, Vec<u8>)>,
    pub(super) synchronization: Option<super::ExtentCopySynchronization>,
    pub(super) verified_membership:
        Option<worth_store_physical_integrity::IntegrityValidatedExtentMembership>,
    pub(super) intent_escaped: bool,
    pub(super) phase: CopyPhase,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving::publication::director) enum CopyProducer {
    ArenaEvacuation,
    BlobMovement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CopyPhase {
    Hashing,
    Intent,
    Copying,
    Manifest,
    Synchronizing,
    Verifying,
    Complete,
    InspectionRequired,
}

impl ExtentCopySession {
    pub(super) fn intent_payload(&self) -> Vec<u8> {
        PhysicalExtentCopyRecord::Intent(self.intent.expect("intent phase has exact geometry"))
            .encode()
    }
    pub(super) fn hash_quantum(
        &mut self,
    ) -> Result<bool, crate::physical_runtime::record_serving::RecordStreamFailure> {
        match self.cursor.next_chunk(&self.allocation)? {
            Some(bytes) => {
                self.digest.update(bytes);
                self.completed += bytes.len() as u64;
                Ok(false)
            }
            None => Ok(true),
        }
    }
}
