pub(crate) use super::control_record_kind::OperationalControlRecordKind;
use super::{BackupMaterializationRecoveryPlan, OperationalOperationId, OperationalTransitionId};
use worth_store_authority::StoreCurrentAuthorityIdentity;
use worth_store_physical_backend::ControlRecoveryObjectHandle;
use worth_store_physical_isolation::{
    BackupCutRecoveryRecord, BackupReachabilityLeaseReleaseRecord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationalControlRecord {
    pub(super) authority_identity: StoreCurrentAuthorityIdentity,
    pub(super) operation_id: OperationalOperationId,
    pub(super) transition_id: OperationalTransitionId,
    pub(super) kind: OperationalControlRecordKind,
}

impl OperationalControlRecord {
    #[cfg(test)]
    pub(crate) fn workflow_opened(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        workflow: super::OperationalWorkflowKind,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::WorkflowOpened { workflow },
        }
    }

    pub(crate) fn source_lease_persisted(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        recovery: BackupCutRecoveryRecord,
        recovery_object: ControlRecoveryObjectHandle,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::SourceLeasePersisted {
                recovery: Box::new(recovery),
                recovery_object,
            },
        }
    }

    pub(crate) fn backup_materialization_recorded(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        materialized: &worth_store_physical_format::MaterializedBackupBundle,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::BackupMaterializationRecorded {
                manifest_digest: materialized.manifest_digest(),
            },
        }
    }

    pub(crate) fn backup_materialization_opened(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        plan: BackupMaterializationRecoveryPlan,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::BackupMaterializationOpened { plan },
        }
    }

    pub(crate) fn independent_backup_verification_recorded_and_source_lease_released(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        verified: &worth_store_offline_verifier::StructurallyVerifiedBackupBundle,
        release: BackupReachabilityLeaseReleaseRecord,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::IndependentBackupVerificationRecordedAndSourceLeaseReleased {
                verification_identity: verified.verification_identity(),
                release,
            },
        }
    }

    pub(crate) fn backup_abandoned(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        reason: impl Into<String>,
        released_source_lease: BackupReachabilityLeaseReleaseRecord,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::BackupAbandoned {
                reason: reason.into(),
                released_source_lease,
            },
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn authorization_consumed(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        authorization_identity: [u8; 32],
        plan_fingerprint: [u8; 32],
        operation_tag: u8,
        execution_plan_fingerprint: Option<[u8; 32]>,
        assertion_identity: [u8; 32],
        expires_at: u64,
        replay_same_operation_identity: bool,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::AuthorizationConsumed {
                authorization_identity,
                plan_fingerprint,
                operation_tag,
                execution_plan_fingerprint,
                assertion_identity,
                expires_at,
                replay_same_operation_identity,
            },
        }
    }

    pub(crate) fn recovery_staging_completed(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        authorization_identity: [u8; 32],
        plan_fingerprint: [u8; 32],
        execution_plan_fingerprint: [u8; 32],
        staged_media_identity: [u8; 32],
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id: OperationalTransitionId::recovery_staging_completed(),
            kind: OperationalControlRecordKind::RecoveryStagingCompleted {
                authorization_identity,
                plan_fingerprint,
                execution_plan_fingerprint,
                staged_media_identity,
            },
        }
    }

    pub(crate) fn repair_execution_opened(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        authorization_identity: [u8; 32],
        plan_fingerprint: [u8; 32],
        owner_node_count: u64,
        topology_tag: u8,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::RepairExecutionOpened {
                authorization_identity,
                plan_fingerprint,
                owner_node_count,
                topology_tag,
            },
        }
    }

    pub(crate) fn repair_owner_receipt_persisted(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        plan_fingerprint: [u8; 32],
        node_fingerprint: [u8; 32],
        receipt_fingerprint: [u8; 32],
        owner_tag: u8,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::RepairOwnerReceiptPersisted {
                plan_fingerprint,
                node_fingerprint,
                receipt_fingerprint,
                owner_tag,
            },
        }
    }

    pub(crate) fn repair_owner_effect_started(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        plan_fingerprint: [u8; 32],
        node_fingerprint: [u8; 32],
        owner_tag: u8,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::RepairOwnerEffectStarted {
                plan_fingerprint,
                node_fingerprint,
                owner_tag,
            },
        }
    }

    pub(crate) fn operational_owner_receipt_persisted(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        workflow: super::OperationalWorkflowKind,
        plan_fingerprint: [u8; 32],
        receipt_fingerprint: [u8; 32],
        owner_kind: super::OperationalOwnerReceiptKind,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::OperationalOwnerReceiptPersisted {
                workflow,
                plan_fingerprint,
                receipt_fingerprint,
                owner_kind,
            },
        }
    }

    pub(crate) fn repair_disposition_recorded(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        plan_fingerprint: [u8; 32],
        disposition_tag: u8,
        disposition_basis: [u8; 32],
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind: OperationalControlRecordKind::RepairDispositionRecorded {
                plan_fingerprint,
                disposition_tag,
                disposition_basis,
            },
        }
    }

    pub const fn authority_identity(&self) -> StoreCurrentAuthorityIdentity {
        self.authority_identity
    }
    pub const fn operation_id(&self) -> &OperationalOperationId {
        &self.operation_id
    }
    pub const fn transition_id(&self) -> &OperationalTransitionId {
        &self.transition_id
    }
    pub const fn kind(&self) -> &OperationalControlRecordKind {
        &self.kind
    }

    pub(crate) const fn from_persisted_parts(
        authority_identity: StoreCurrentAuthorityIdentity,
        operation_id: OperationalOperationId,
        transition_id: OperationalTransitionId,
        kind: OperationalControlRecordKind,
    ) -> Self {
        Self {
            authority_identity,
            operation_id,
            transition_id,
            kind,
        }
    }

    pub(crate) fn into_replay_parts(
        self,
    ) -> (OperationalOperationId, OperationalControlRecordKind) {
        (self.operation_id, self.kind)
    }
}
