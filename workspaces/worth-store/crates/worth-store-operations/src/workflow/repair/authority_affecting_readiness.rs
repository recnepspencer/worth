use super::integrity_classification::IntegrityRepairClassificationPlan;
use crate::workflow::restore::BackupRestoreReplayPlan;
use worth_store_physical_backend::LoweredNonCurrentStagingPlan;

use crate::{
    AuthorizationConsumptionDenial, AuthorizationConsumptionReceipt, OperationalOperationId,
    OperationalSecurityScope,
};

use super::{
    authority_owner_dag::RepairOwnerNodes, journal::RepairExecutionJournal, RepairJournalDenial,
};

#[derive(Debug)]
pub enum AuthorityAffectingRepairReadinessDenial {
    StaleAuthority,
    Target(crate::control_store::NonCurrentRecoveryTargetDenial),
    Authorization(AuthorizationConsumptionDenial),
    Journal(RepairJournalDenial),
}

pub struct ExecutionReadyAuthorityAffectingRepair<'a> {
    pub(super) operation_id: OperationalOperationId,
    pub(super) authorization: AuthorizationConsumptionReceipt,
    pub(super) staging_authority: worth_store_authority::StoreCurrentAuthorityIdentity,
    pub(super) security_scope: OperationalSecurityScope,
    pub(super) integrity: IntegrityRepairClassificationPlan,
    pub(super) backend: LoweredNonCurrentStagingPlan,
    pub(super) recovery: BackupRestoreReplayPlan,
    pub(super) layout: Option<worth_store_layout_indexes::LayoutRepairConsequencePlan>,
    pub(super) blob: Option<worth_store_blob_chunks::BlobRepairConsequencePlan>,
    pub(super) nodes: RepairOwnerNodes,
    pub(super) journal: RepairExecutionJournal<'a>,
    pub(super) _target_admission: crate::control_store::NonCurrentRecoveryTargetAdmission,
}
