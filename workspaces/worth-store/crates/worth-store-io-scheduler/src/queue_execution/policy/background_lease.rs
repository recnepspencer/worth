use crate::{BackgroundIdleCapacityLease, BackgroundIoPressureClass};

use super::{
    QueueDurabilityClass, QueueGroupingBasis, QueueRecoveryOrdering, QueueWorkClass,
    QueueWorkDeclaration, QueueWritebackPolicy,
};

pub fn lower_background_queue_lease(lease: BackgroundIdleCapacityLease) -> QueueWorkDeclaration {
    let basis = lease.basis();
    let security_identity = basis.security_scope_identity();
    let class = lease.class();
    let work = QueueWorkDeclaration::background(lease);
    work.with_grouping_basis(QueueGroupingBasis::new(
        security_identity,
        security_identity.tenant_scope(),
        security_identity.key_scope(),
        security_identity.authenticity_requirement(),
        durability_for_background(class),
        background_flush_epoch(class),
        QueueWorkClass::Background(class),
        recovery_ordering_for_background(class),
        writeback_policy_for_background(class),
    ))
}

pub(super) const fn durability_for_background(
    class: BackgroundIoPressureClass,
) -> QueueDurabilityClass {
    match class {
        BackgroundIoPressureClass::CompactionRewrite
        | BackgroundIoPressureClass::CheckpointFlush
        | BackgroundIoPressureClass::IngestPressure
        | BackgroundIoPressureClass::BlobReclaimPressure
        | BackgroundIoPressureClass::MigrationPressure => QueueDurabilityClass::BufferedWrite,
        BackgroundIoPressureClass::ScrubScan
        | BackgroundIoPressureClass::ReplicationPrepRead
        | BackgroundIoPressureClass::BackupPrepRead
        | BackgroundIoPressureClass::RepairScan
        | BackgroundIoPressureClass::VerificationPressure => QueueDurabilityClass::ReadOnly,
    }
}

const fn recovery_ordering_for_background(
    class: BackgroundIoPressureClass,
) -> QueueRecoveryOrdering {
    match class {
        BackgroundIoPressureClass::CheckpointFlush => QueueRecoveryOrdering::WalBeforeData,
        BackgroundIoPressureClass::RepairScan | BackgroundIoPressureClass::VerificationPressure => {
            QueueRecoveryOrdering::RecoveryReadOnly
        }
        BackgroundIoPressureClass::CompactionRewrite
        | BackgroundIoPressureClass::ScrubScan
        | BackgroundIoPressureClass::ReplicationPrepRead
        | BackgroundIoPressureClass::IngestPressure
        | BackgroundIoPressureClass::BlobReclaimPressure
        | BackgroundIoPressureClass::MigrationPressure
        | BackgroundIoPressureClass::BackupPrepRead => QueueRecoveryOrdering::NotRecoveryCritical,
    }
}

const fn writeback_policy_for_background(class: BackgroundIoPressureClass) -> QueueWritebackPolicy {
    match class {
        BackgroundIoPressureClass::CompactionRewrite
        | BackgroundIoPressureClass::CheckpointFlush
        | BackgroundIoPressureClass::IngestPressure
        | BackgroundIoPressureClass::BlobReclaimPressure
        | BackgroundIoPressureClass::MigrationPressure => {
            QueueWritebackPolicy::DeferredWithinFlushEpoch
        }
        BackgroundIoPressureClass::ScrubScan
        | BackgroundIoPressureClass::ReplicationPrepRead
        | BackgroundIoPressureClass::BackupPrepRead
        | BackgroundIoPressureClass::RepairScan
        | BackgroundIoPressureClass::VerificationPressure => QueueWritebackPolicy::None,
    }
}

const fn background_flush_epoch(class: BackgroundIoPressureClass) -> u64 {
    match class {
        BackgroundIoPressureClass::CheckpointFlush => 1,
        BackgroundIoPressureClass::CompactionRewrite
        | BackgroundIoPressureClass::ScrubScan
        | BackgroundIoPressureClass::ReplicationPrepRead
        | BackgroundIoPressureClass::IngestPressure
        | BackgroundIoPressureClass::BlobReclaimPressure
        | BackgroundIoPressureClass::MigrationPressure
        | BackgroundIoPressureClass::BackupPrepRead
        | BackgroundIoPressureClass::RepairScan
        | BackgroundIoPressureClass::VerificationPressure => 0,
    }
}
