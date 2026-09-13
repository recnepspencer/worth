use std::collections::HashMap;

use super::{
    IndeterminateRecoveryStagingHandle, OperationalControlHistoryViolationKind,
    OperationalOperationId, OperationalOwnerReceiptKind, OperationalWorkflowKind,
    RecoveryStagingOperationKind,
};

pub(super) struct ReplayedRecoveryStaging {
    authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
    operation_kind: RecoveryStagingOperationKind,
    authorization_identity: [u8; 32],
    plan_fingerprint: [u8; 32],
    execution_plan_fingerprint: [u8; 32],
    backend_owner_receipt: Option<[u8; 32]>,
    recovery_owner_receipt: Option<[u8; 32]>,
    completed_media_identity: Option<[u8; 32]>,
}

pub(super) fn observe_authorized_staging(
    stages: &mut HashMap<OperationalOperationId, ReplayedRecoveryStaging>,
    operation: &OperationalOperationId,
    authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
    operation_tag: u8,
    authorization_identity: [u8; 32],
    plan_fingerprint: [u8; 32],
    execution_plan_fingerprint: Option<[u8; 32]>,
) -> Result<(), OperationalControlHistoryViolationKind> {
    let operation_kind = match operation_tag {
        1 => RecoveryStagingOperationKind::BackupRestore,
        2 => RecoveryStagingOperationKind::PointInTimeRecovery,
        3 => RecoveryStagingOperationKind::Rollback,
        _ => return Ok(()),
    };
    let execution_plan_fingerprint = execution_plan_fingerprint
        .ok_or(OperationalControlHistoryViolationKind::RecoveryStagingBindingMismatch)?;
    if stages.contains_key(operation) {
        return Err(OperationalControlHistoryViolationKind::DuplicateRecoveryStaging);
    }
    stages
        .try_reserve(1)
        .map_err(|_| OperationalControlHistoryViolationKind::DuplicateRecoveryStaging)?;
    stages.insert(
        operation.clone(),
        ReplayedRecoveryStaging {
            authority_identity,
            operation_kind,
            authorization_identity,
            plan_fingerprint,
            execution_plan_fingerprint,
            backend_owner_receipt: None,
            recovery_owner_receipt: None,
            completed_media_identity: None,
        },
    );
    Ok(())
}

pub(super) fn observe_owner_receipt(
    stages: &mut HashMap<OperationalOperationId, ReplayedRecoveryStaging>,
    operation: &OperationalOperationId,
    workflow: OperationalWorkflowKind,
    plan_fingerprint: [u8; 32],
    receipt_fingerprint: [u8; 32],
    owner_kind: OperationalOwnerReceiptKind,
) -> Result<(), OperationalControlHistoryViolationKind> {
    let stage = stages
        .get_mut(operation)
        .ok_or(OperationalControlHistoryViolationKind::RecoveryOwnerReceiptBeforeAuthorization)?;
    if !stage.matches_workflow(workflow) {
        return Err(OperationalControlHistoryViolationKind::RecoveryOwnerReceiptWorkflowMismatch);
    }
    if stage.plan_fingerprint != plan_fingerprint || receipt_fingerprint == [0; 32] {
        return Err(OperationalControlHistoryViolationKind::RecoveryOwnerReceiptBindingMismatch);
    }
    if stage.completed_media_identity.is_some() {
        return Err(OperationalControlHistoryViolationKind::RecoveryStagingBindingMismatch);
    }
    let slot = match owner_kind {
        OperationalOwnerReceiptKind::Backend => &mut stage.backend_owner_receipt,
        OperationalOwnerReceiptKind::Recovery => &mut stage.recovery_owner_receipt,
    };
    match slot {
        None => {
            *slot = Some(receipt_fingerprint);
            Ok(())
        }
        Some(observed) if *observed == receipt_fingerprint => Ok(()),
        Some(_) => Err(OperationalControlHistoryViolationKind::DuplicateRecoveryOwnerReceipt),
    }
}

pub(super) fn observe_staging_completed(
    stages: &mut HashMap<OperationalOperationId, ReplayedRecoveryStaging>,
    operation: &OperationalOperationId,
    authorization_identity: [u8; 32],
    plan_fingerprint: [u8; 32],
    execution_plan_fingerprint: [u8; 32],
    staged_media_identity: [u8; 32],
) -> Result<(), OperationalControlHistoryViolationKind> {
    let stage = stages.get_mut(operation).ok_or(
        OperationalControlHistoryViolationKind::RecoveryStagingCompletionBeforeAuthorization,
    )?;
    if stage.authorization_identity != authorization_identity
        || stage.plan_fingerprint != plan_fingerprint
        || stage.execution_plan_fingerprint != execution_plan_fingerprint
    {
        return Err(OperationalControlHistoryViolationKind::RecoveryStagingBindingMismatch);
    }
    if stage.backend_owner_receipt.is_none() || stage.recovery_owner_receipt.is_none() {
        return Err(
            OperationalControlHistoryViolationKind::RecoveryStagingCompletionBeforeOwnerReceipts,
        );
    }
    if staged_media_identity == [0; 32] || stage.completed_media_identity.is_some() {
        return Err(OperationalControlHistoryViolationKind::RecoveryStagingBindingMismatch);
    }
    stage.completed_media_identity = Some(staged_media_identity);
    Ok(())
}

impl ReplayedRecoveryStaging {
    fn matches_workflow(&self, workflow: OperationalWorkflowKind) -> bool {
        matches!(
            (self.operation_kind, workflow),
            (
                RecoveryStagingOperationKind::BackupRestore,
                OperationalWorkflowKind::Restore
            ) | (
                RecoveryStagingOperationKind::PointInTimeRecovery,
                OperationalWorkflowKind::PointInTimeRecovery
            ) | (
                RecoveryStagingOperationKind::Rollback,
                OperationalWorkflowKind::Rollback
            )
        )
    }

    pub(super) fn pending_handle(
        self,
        operation_id: OperationalOperationId,
    ) -> IndeterminateRecoveryStagingHandle {
        IndeterminateRecoveryStagingHandle::new(
            operation_id,
            self.authority_identity,
            self.operation_kind,
            self.authorization_identity,
            self.plan_fingerprint,
            self.execution_plan_fingerprint,
            self.completed_media_identity,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staging_completion_requires_both_durable_owner_receipts() {
        let (mut stages, operation) = authorized_restore();
        let denied =
            observe_staging_completed(&mut stages, &operation, [2; 32], [3; 32], [4; 32], [7; 32]);
        assert_eq!(
            denied,
            Err(
                OperationalControlHistoryViolationKind::RecoveryStagingCompletionBeforeOwnerReceipts
            )
        );
    }

    #[test]
    fn owner_receipt_retry_is_exactly_idempotent_but_conflict_is_denied() {
        let (mut stages, operation) = authorized_restore();
        for receipt in [[8; 32], [8; 32]] {
            observe_owner_receipt(
                &mut stages,
                &operation,
                OperationalWorkflowKind::Restore,
                [3; 32],
                receipt,
                OperationalOwnerReceiptKind::Recovery,
            )
            .expect("an exact durable retry is idempotent");
        }
        assert_eq!(
            observe_owner_receipt(
                &mut stages,
                &operation,
                OperationalWorkflowKind::Restore,
                [3; 32],
                [9; 32],
                OperationalOwnerReceiptKind::Recovery,
            ),
            Err(OperationalControlHistoryViolationKind::DuplicateRecoveryOwnerReceipt)
        );
    }

    #[test]
    fn owner_receipt_cannot_cross_workflow_meaning() {
        let (mut stages, operation) = authorized_restore();
        assert_eq!(
            observe_owner_receipt(
                &mut stages,
                &operation,
                OperationalWorkflowKind::Rollback,
                [3; 32],
                [8; 32],
                OperationalOwnerReceiptKind::Recovery,
            ),
            Err(OperationalControlHistoryViolationKind::RecoveryOwnerReceiptWorkflowMismatch)
        );
    }

    fn authorized_restore() -> (
        HashMap<OperationalOperationId, ReplayedRecoveryStaging>,
        OperationalOperationId,
    ) {
        let operation = OperationalOperationId::new("restore-owner-receipt-test").unwrap();
        let mut stages = HashMap::new();
        observe_authorized_staging(
            &mut stages,
            &operation,
            worth_store_authority::StoreCurrentAuthorityIdentity::from_persisted_fingerprint(
                [1; 32],
            ),
            1,
            [2; 32],
            [3; 32],
            Some([4; 32]),
        )
        .unwrap();
        (stages, operation)
    }
}
