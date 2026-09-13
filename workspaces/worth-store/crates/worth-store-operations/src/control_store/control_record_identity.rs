use sha2::{Digest, Sha256};

use super::{OperationalControlRecord, OperationalControlRecordKind};

impl OperationalControlRecord {
    /// Stable identity of the complete durable control artifact.
    ///
    /// Audit, formal refinement, and certification consume this owner-derived
    /// identity so none of those readers can accidentally omit a payload field.
    pub fn stable_fingerprint(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"worth-store-operational-control-artifact-v2");
        digest.update(self.authority_identity().fingerprint());
        bind_bytes(&mut digest, self.operation_id().as_str().as_bytes());
        bind_bytes(&mut digest, self.transition_id().as_str().as_bytes());
        fingerprint_kind(self.kind(), &mut digest);
        digest.finalize().into()
    }
}

fn fingerprint_kind(kind: &OperationalControlRecordKind, digest: &mut Sha256) {
    digest.update([stable_kind_tag(kind)]);
    use OperationalControlRecordKind as Kind;
    match kind {
        Kind::WorkflowOpened { workflow } => digest.update([workflow_tag(*workflow)]),
        Kind::SourceLeasePersisted {
            recovery,
            recovery_object,
        } => {
            bind_bytes(digest, recovery.recovery_bytes());
            digest.update(recovery_object.digest());
            digest.update(recovery_object.bytes().to_be_bytes());
        }
        Kind::BackupMaterializationOpened { plan } => {
            digest.update(plan.cut_identity());
            digest.update((plan.buffer_bytes() as u64).to_be_bytes());
            let (platform, path) = plan
                .persisted_path()
                .expect("admitted recovery plans retain their canonical path encoding");
            digest.update([platform]);
            bind_bytes(digest, &path);
        }
        Kind::BackupMaterializationRecorded { manifest_digest } => digest.update(manifest_digest),
        Kind::IndependentBackupVerificationRecordedAndSourceLeaseReleased {
            verification_identity,
            release,
        } => {
            digest.update(verification_identity);
            bind_bytes(digest, release.recovery_bytes());
        }
        Kind::BackupAbandoned {
            reason,
            released_source_lease,
        } => {
            bind_bytes(digest, reason.as_bytes());
            bind_bytes(digest, released_source_lease.recovery_bytes());
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
            digest.update(authorization_identity);
            digest.update(plan_fingerprint);
            digest.update([*operation_tag]);
            fingerprint_optional_identity(digest, *execution_plan_fingerprint);
            digest.update(assertion_identity);
            digest.update(expires_at.to_be_bytes());
            digest.update([u8::from(*replay_same_operation_identity)]);
        }
        Kind::RepairExecutionOpened {
            authorization_identity,
            plan_fingerprint,
            owner_node_count,
            topology_tag,
        } => {
            digest.update(authorization_identity);
            digest.update(plan_fingerprint);
            digest.update(owner_node_count.to_be_bytes());
            digest.update([*topology_tag]);
        }
        Kind::RepairOwnerReceiptPersisted {
            plan_fingerprint,
            node_fingerprint,
            receipt_fingerprint,
            owner_tag,
        } => {
            digest.update(plan_fingerprint);
            digest.update(node_fingerprint);
            digest.update(receipt_fingerprint);
            digest.update([*owner_tag]);
        }
        Kind::RepairOwnerEffectStarted {
            plan_fingerprint,
            node_fingerprint,
            owner_tag,
        } => {
            digest.update(plan_fingerprint);
            digest.update(node_fingerprint);
            digest.update([*owner_tag]);
        }
        Kind::OperationalOwnerReceiptPersisted {
            workflow,
            plan_fingerprint,
            receipt_fingerprint,
            owner_kind,
        } => {
            digest.update([workflow_tag(*workflow)]);
            digest.update(plan_fingerprint);
            digest.update(receipt_fingerprint);
            digest.update([owner_kind.tag()]);
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
            digest.update(authorization_plan_fingerprint);
            digest.update(execution_plan_fingerprint);
            digest.update(receipt_identity);
            digest.update(durable_target_identity);
            digest.update(source_lease_identity);
            digest.update(source_bytes_read.to_be_bytes());
            digest.update(output_bytes_written.to_be_bytes());
            digest.update(backend_requests.to_be_bytes());
            digest.update(maximum_resident_buffer_bytes.to_be_bytes());
        }
        Kind::ReplicaBootstrapCompleted {
            receipt_identity,
            verification_identity,
            source_lease_identity,
        } => {
            digest.update(receipt_identity);
            digest.update(verification_identity);
            digest.update(source_lease_identity);
        }
        Kind::ReplicaBootstrapAbandoned {
            receipt_identity,
            reason,
            source_lease_identity,
        } => {
            digest.update(receipt_identity);
            bind_bytes(digest, reason.as_bytes());
            digest.update(source_lease_identity);
        }
        Kind::ReplicaPromotionFenceRecorded {
            authorization_plan_fingerprint,
            execution_plan_fingerprint,
            fence_identity,
            promoted_epoch,
        } => {
            digest.update(authorization_plan_fingerprint);
            digest.update(execution_plan_fingerprint);
            digest.update(fence_identity);
            digest.update(promoted_epoch.to_be_bytes());
        }
        Kind::ReplicaPromotionRecorded {
            authorization_plan_fingerprint,
            execution_plan_fingerprint,
            receipt_identity,
            fence_identity,
            promoted_epoch,
        } => {
            digest.update(authorization_plan_fingerprint);
            digest.update(execution_plan_fingerprint);
            digest.update(receipt_identity);
            digest.update(fence_identity);
            digest.update(promoted_epoch.to_be_bytes());
        }
        Kind::ReplicaPromotionPublished {
            receipt_identity,
            verification_identity,
            publication_identity,
            target_identity,
            promoted_epoch,
        } => {
            digest.update(receipt_identity);
            digest.update(verification_identity);
            digest.update(publication_identity);
            digest.update(target_identity);
            digest.update(promoted_epoch.to_be_bytes());
        }
        Kind::ReplicaPromotionReadmitted {
            publication_identity,
            serve_lease_identity,
            serving_epoch,
        } => {
            digest.update(publication_identity);
            digest.update(serve_lease_identity);
            digest.update(serving_epoch.to_be_bytes());
        }
        Kind::OldPrimaryRejoinPlanned {
            promotion_receipt_identity,
            rejoin_plan_fingerprint,
            disposition_tag,
        } => {
            digest.update(promotion_receipt_identity);
            digest.update(rejoin_plan_fingerprint);
            digest.update([*disposition_tag]);
        }
        Kind::OldPrimaryRejoinCompleted {
            rejoin_plan_fingerprint,
            rejoin_receipt_identity,
            forensic_retention_identity,
            rebootstrap_target_identity,
            disposition_tag,
        } => {
            digest.update(rejoin_plan_fingerprint);
            digest.update(rejoin_receipt_identity);
            digest.update(forensic_retention_identity);
            digest.update(rebootstrap_target_identity);
            digest.update([*disposition_tag]);
        }
        Kind::RepairDispositionRecorded {
            plan_fingerprint,
            disposition_tag,
            disposition_basis,
        } => {
            digest.update(plan_fingerprint);
            digest.update([*disposition_tag]);
            digest.update(disposition_basis);
        }
        Kind::RecoveryStagingCompleted {
            authorization_identity,
            plan_fingerprint,
            execution_plan_fingerprint,
            staged_media_identity,
        } => {
            digest.update(authorization_identity);
            digest.update(plan_fingerprint);
            digest.update(execution_plan_fingerprint);
            digest.update(staged_media_identity);
        }
    }
}

const fn workflow_tag(workflow: super::OperationalWorkflowKind) -> u8 {
    use super::OperationalWorkflowKind as Workflow;
    match workflow {
        Workflow::OfflineInspection => 1,
        Workflow::Backup => 2,
        Workflow::Restore => 3,
        Workflow::PointInTimeRecovery => 4,
        Workflow::Rollback => 5,
        Workflow::Repair => 6,
        Workflow::ReplicaBootstrap => 7,
        Workflow::ReplicaPromotion => 8,
        Workflow::ForensicAcquisition => 9,
    }
}

const fn stable_kind_tag(kind: &OperationalControlRecordKind) -> u8 {
    use OperationalControlRecordKind as Kind;
    match kind {
        Kind::WorkflowOpened { .. } => 0,
        Kind::SourceLeasePersisted { .. } => 1,
        Kind::BackupMaterializationOpened { .. } => 2,
        Kind::BackupMaterializationRecorded { .. } => 3,
        Kind::IndependentBackupVerificationRecordedAndSourceLeaseReleased { .. } => 4,
        Kind::BackupAbandoned { .. } => 5,
        Kind::AuthorizationConsumed { .. } => 6,
        Kind::RepairExecutionOpened { .. } => 7,
        Kind::RepairOwnerReceiptPersisted { .. } => 8,
        Kind::RepairOwnerEffectStarted { .. } => 9,
        Kind::OperationalOwnerReceiptPersisted { .. } => 10,
        Kind::ReplicaBootstrapTransferRecorded { .. } => 11,
        Kind::ReplicaBootstrapCompleted { .. } => 12,
        Kind::ReplicaBootstrapAbandoned { .. } => 13,
        Kind::ReplicaPromotionFenceRecorded { .. } => 14,
        Kind::ReplicaPromotionRecorded { .. } => 15,
        Kind::ReplicaPromotionPublished { .. } => 16,
        Kind::ReplicaPromotionReadmitted { .. } => 17,
        Kind::OldPrimaryRejoinPlanned { .. } => 18,
        Kind::OldPrimaryRejoinCompleted { .. } => 19,
        Kind::RepairDispositionRecorded { .. } => 20,
        Kind::RecoveryStagingCompleted { .. } => 21,
    }
}

fn bind_bytes(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_be_bytes());
    digest.update(bytes);
}

fn fingerprint_optional_identity(digest: &mut Sha256, identity: Option<[u8; 32]>) {
    match identity {
        Some(identity) => {
            digest.update([1]);
            digest.update(identity);
        }
        None => digest.update([0]),
    }
}

#[cfg(test)]
#[path = "control_record_identity_tests.rs"]
mod tests;
