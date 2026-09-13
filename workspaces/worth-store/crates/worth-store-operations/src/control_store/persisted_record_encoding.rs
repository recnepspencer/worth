use super::persisted_record_codec::OperationalControlEncodingDenial;
use super::persisted_record_codec_io::ControlRecordEncoder;
use super::{OperationalControlRecordKind, OperationalWorkflowKind};

pub(super) fn encode_kind(
    output: &mut ControlRecordEncoder,
    kind: &OperationalControlRecordKind,
) -> Result<(), OperationalControlEncodingDenial> {
    use OperationalControlRecordKind as Kind;
    match kind {
        Kind::WorkflowOpened { workflow } => {
            output.u8(1)?;
            output.u8(workflow_tag(*workflow))
        }
        Kind::SourceLeasePersisted {
            recovery,
            recovery_object,
        } => {
            output.u8(4)?;
            output.bytes(&recovery.cut_identity())?;
            output.bytes(&recovery_object.digest())?;
            output.u64(recovery_object.bytes())
        }
        Kind::BackupMaterializationOpened { plan } => {
            let (platform, path) = plan.persisted_path().map_err(|denial| match denial {
                super::operational_media_path::OperationalMediaPathDenial::AllocationFailed => {
                    OperationalControlEncodingDenial::AllocationFailed
                }
                _ => OperationalControlEncodingDenial::RecordTooLarge,
            })?;
            output.u8(11)?;
            output.bytes(&plan.cut_identity())?;
            output.u8(platform)?;
            output.length_prefixed_bytes(&path)?;
            output.u64(plan.buffer_bytes() as u64)
        }
        Kind::BackupMaterializationRecorded { manifest_digest } => {
            output.u8(5)?;
            output.bytes(manifest_digest)
        }
        Kind::IndependentBackupVerificationRecordedAndSourceLeaseReleased {
            verification_identity,
            release,
        } => {
            output.u8(6)?;
            output.bytes(verification_identity)?;
            output.length_prefixed_bytes(release.recovery_bytes())
        }
        Kind::BackupAbandoned {
            reason,
            released_source_lease,
        } => {
            output.u8(9)?;
            output.string(reason)?;
            output.u8(1)?;
            output.length_prefixed_bytes(released_source_lease.recovery_bytes())
        }
        Kind::AuthorizationConsumed {
            authorization_identity,
            plan_fingerprint,
            operation_tag,
            execution_plan_fingerprint,
            assertion_identity,
            expires_at,
            replay_same_operation_identity,
        } => {
            output.u8(12)?;
            output.bytes(authorization_identity)?;
            output.bytes(plan_fingerprint)?;
            output.u8(*operation_tag)?;
            if let Some(fingerprint) = execution_plan_fingerprint {
                output.u8(1)?;
                output.bytes(fingerprint)?;
            } else {
                output.u8(0)?;
            }
            output.bytes(assertion_identity)?;
            output.u64(*expires_at)?;
            output.u8(u8::from(*replay_same_operation_identity))
        }
        Kind::RepairExecutionOpened {
            authorization_identity,
            plan_fingerprint,
            owner_node_count,
            topology_tag,
        } => {
            output.u8(13)?;
            output.bytes(authorization_identity)?;
            output.bytes(plan_fingerprint)?;
            output.u64(*owner_node_count)?;
            output.u8(*topology_tag)
        }
        Kind::RepairOwnerReceiptPersisted {
            plan_fingerprint,
            node_fingerprint,
            receipt_fingerprint,
            owner_tag,
        } => {
            output.u8(14)?;
            output.bytes(plan_fingerprint)?;
            output.bytes(node_fingerprint)?;
            output.bytes(receipt_fingerprint)?;
            output.u8(*owner_tag)
        }
        Kind::RepairOwnerEffectStarted {
            plan_fingerprint,
            node_fingerprint,
            owner_tag,
        } => {
            output.u8(21)?;
            output.bytes(plan_fingerprint)?;
            output.bytes(node_fingerprint)?;
            output.u8(*owner_tag)
        }
        Kind::OperationalOwnerReceiptPersisted {
            workflow,
            plan_fingerprint,
            receipt_fingerprint,
            owner_kind,
        } => {
            output.u8(22)?;
            output.u8(workflow_tag(*workflow))?;
            output.bytes(plan_fingerprint)?;
            output.bytes(receipt_fingerprint)?;
            output.u8(owner_kind.tag())
        }
        Kind::ReplicaBootstrapTransferRecorded {
            authorization_plan_fingerprint,
            execution_plan_fingerprint,
            receipt_identity,
            durable_target_identity,
            source_lease_identity,
            source_bytes_read,
            output_bytes_written,
            backend_requests,
            maximum_resident_buffer_bytes,
        } => {
            output.u8(23)?;
            output.bytes(authorization_plan_fingerprint)?;
            output.bytes(execution_plan_fingerprint)?;
            output.bytes(receipt_identity)?;
            output.bytes(durable_target_identity)?;
            output.bytes(source_lease_identity)?;
            output.u64(*source_bytes_read)?;
            output.u64(*output_bytes_written)?;
            output.u64(*backend_requests)?;
            output.u64(*maximum_resident_buffer_bytes)
        }
        Kind::ReplicaBootstrapCompleted {
            receipt_identity,
            verification_identity,
            source_lease_identity,
        } => {
            output.u8(26)?;
            output.bytes(receipt_identity)?;
            output.bytes(verification_identity)?;
            output.bytes(source_lease_identity)
        }
        Kind::ReplicaBootstrapAbandoned {
            receipt_identity,
            reason,
            source_lease_identity,
        } => {
            output.u8(27)?;
            output.bytes(receipt_identity)?;
            output.string(reason)?;
            output.bytes(source_lease_identity)
        }
        Kind::ReplicaPromotionFenceRecorded {
            authorization_plan_fingerprint,
            execution_plan_fingerprint,
            fence_identity,
            promoted_epoch,
        } => {
            output.u8(24)?;
            output.bytes(authorization_plan_fingerprint)?;
            output.bytes(execution_plan_fingerprint)?;
            output.bytes(fence_identity)?;
            output.u64(*promoted_epoch)
        }
        Kind::ReplicaPromotionRecorded {
            authorization_plan_fingerprint,
            execution_plan_fingerprint,
            receipt_identity,
            fence_identity,
            promoted_epoch,
        } => {
            output.u8(25)?;
            output.bytes(authorization_plan_fingerprint)?;
            output.bytes(execution_plan_fingerprint)?;
            output.bytes(receipt_identity)?;
            output.bytes(fence_identity)?;
            output.u64(*promoted_epoch)
        }
        Kind::ReplicaPromotionPublished {
            receipt_identity,
            verification_identity,
            publication_identity,
            target_identity,
            promoted_epoch,
        } => {
            output.u8(28)?;
            output.bytes(receipt_identity)?;
            output.bytes(verification_identity)?;
            output.bytes(publication_identity)?;
            output.bytes(target_identity)?;
            output.u64(*promoted_epoch)
        }
        Kind::ReplicaPromotionReadmitted {
            publication_identity,
            serve_lease_identity,
            serving_epoch,
        } => {
            output.u8(29)?;
            output.bytes(publication_identity)?;
            output.bytes(serve_lease_identity)?;
            output.u64(*serving_epoch)
        }
        Kind::OldPrimaryRejoinPlanned {
            promotion_receipt_identity,
            rejoin_plan_fingerprint,
            disposition_tag,
        } => {
            output.u8(30)?;
            output.bytes(promotion_receipt_identity)?;
            output.bytes(rejoin_plan_fingerprint)?;
            output.u8(*disposition_tag)
        }
        Kind::OldPrimaryRejoinCompleted {
            rejoin_plan_fingerprint,
            rejoin_receipt_identity,
            forensic_retention_identity,
            rebootstrap_target_identity,
            disposition_tag,
        } => {
            output.u8(31)?;
            output.bytes(rejoin_plan_fingerprint)?;
            output.bytes(rejoin_receipt_identity)?;
            output.bytes(forensic_retention_identity)?;
            output.bytes(rebootstrap_target_identity)?;
            output.u8(*disposition_tag)
        }
        Kind::RepairDispositionRecorded {
            plan_fingerprint,
            disposition_tag,
            disposition_basis,
        } => {
            output.u8(15)?;
            output.bytes(plan_fingerprint)?;
            output.u8(*disposition_tag)?;
            output.bytes(disposition_basis)
        }
        Kind::RecoveryStagingCompleted {
            authorization_identity,
            plan_fingerprint,
            execution_plan_fingerprint,
            staged_media_identity,
        } => {
            output.u8(18)?;
            output.bytes(authorization_identity)?;
            output.bytes(plan_fingerprint)?;
            output.bytes(execution_plan_fingerprint)?;
            output.bytes(staged_media_identity)
        }
    }
}

const fn workflow_tag(kind: OperationalWorkflowKind) -> u8 {
    match kind {
        OperationalWorkflowKind::OfflineInspection => 1,
        OperationalWorkflowKind::Backup => 2,
        OperationalWorkflowKind::Restore => 3,
        OperationalWorkflowKind::PointInTimeRecovery => 4,
        OperationalWorkflowKind::Rollback => 5,
        OperationalWorkflowKind::Repair => 6,
        OperationalWorkflowKind::ReplicaBootstrap => 7,
        OperationalWorkflowKind::ReplicaPromotion => 8,
        OperationalWorkflowKind::ForensicAcquisition => 9,
    }
}
