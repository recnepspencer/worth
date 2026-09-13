use worth_store_operations::{OperationalControlRecord, OperationalControlRecordKind};

use super::{
    binding::binding_from_record, OperationalRecoveryAction, OperationalRecoveryActionKind,
};

pub fn map_operational_control_record(
    record: &OperationalControlRecord,
) -> OperationalRecoveryAction {
    let kind = map_operational_kind(record.kind());
    OperationalRecoveryAction {
        authority_identity: record.authority_identity().fingerprint(),
        operation_identity: record.operation_id().as_str().to_owned(),
        transition_identity: record.transition_id().as_str().to_owned(),
        kind,
        owner_tag: match record.kind() {
            OperationalControlRecordKind::OperationalOwnerReceiptPersisted {
                owner_kind, ..
            } => Some(owner_kind.tag()),
            _ => None,
        },
        binding: binding_from_record(record.kind()),
    }
}

fn map_operational_kind(kind: &OperationalControlRecordKind) -> OperationalRecoveryActionKind {
    use OperationalControlRecordKind as Record;
    match kind {
        Record::WorkflowOpened { .. } => OperationalRecoveryActionKind::WorkflowOpened,
        Record::SourceLeasePersisted { .. } => OperationalRecoveryActionKind::SourceLeasePersisted,
        Record::BackupMaterializationOpened { .. } => {
            OperationalRecoveryActionKind::MaterializationOpened
        }
        Record::BackupMaterializationRecorded { .. } => {
            OperationalRecoveryActionKind::MaterializationRecorded
        }
        Record::IndependentBackupVerificationRecordedAndSourceLeaseReleased { .. } => {
            OperationalRecoveryActionKind::IndependentVerificationRecorded
        }
        Record::BackupAbandoned { .. } | Record::ReplicaBootstrapAbandoned { .. } => {
            OperationalRecoveryActionKind::Abandoned
        }
        Record::AuthorizationConsumed { .. } => {
            OperationalRecoveryActionKind::AuthorizationConsumed
        }
        Record::RepairExecutionOpened { .. } => OperationalRecoveryActionKind::OwnerExecutionOpened,
        Record::RepairOwnerEffectStarted { .. } => {
            OperationalRecoveryActionKind::OwnerEffectStarted
        }
        Record::RepairOwnerReceiptPersisted { .. } => {
            OperationalRecoveryActionKind::OwnerReceiptPersisted
        }
        Record::OperationalOwnerReceiptPersisted { .. } => {
            OperationalRecoveryActionKind::WorkflowOwnerReceiptPersisted
        }
        Record::RepairDispositionRecorded { .. } => {
            OperationalRecoveryActionKind::DispositionRecorded
        }
        Record::RecoveryStagingCompleted { .. } => OperationalRecoveryActionKind::StagingCompleted,
        Record::ReplicaBootstrapTransferRecorded { .. } => {
            OperationalRecoveryActionKind::ReplicaBootstrapTransferRecorded
        }
        Record::ReplicaBootstrapCompleted { .. } => {
            OperationalRecoveryActionKind::ReplicaBootstrapCompleted
        }
        Record::ReplicaPromotionFenceRecorded { .. } => {
            OperationalRecoveryActionKind::ReplicaPromotionFenceRecorded
        }
        Record::ReplicaPromotionRecorded { .. } => {
            OperationalRecoveryActionKind::ReplicaPromotionRecorded
        }
        Record::ReplicaPromotionPublished { .. } => {
            OperationalRecoveryActionKind::ReplicaPromotionPublished
        }
        Record::ReplicaPromotionReadmitted { .. } => {
            OperationalRecoveryActionKind::ReplicaPromotionReadmitted
        }
        Record::OldPrimaryRejoinPlanned { .. } => {
            OperationalRecoveryActionKind::OldPrimaryRejoinPlanned
        }
        Record::OldPrimaryRejoinCompleted { .. } => {
            OperationalRecoveryActionKind::OldPrimaryRejoinCompleted
        }
    }
}
