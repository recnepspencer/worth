use worth_store_contracts::QueueProducerResourceShape;
use worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement;

use super::{PhysicalWorkDurabilityRequirement, PhysicalWorkOperationFamily, PhysicalWorkScope};

/// Store-owned demand retained before lowering into scheduler units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalWorkResourceDemand {
    queue: QueueProducerResourceShape,
    flush_epoch: u64,
}

impl PhysicalWorkResourceDemand {
    pub(super) fn derive(
        scope: &PhysicalWorkScope,
        operation: PhysicalWorkOperationFamily,
        durability: PhysicalWorkDurabilityRequirement,
    ) -> Self {
        let members = scope.member_count() as u64;
        let bytes = accounted_bytes(scope);
        let queue = QueueProducerResourceShape::new()
            .with_queue_slots(members)
            .with_worker_permits(members)
            .with_bandwidth_tokens(bytes)
            .with_write_back_windows(
                if matches!(operation, PhysicalWorkOperationFamily::ArtifactRangeWrite) {
                    members
                } else {
                    0
                },
            )
            .with_flush_permits(u64::from(matches!(
                durability,
                PhysicalWorkDurabilityRequirement::ArtifactRangeWrite(
                    ArtifactRangeWriteDurabilityRequirement::FileDataSynchronization
                ) | PhysicalWorkDurabilityRequirement::WalDurabilityBarrier
                    | PhysicalWorkDurabilityRequirement::CheckpointCapture
                    | PhysicalWorkDurabilityRequirement::WalReclamation
                    | PhysicalWorkDurabilityRequirement::RootPublication
            )))
            .with_sync_debt(u64::from(matches!(
                operation,
                PhysicalWorkOperationFamily::ArtifactPublication
                    | PhysicalWorkOperationFamily::CheckpointCapture
                    | PhysicalWorkOperationFamily::DurabilityBarrier
                    | PhysicalWorkOperationFamily::WalReclamation
                    | PhysicalWorkOperationFamily::RootPublication
            )));
        Self {
            queue,
            flush_epoch: 0,
        }
    }

    pub const fn queue_shape(self) -> QueueProducerResourceShape {
        self.queue
    }

    pub const fn flush_epoch(self) -> u64 {
        self.flush_epoch
    }
}
fn accounted_bytes(scope: &PhysicalWorkScope) -> u64 {
    if let Some(range) = scope.inspection_target() {
        return range.length() as u64;
    }
    if let Some(target) = scope.wal_reclamation_target() {
        return target.byte_count();
    }
    if let Some(target) = scope.checkpoint_target() {
        return target.accounted_bytes();
    }
    if let Some(target) = scope.wal_append_target() {
        return target.byte_count();
    }
    if scope.wal_barrier_target().is_some() {
        return 1;
    }
    if let Some(target) = scope.root_publication_target() {
        return target.accounted_bytes();
    }
    scope.coordinates().iter().fold(0_u64, |total, range| {
        total.saturating_add(range.length() as u64)
    })
}
