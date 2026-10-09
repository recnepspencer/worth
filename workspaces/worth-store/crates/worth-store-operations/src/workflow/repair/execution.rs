use super::integrity_classification::{
    IntegrityOperationalRepairOwner, IntegrityRepairClassificationReceipt,
};
use sha2::{Digest, Sha256};
use worth_store_authority::{StoreCurrentAuthorityIdentity, StoreCurrentAuthorityWitness};

use crate::authorization::{consume_authorization_through, recover_authorization_consumption};
use crate::{
    AuthorizationConsumptionDenial, AuthorizationConsumptionReceipt,
    AuthorizationRevocationObservation, IndeterminateRepairRecoveryHandle, OperationalControlStore,
    OperationalOperationId, OperationalTransitionId, OwnerPlanNodeIdentity,
};

use super::journal::RepairExecutionJournal;
use super::{
    AuthorizedRepairPlan, LoweredRepairOwnerPlanDag, RepairExecutionBoundary,
    RepairExecutionBoundaryMoment, RepairExecutionControlPort, RepairExecutionDisposition,
    RepairExecutionInterrupted, RepairJournalDenial, UninterruptedRepairExecution,
};

#[derive(Debug)]
pub enum RepairReadinessDenial {
    StaleAuthority,
    StoreDerivedIndexRebuildRequired,
    Authorization(AuthorizationConsumptionDenial),
    Journal(RepairJournalDenial),
}

pub struct ExecutionReadyRepair<'a> {
    authority_identity: StoreCurrentAuthorityIdentity,
    authorization: AuthorizationConsumptionReceipt,
    integrity_node: OwnerPlanNodeIdentity,
    integrity: super::integrity_classification::IntegrityRepairClassificationPlan,
    journal: RepairExecutionJournal<'a>,
}

#[derive(Debug)]
pub enum RepairExecutionDenial {
    Journal(RepairJournalDenial),
    RecoveredReceiptMismatch { node: OwnerPlanNodeIdentity },
    Interrupted(RepairExecutionInterrupted),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutedRepairOwnerReceipt {
    Integrity(IntegrityRepairClassificationReceipt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedRepairOwnerReceiptDag {
    receipts: Vec<(OwnerPlanNodeIdentity, ExecutedRepairOwnerReceipt)>,
}

impl ExecutedRepairOwnerReceiptDag {
    pub fn receipts(&self) -> &[(OwnerPlanNodeIdentity, ExecutedRepairOwnerReceipt)] {
        &self.receipts
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedRepair {
    operation_id: OperationalOperationId,
    authority_identity: StoreCurrentAuthorityIdentity,
    authorization: AuthorizationConsumptionReceipt,
    owners: ExecutedRepairOwnerReceiptDag,
}

impl ExecutedRepair {
    pub const fn operation_id(&self) -> &OperationalOperationId {
        &self.operation_id
    }
    pub const fn authority_identity(&self) -> StoreCurrentAuthorityIdentity {
        self.authority_identity
    }
    pub const fn authorization(&self) -> AuthorizationConsumptionReceipt {
        self.authorization
    }
    pub const fn owner_receipts(&self) -> &ExecutedRepairOwnerReceiptDag {
        &self.owners
    }
}

impl AuthorizedRepairPlan {
    pub fn ready<'a>(
        self,
        control: &'a OperationalControlStore,
        authorization_transition: OperationalTransitionId,
        current: &StoreCurrentAuthorityWitness,
        observed_at: u64,
        revocation: AuthorizationRevocationObservation,
    ) -> Result<ExecutionReadyRepair<'a>, RepairReadinessDenial> {
        self.ready_through(
            control,
            control,
            authorization_transition,
            current,
            observed_at,
            revocation,
        )
    }

    pub(super) fn ready_through<'a>(
        self,
        control: &'a OperationalControlStore,
        append: &'a dyn crate::OperationalControlStorePort,
        authorization_transition: OperationalTransitionId,
        current: &StoreCurrentAuthorityWitness,
        observed_at: u64,
        revocation: AuthorizationRevocationObservation,
    ) -> Result<ExecutionReadyRepair<'a>, RepairReadinessDenial> {
        if self.authorization.binding().authority_identity() != current.authority_identity() {
            return Err(RepairReadinessDenial::StaleAuthority);
        }
        if !self.layout.is_empty() {
            return Err(RepairReadinessDenial::StoreDerivedIndexRebuildRequired);
        }
        let operation_id = self.operation_id;
        let consumed = consume_authorization_through(
            control,
            append,
            operation_id.clone(),
            authorization_transition,
            self.authorization,
            None,
            observed_at,
            revocation,
        )
        .map_err(RepairReadinessDenial::Authorization)?;
        let plan_fingerprint = consumed.authorized().binding().fingerprint();
        let owner_nodes = 1;
        let journal = RepairExecutionJournal::open_through(
            control,
            append,
            current.authority_identity(),
            operation_id,
            consumed.receipt().authorization_identity(),
            plan_fingerprint,
            owner_nodes,
            crate::RepairRecoveryTopology::CurrentAuthorityPreserving,
        )
        .map_err(RepairReadinessDenial::Journal)?;
        Ok(ExecutionReadyRepair {
            authority_identity: current.authority_identity(),
            authorization: consumed.receipt(),
            integrity_node: self.integrity_node,
            integrity: self.integrity,
            journal,
        })
    }
}

impl LoweredRepairOwnerPlanDag {
    pub fn recover_ready<'a>(
        self,
        handle: &IndeterminateRepairRecoveryHandle,
        control: &'a OperationalControlStore,
        current: &StoreCurrentAuthorityWitness,
    ) -> Result<ExecutionReadyRepair<'a>, RepairReadinessDenial> {
        self.recover_ready_through(handle, control, control, current)
    }

    pub(super) fn recover_ready_through<'a>(
        self,
        handle: &IndeterminateRepairRecoveryHandle,
        control: &'a OperationalControlStore,
        append: &'a dyn crate::OperationalControlStorePort,
        current: &StoreCurrentAuthorityWitness,
    ) -> Result<ExecutionReadyRepair<'a>, RepairReadinessDenial> {
        let binding = self.authorization.binding();
        if binding.authority_identity() != current.authority_identity()
            || self.operation_id != *handle.operation_id()
            || binding.fingerprint() != handle.plan_fingerprint()
        {
            return Err(RepairReadinessDenial::StaleAuthority);
        }
        if !self.layout.is_empty() {
            return Err(RepairReadinessDenial::StoreDerivedIndexRebuildRequired);
        }
        let recovered_receipts_are_owned = handle.durable_owner_receipts().iter().all(|receipt| {
            receipt.node_fingerprint() == self.integrity_node.fingerprint()
                && receipt.owner_tag() == 2
        });
        let recovered_starts_are_owned = handle.started_owner_nodes().iter().all(|started| {
            started.node_fingerprint() == self.integrity_node.fingerprint()
                && started.owner_tag() == 2
        });
        if !recovered_receipts_are_owned || !recovered_starts_are_owned {
            return Err(RepairReadinessDenial::Journal(
                RepairJournalDenial::InvalidHistory,
            ));
        }
        let authorization = recover_authorization_consumption(
            control,
            handle.operation_id(),
            handle.authorization_identity(),
            handle.plan_fingerprint(),
        )
        .map_err(RepairReadinessDenial::Authorization)?;
        let journal = RepairExecutionJournal::recover_through(
            control,
            append,
            current.authority_identity(),
            handle,
        )
        .map_err(RepairReadinessDenial::Journal)?;
        Ok(ExecutionReadyRepair {
            authority_identity: current.authority_identity(),
            authorization,
            integrity_node: self.integrity_node,
            integrity: self.integrity,
            journal,
        })
    }
}

impl ExecutionReadyRepair<'_> {
    pub fn execute(self) -> Result<ExecutedRepair, RepairExecutionDenial> {
        self.execute_with_control(&UninterruptedRepairExecution)
    }

    pub fn execute_with_control(
        mut self,
        control: &impl RepairExecutionControlPort,
    ) -> Result<ExecutedRepair, RepairExecutionDenial> {
        let mut receipts = Vec::new();
        receipts
            .try_reserve_exact(1)
            .map_err(|_| RepairExecutionDenial::Journal(RepairJournalDenial::InvalidHistory))?;
        self.journal
            .begin_owner_effect(self.integrity_node, 2)
            .map_err(RepairExecutionDenial::Journal)?;
        observe(
            control,
            self.integrity_node,
            RepairExecutionBoundaryMoment::BeforeOwnerEffect,
        )?;
        let integrity = IntegrityOperationalRepairOwner::execute(&self.integrity);
        observe(
            control,
            self.integrity_node,
            RepairExecutionBoundaryMoment::AfterOwnerEffectBeforeReceipt,
        )?;
        record_receipt(
            &mut self.journal,
            self.integrity_node,
            ExecutedRepairOwnerReceipt::Integrity(integrity),
            2,
        )?;
        observe(
            control,
            self.integrity_node,
            RepairExecutionBoundaryMoment::AfterReceiptPersistence,
        )?;
        receipts.push((
            self.integrity_node,
            ExecutedRepairOwnerReceipt::Integrity(integrity),
        ));
        let completion_basis = self
            .journal
            .completion_basis()
            .map_err(RepairExecutionDenial::Journal)?;
        self.journal
            .close(RepairExecutionDisposition::Executed, completion_basis)
            .map_err(RepairExecutionDenial::Journal)?;
        let operation_id = self.journal.operation_id().clone();
        Ok(ExecutedRepair {
            operation_id,
            authority_identity: self.authority_identity,
            authorization: self.authorization,
            owners: ExecutedRepairOwnerReceiptDag { receipts },
        })
    }
}

fn observe(
    control: &impl RepairExecutionControlPort,
    node: OwnerPlanNodeIdentity,
    moment: RepairExecutionBoundaryMoment,
) -> Result<(), RepairExecutionDenial> {
    control
        .observe(RepairExecutionBoundary::new(node, moment))
        .map_err(RepairExecutionDenial::Interrupted)
}

fn record_receipt(
    journal: &mut RepairExecutionJournal<'_>,
    node: OwnerPlanNodeIdentity,
    receipt: ExecutedRepairOwnerReceipt,
    owner_tag: u8,
) -> Result<(), RepairExecutionDenial> {
    let fingerprint = receipt_fingerprint(receipt);
    if let Some(reopened) = journal.completed(node) {
        if reopened != fingerprint {
            return Err(RepairExecutionDenial::RecoveredReceiptMismatch { node });
        }
        return Ok(());
    }
    journal
        .persist_owner_receipt(node, fingerprint, owner_tag)
        .map_err(RepairExecutionDenial::Journal)
}

fn receipt_fingerprint(receipt: ExecutedRepairOwnerReceipt) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-store-executed-repair-owner-receipt-v1");
    match receipt {
        ExecutedRepairOwnerReceipt::Integrity(value) => {
            digest.update([2]);
            digest.update(value.plan_fingerprint());
            digest.update(value.classified_regions().to_be_bytes());
            digest.update(value.quarantined_regions().to_be_bytes());
        }
    }
    digest.finalize().into()
}
