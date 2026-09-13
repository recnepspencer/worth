use super::integrity_classification::{
    IntegrityOperationalRepairOwner, IntegrityRepairClassificationReceipt,
};
use crate::workflow::restore::{
    BackupRestoreReplayDenial, BackupRestoreReplayOwner, RecoveredBackupFrontierReceipt,
};
use worth_store_physical_backend::{
    ClosedNonCurrentStagingMedia, NonCurrentStagingExecutionDenial,
    NonCurrentStagingExecutionReceipt, NonCurrentStagingOwnerExecutionDenial,
    PhysicalRecoveryStagingOwner,
};

use crate::authorization::StagingAuthorizationContinuation;
use crate::{
    AuthorizationConsumptionReceipt, OperationalOperationId, OperationalSecurityScope,
    OwnerPlanNodeIdentity,
};

use super::authority_receipt_persistence::{
    backend_receipt, blob_receipt, integrity_receipt, layout_receipt, persist, recovery_receipt,
};
use super::{
    ExecutionReadyAuthorityAffectingRepair, RepairExecutionBoundary, RepairExecutionBoundaryMoment,
    RepairExecutionControlPort, RepairExecutionInterrupted, UninterruptedRepairExecution,
};
use super::{RepairExecutionDisposition, RepairJournalDenial};

#[derive(Debug)]
pub enum AuthorityAffectingRepairExecutionDenial {
    Authorization(crate::StagingAuthorizationContinuationDenial),
    Backend(NonCurrentStagingExecutionDenial),
    Recovery(BackupRestoreReplayDenial),
    Layout(worth_store_layout_indexes::LayoutRepairConsequenceDenial),
    Blob(worth_store_blob_chunks::BlobRepairConsequenceDenial),
    Journal(RepairJournalDenial),
    RecoveredReceiptMismatch { node: OwnerPlanNodeIdentity },
    Interrupted(RepairExecutionInterrupted),
}

enum StagedRepairOwnerDenial {
    Recovery(BackupRestoreReplayDenial),
    Interrupted(RepairExecutionInterrupted),
    Journal(RepairJournalDenial),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedAuthorityAffectingRepair {
    operation_id: OperationalOperationId,
    authorization: AuthorizationConsumptionReceipt,
    integrity: IntegrityRepairClassificationReceipt,
    backend: NonCurrentStagingExecutionReceipt,
    recovery: RecoveredBackupFrontierReceipt,
    layout: Option<worth_store_layout_indexes::LayoutRepairConsequenceReceipt>,
    blob: Option<worth_store_blob_chunks::BlobRepairConsequenceReceipt>,
    staging_authority: worth_store_authority::StoreCurrentAuthorityIdentity,
    security_scope: OperationalSecurityScope,
}

impl ExecutedAuthorityAffectingRepair {
    pub const fn operation_id(&self) -> &OperationalOperationId {
        &self.operation_id
    }
    pub const fn authorization(&self) -> AuthorizationConsumptionReceipt {
        self.authorization
    }
    pub const fn staged_media(&self) -> &ClosedNonCurrentStagingMedia {
        self.backend.media()
    }
    pub const fn backend(&self) -> &NonCurrentStagingExecutionReceipt {
        &self.backend
    }
    pub const fn recovered_frontier(&self) -> RecoveredBackupFrontierReceipt {
        self.recovery
    }
    pub const fn integrity(&self) -> IntegrityRepairClassificationReceipt {
        self.integrity
    }
    pub const fn layout(
        &self,
    ) -> Option<worth_store_layout_indexes::LayoutRepairConsequenceReceipt> {
        self.layout
    }
    pub const fn blob(&self) -> Option<worth_store_blob_chunks::BlobRepairConsequenceReceipt> {
        self.blob
    }
    pub const fn staging_authority(&self) -> worth_store_authority::StoreCurrentAuthorityIdentity {
        self.staging_authority
    }
    pub const fn security_scope(&self) -> OperationalSecurityScope {
        self.security_scope
    }
}

impl ExecutionReadyAuthorityAffectingRepair<'_> {
    pub fn execute<Ports>(
        self,
        ports: &Ports,
    ) -> Result<ExecutedAuthorityAffectingRepair, AuthorityAffectingRepairExecutionDenial>
    where
        Ports:
            crate::StagingAuthorizationContinuationPort + crate::workflow::StagedWalApplicationPort,
    {
        self.execute_with_control(ports, &UninterruptedRepairExecution)
    }

    pub fn execute_with_control<Ports>(
        mut self,
        ports: &Ports,
        control: &impl RepairExecutionControlPort,
    ) -> Result<ExecutedAuthorityAffectingRepair, AuthorityAffectingRepairExecutionDenial>
    where
        Ports:
            crate::StagingAuthorizationContinuationPort + crate::workflow::StagedWalApplicationPort,
    {
        self.journal
            .begin_owner_effect(self.nodes.integrity, 2)
            .map_err(AuthorityAffectingRepairExecutionDenial::Journal)?;
        observe(
            control,
            self.nodes.integrity,
            RepairExecutionBoundaryMoment::BeforeOwnerEffect,
        )?;
        let integrity = IntegrityOperationalRepairOwner::execute(&self.integrity);
        observe(
            control,
            self.nodes.integrity,
            RepairExecutionBoundaryMoment::AfterOwnerEffectBeforeReceipt,
        )?;
        persist(
            &mut self.journal,
            self.nodes.integrity,
            integrity_receipt(integrity),
            2,
        )?;
        observe(
            control,
            self.nodes.integrity,
            RepairExecutionBoundaryMoment::AfterReceiptPersistence,
        )?;
        let mut continuation = StagingAuthorizationContinuation::new(self.authorization, ports);
        self.journal
            .begin_owner_effect(self.nodes.backend, 1)
            .map_err(AuthorityAffectingRepairExecutionDenial::Journal)?;
        observe(
            control,
            self.nodes.backend,
            RepairExecutionBoundaryMoment::BeforeOwnerEffect,
        )?;
        let staged = PhysicalRecoveryStagingOwner::execute_lowered_guarded_with_owner_effect(
            self.backend,
            |boundary| continuation.admit(boundary),
            |staging| {
                control
                    .observe(RepairExecutionBoundary::new(
                        self.nodes.backend,
                        RepairExecutionBoundaryMoment::AfterOwnerEffectBeforeReceipt,
                    ))
                    .map_err(StagedRepairOwnerDenial::Interrupted)?;
                self.journal
                    .begin_owner_effect(self.nodes.recovery, 3)
                    .map_err(StagedRepairOwnerDenial::Journal)?;
                control
                    .observe(RepairExecutionBoundary::new(
                        self.nodes.recovery,
                        RepairExecutionBoundaryMoment::BeforeOwnerEffect,
                    ))
                    .map_err(StagedRepairOwnerDenial::Interrupted)?;
                let receipt = BackupRestoreReplayOwner::execute(self.recovery, staging, ports)
                    .map_err(StagedRepairOwnerDenial::Recovery)?;
                control
                    .observe(RepairExecutionBoundary::new(
                        self.nodes.recovery,
                        RepairExecutionBoundaryMoment::AfterOwnerEffectBeforeReceipt,
                    ))
                    .map_err(StagedRepairOwnerDenial::Interrupted)?;
                Ok(receipt)
            },
        );
        let (backend, recovery) = match staged {
            Ok(receipts) => receipts,
            Err(NonCurrentStagingOwnerExecutionDenial::Backend(
                NonCurrentStagingExecutionDenial::ContinuationDenied { .. },
            )) => {
                return Err(AuthorityAffectingRepairExecutionDenial::Authorization(
                    continuation
                        .denial()
                        .expect("a denied gate records its cause"),
                ))
            }
            Err(NonCurrentStagingOwnerExecutionDenial::Backend(denial)) => {
                return Err(AuthorityAffectingRepairExecutionDenial::Backend(denial))
            }
            Err(NonCurrentStagingOwnerExecutionDenial::Owner(
                StagedRepairOwnerDenial::Recovery(denial),
            )) => return Err(AuthorityAffectingRepairExecutionDenial::Recovery(denial)),
            Err(NonCurrentStagingOwnerExecutionDenial::Owner(
                StagedRepairOwnerDenial::Interrupted(denial),
            )) => return Err(AuthorityAffectingRepairExecutionDenial::Interrupted(denial)),
            Err(NonCurrentStagingOwnerExecutionDenial::Owner(
                StagedRepairOwnerDenial::Journal(denial),
            )) => return Err(AuthorityAffectingRepairExecutionDenial::Journal(denial)),
        };
        persist(
            &mut self.journal,
            self.nodes.backend,
            backend_receipt(&backend),
            1,
        )?;
        observe(
            control,
            self.nodes.backend,
            RepairExecutionBoundaryMoment::AfterReceiptPersistence,
        )?;
        persist(
            &mut self.journal,
            self.nodes.recovery,
            recovery_receipt(recovery),
            3,
        )?;
        observe(
            control,
            self.nodes.recovery,
            RepairExecutionBoundaryMoment::AfterReceiptPersistence,
        )?;
        let layout = match self.layout.as_ref() {
            Some(plan) => {
                let node = self.nodes.layout.expect("plan has owner node");
                self.journal
                    .begin_owner_effect(node, 5)
                    .map_err(AuthorityAffectingRepairExecutionDenial::Journal)?;
                observe(
                    control,
                    node,
                    RepairExecutionBoundaryMoment::BeforeOwnerEffect,
                )?;
                let receipt = worth_store_layout_indexes::LayoutRepairConsequenceOwner::execute(
                    plan,
                    backend.media(),
                )
                .map_err(AuthorityAffectingRepairExecutionDenial::Layout)?;
                observe(
                    control,
                    node,
                    RepairExecutionBoundaryMoment::AfterOwnerEffectBeforeReceipt,
                )?;
                persist(&mut self.journal, node, layout_receipt(receipt), 5)?;
                observe(
                    control,
                    node,
                    RepairExecutionBoundaryMoment::AfterReceiptPersistence,
                )?;
                Some(receipt)
            }
            None => None,
        };
        let blob = match self.blob.as_ref() {
            Some(plan) => {
                let node = self.nodes.blob.expect("plan has owner node");
                self.journal
                    .begin_owner_effect(node, 6)
                    .map_err(AuthorityAffectingRepairExecutionDenial::Journal)?;
                observe(
                    control,
                    node,
                    RepairExecutionBoundaryMoment::BeforeOwnerEffect,
                )?;
                let receipt = worth_store_blob_chunks::BlobRepairConsequenceOwner::execute(
                    plan,
                    backend.media(),
                )
                .map_err(AuthorityAffectingRepairExecutionDenial::Blob)?;
                observe(
                    control,
                    node,
                    RepairExecutionBoundaryMoment::AfterOwnerEffectBeforeReceipt,
                )?;
                persist(&mut self.journal, node, blob_receipt(receipt), 6)?;
                observe(
                    control,
                    node,
                    RepairExecutionBoundaryMoment::AfterReceiptPersistence,
                )?;
                Some(receipt)
            }
            None => None,
        };
        let completion_basis = self
            .journal
            .completion_basis()
            .map_err(AuthorityAffectingRepairExecutionDenial::Journal)?;
        self.journal
            .close(RepairExecutionDisposition::Executed, completion_basis)
            .map_err(AuthorityAffectingRepairExecutionDenial::Journal)?;
        Ok(ExecutedAuthorityAffectingRepair {
            operation_id: self.operation_id,
            authorization: self.authorization,
            integrity,
            backend,
            recovery,
            layout,
            blob,
            staging_authority: self.staging_authority,
            security_scope: self.security_scope,
        })
    }
}

fn observe(
    control: &impl RepairExecutionControlPort,
    node: OwnerPlanNodeIdentity,
    moment: RepairExecutionBoundaryMoment,
) -> Result<(), AuthorityAffectingRepairExecutionDenial> {
    control
        .observe(RepairExecutionBoundary::new(node, moment))
        .map_err(AuthorityAffectingRepairExecutionDenial::Interrupted)
}
