use super::{
    BackupMaterializationRecoveryPlan, OperationalControlRecord, OperationalControlRecordKind,
    OperationalOperationId, OperationalOwnerReceiptKind, OperationalTransitionId,
    OperationalWorkflowKind, PersistedOperationalControlRecordKind, PersistedWorkflowKind,
};
use worth_store_physical_backend::ControlMediaFault;
use worth_store_physical_backend::ControlRecoveryObjectHandle;
use worth_store_physical_isolation::{
    BackupCutRecoveryRecord, BackupReachabilityLeaseReleaseRecord,
};

#[derive(Debug)]
pub(crate) enum PersistedControlRecordDecodeDenial {
    InvalidEncoding,
    AllocationFailed,
    Media(ControlMediaFault),
    ReplayBudgetExceeded { required: u64, limit: u64 },
}

pub(crate) struct PersistedOperationalControlRecord {
    pub(crate) authority_identity_fingerprint: [u8; 32],
    pub(crate) operation_id: String,
    pub(crate) transition_id: String,
    pub(crate) kind: PersistedOperationalControlRecordKind,
}

impl PersistedOperationalControlRecord {
    pub(crate) fn into_domain(
        self,
        load_recovery_object: impl FnOnce(
            ControlRecoveryObjectHandle,
        )
            -> Result<Vec<u8>, PersistedControlRecordDecodeDenial>,
    ) -> Result<OperationalControlRecord, PersistedControlRecordDecodeDenial> {
        let operation_id = OperationalOperationId::new(self.operation_id)
            .map_err(|_| PersistedControlRecordDecodeDenial::InvalidEncoding)?;
        let transition_id = OperationalTransitionId::new(self.transition_id)
            .map_err(|_| PersistedControlRecordDecodeDenial::InvalidEncoding)?;
        Ok(OperationalControlRecord::from_persisted_parts(
            worth_store_authority::StoreCurrentAuthorityIdentity::from_persisted_fingerprint(
                self.authority_identity_fingerprint,
            ),
            operation_id,
            transition_id,
            self.kind.into_domain(load_recovery_object)?,
        ))
    }
}

impl PersistedOperationalControlRecordKind {
    fn into_domain(
        self,
        load_recovery_object: impl FnOnce(
            ControlRecoveryObjectHandle,
        )
            -> Result<Vec<u8>, PersistedControlRecordDecodeDenial>,
    ) -> Result<OperationalControlRecordKind, PersistedControlRecordDecodeDenial> {
        Ok(match self {
            Self::WorkflowOpened { workflow } => OperationalControlRecordKind::WorkflowOpened {
                workflow: workflow.into(),
            },
            Self::SourceLeasePersisted {
                cut_identity,
                object_digest,
                object_bytes,
            } => {
                let recovery_object =
                    ControlRecoveryObjectHandle::from_persisted(object_digest, object_bytes)
                        .ok_or(PersistedControlRecordDecodeDenial::InvalidEncoding)?;
                let recovery_bytes = load_recovery_object(recovery_object)?;
                let recovery = BackupCutRecoveryRecord::recover(&recovery_bytes)
                    .map_err(|_| PersistedControlRecordDecodeDenial::InvalidEncoding)?;
                if recovery.cut_identity() != cut_identity {
                    return Err(PersistedControlRecordDecodeDenial::InvalidEncoding);
                }
                OperationalControlRecordKind::SourceLeasePersisted {
                    recovery: Box::new(recovery),
                    recovery_object,
                }
            }
            Self::BackupMaterializationOpened {
                cut_identity,
                target_platform,
                target_bytes,
                buffer_bytes,
            } => {
                let target_parent = super::operational_media_path::decode_operational_media_path(
                    target_platform,
                    &target_bytes,
                )
                .map_err(|_| PersistedControlRecordDecodeDenial::InvalidEncoding)?;
                let plan = BackupMaterializationRecoveryPlan::from_persisted(
                    cut_identity,
                    target_parent,
                    buffer_bytes,
                )
                .map_err(|denial| match denial {
                    super::BackupMaterializationRecoveryPlanDenial::AllocationFailed => {
                        PersistedControlRecordDecodeDenial::AllocationFailed
                    }
                    _ => PersistedControlRecordDecodeDenial::InvalidEncoding,
                })?;
                OperationalControlRecordKind::BackupMaterializationOpened { plan }
            }
            Self::BackupMaterializationRecorded { manifest_digest } => {
                OperationalControlRecordKind::BackupMaterializationRecorded { manifest_digest }
            },
            Self::IndependentBackupVerificationRecordedAndSourceLeaseReleased {
                verification_identity,
                release_recovery_bytes,
            } => OperationalControlRecordKind::IndependentBackupVerificationRecordedAndSourceLeaseReleased {
                verification_identity,
                release: BackupReachabilityLeaseReleaseRecord::recover(&release_recovery_bytes)
                    .map_err(|_| PersistedControlRecordDecodeDenial::InvalidEncoding)?,
            },
            Self::BackupAbandoned {
                reason,
                released_source_lease,
            } => OperationalControlRecordKind::BackupAbandoned {
                reason,
                released_source_lease: BackupReachabilityLeaseReleaseRecord::recover(
                    &released_source_lease,
                )
                    .map_err(|_| PersistedControlRecordDecodeDenial::InvalidEncoding)?,
            },
            Self::AuthorizationConsumed {
                authorization_identity,
                plan_fingerprint,
                operation_tag,
                execution_plan_fingerprint,
                assertion_identity,
                expires_at,
                replay_same_operation_identity,
            } => OperationalControlRecordKind::AuthorizationConsumed {
                authorization_identity,
                plan_fingerprint,
                operation_tag,
                execution_plan_fingerprint,
                assertion_identity,
                expires_at,
                replay_same_operation_identity,
            },
            Self::RepairExecutionOpened { authorization_identity, plan_fingerprint, owner_node_count,
                topology_tag } =>
                OperationalControlRecordKind::RepairExecutionOpened {
                    authorization_identity, plan_fingerprint, owner_node_count, topology_tag },
            Self::RepairOwnerReceiptPersisted { plan_fingerprint, node_fingerprint,
                receipt_fingerprint, owner_tag } =>
                OperationalControlRecordKind::RepairOwnerReceiptPersisted {
                    plan_fingerprint, node_fingerprint, receipt_fingerprint, owner_tag },
            Self::RepairOwnerEffectStarted { plan_fingerprint, node_fingerprint, owner_tag } =>
                OperationalControlRecordKind::RepairOwnerEffectStarted {
                    plan_fingerprint, node_fingerprint, owner_tag },
            Self::OperationalOwnerReceiptPersisted { workflow, plan_fingerprint,
                receipt_fingerprint, owner_tag } =>
                OperationalControlRecordKind::OperationalOwnerReceiptPersisted {
                    workflow: workflow.into(), plan_fingerprint,
                    receipt_fingerprint,
                    owner_kind: OperationalOwnerReceiptKind::from_tag(owner_tag)
                        .ok_or(PersistedControlRecordDecodeDenial::InvalidEncoding)? },
            Self::ReplicaBootstrapTransferRecorded {
                authorization_plan_fingerprint, execution_plan_fingerprint, receipt_identity,
                durable_target_identity, source_lease_identity,
                source_bytes_read, output_bytes_written, backend_requests,
                maximum_resident_buffer_bytes,
            } => OperationalControlRecordKind::ReplicaBootstrapTransferRecorded {
                authorization_plan_fingerprint, execution_plan_fingerprint, receipt_identity,
                durable_target_identity, source_lease_identity,
                source_bytes_read, output_bytes_written, backend_requests,
                maximum_resident_buffer_bytes,
            },
            Self::ReplicaBootstrapCompleted {
                receipt_identity, verification_identity, source_lease_identity,
            } => OperationalControlRecordKind::ReplicaBootstrapCompleted {
                receipt_identity, verification_identity, source_lease_identity,
            },
            Self::ReplicaBootstrapAbandoned {
                receipt_identity, reason, source_lease_identity,
            } => OperationalControlRecordKind::ReplicaBootstrapAbandoned {
                receipt_identity, reason, source_lease_identity,
            },
            Self::ReplicaPromotionFenceRecorded {
                authorization_plan_fingerprint, execution_plan_fingerprint, fence_identity,
                promoted_epoch,
            } => OperationalControlRecordKind::ReplicaPromotionFenceRecorded {
                authorization_plan_fingerprint, execution_plan_fingerprint, fence_identity,
                promoted_epoch,
            },
            Self::ReplicaPromotionRecorded {
                authorization_plan_fingerprint, execution_plan_fingerprint, receipt_identity,
                fence_identity, promoted_epoch,
            } => OperationalControlRecordKind::ReplicaPromotionRecorded {
                authorization_plan_fingerprint, execution_plan_fingerprint, receipt_identity,
                fence_identity, promoted_epoch,
            },
            Self::ReplicaPromotionPublished {
                receipt_identity, verification_identity, publication_identity,
                target_identity, promoted_epoch,
            } => OperationalControlRecordKind::ReplicaPromotionPublished {
                receipt_identity, verification_identity, publication_identity,
                target_identity, promoted_epoch,
            },
            Self::ReplicaPromotionReadmitted {
                publication_identity, serve_lease_identity, serving_epoch,
            } => OperationalControlRecordKind::ReplicaPromotionReadmitted {
                publication_identity, serve_lease_identity, serving_epoch,
            },
            Self::OldPrimaryRejoinPlanned {
                promotion_receipt_identity, rejoin_plan_fingerprint, disposition_tag,
            } => OperationalControlRecordKind::OldPrimaryRejoinPlanned {
                promotion_receipt_identity, rejoin_plan_fingerprint, disposition_tag,
            },
            Self::OldPrimaryRejoinCompleted {
                rejoin_plan_fingerprint, rejoin_receipt_identity,
                forensic_retention_identity, rebootstrap_target_identity, disposition_tag,
            } => OperationalControlRecordKind::OldPrimaryRejoinCompleted {
                rejoin_plan_fingerprint, rejoin_receipt_identity,
                forensic_retention_identity, rebootstrap_target_identity, disposition_tag,
            },
            Self::RepairDispositionRecorded { plan_fingerprint, disposition_tag,
                disposition_basis } =>
                OperationalControlRecordKind::RepairDispositionRecorded {
                    plan_fingerprint, disposition_tag, disposition_basis },
            Self::RecoveryStagingCompleted { authorization_identity, plan_fingerprint,
                execution_plan_fingerprint, staged_media_identity } =>
                OperationalControlRecordKind::RecoveryStagingCompleted {
                    authorization_identity, plan_fingerprint, execution_plan_fingerprint,
                    staged_media_identity },
        })
    }
}

impl From<OperationalWorkflowKind> for PersistedWorkflowKind {
    fn from(value: OperationalWorkflowKind) -> Self {
        match value {
            OperationalWorkflowKind::OfflineInspection => Self::OfflineInspection,
            OperationalWorkflowKind::Backup => Self::Backup,
            OperationalWorkflowKind::Restore => Self::Restore,
            OperationalWorkflowKind::PointInTimeRecovery => Self::PointInTimeRecovery,
            OperationalWorkflowKind::Rollback => Self::Rollback,
            OperationalWorkflowKind::Repair => Self::Repair,
            OperationalWorkflowKind::ReplicaBootstrap => Self::ReplicaBootstrap,
            OperationalWorkflowKind::ReplicaPromotion => Self::ReplicaPromotion,
            OperationalWorkflowKind::ForensicAcquisition => Self::ForensicAcquisition,
        }
    }
}

impl From<PersistedWorkflowKind> for OperationalWorkflowKind {
    fn from(value: PersistedWorkflowKind) -> Self {
        match value {
            PersistedWorkflowKind::OfflineInspection => Self::OfflineInspection,
            PersistedWorkflowKind::Backup => Self::Backup,
            PersistedWorkflowKind::Restore => Self::Restore,
            PersistedWorkflowKind::PointInTimeRecovery => Self::PointInTimeRecovery,
            PersistedWorkflowKind::Rollback => Self::Rollback,
            PersistedWorkflowKind::Repair => Self::Repair,
            PersistedWorkflowKind::ReplicaBootstrap => Self::ReplicaBootstrap,
            PersistedWorkflowKind::ReplicaPromotion => Self::ReplicaPromotion,
            PersistedWorkflowKind::ForensicAcquisition => Self::ForensicAcquisition,
        }
    }
}
