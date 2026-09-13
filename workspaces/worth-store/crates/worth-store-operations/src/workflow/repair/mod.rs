mod authority_affecting;
mod authority_affecting_execution;
mod authority_affecting_readiness;
mod authority_owner_dag;
mod authority_receipt_persistence;
mod authority_staging_artifacts;
#[cfg(any(test, feature = "certification-test-authority"))]
mod certification_control_store;
mod execution;
mod execution_control;
mod integrity_classification;
mod intent;
mod journal;
mod journal_replay;
mod lowering;
mod region_projection;
mod resolved_region;

#[cfg(test)]
mod authority_affecting_tests;
#[cfg(test)]
mod crash_matrix_tests;
#[cfg(test)]
mod crash_recovery_tests;
#[cfg(test)]
mod derived_maintenance_tests;

pub use execution::{
    ExecutedRepair, ExecutedRepairOwnerReceipt, ExecutedRepairOwnerReceiptDag,
    ExecutionReadyRepair, RepairExecutionDenial, RepairReadinessDenial,
};
pub use execution_control::{
    RepairExecutionBoundary, RepairExecutionBoundaryMoment, RepairExecutionControlPort,
    RepairExecutionInterrupted, RepairExecutionInterruptionCause, UninterruptedRepairExecution,
};
pub use integrity_classification::{
    IntegrityRepairClassificationDenial, IntegrityRepairClassificationReceipt,
};
#[cfg(any(test, feature = "certification-test-authority"))]
pub(crate) use intent::{
    certification_authority_repair_candidates_from_backup_observation,
    certification_authority_repair_from_backup_observation,
    certification_derived_maintenance_from_fixture_observation,
};
pub use intent::{
    CurrentAuthorityPreservingMaintenancePlan, EvidenceBoundRepairPlan, RepairCandidateSet,
    RepairIntent, RepairPlanExplanation, RepairResolutionDenial, UnrecoverableDamageReport,
};
pub use journal::{RepairExecutionDisposition, RepairJournalDenial};
pub use lowering::{AuthorizedRepairPlan, LoweredRepairOwnerPlanDag, RepairLoweringDenial};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DerivedRepairOperation;

#[derive(Debug, Clone, Copy)]
pub(crate) struct AuthorityAffectingRepairOperation;
pub use authority_affecting::{
    AuthorityAffectingRepairLoweringDenial, AuthorityAffectingStagedRepairPlan,
    AuthorizedAuthorityAffectingRepairPlan, LoweredAuthorityAffectingRepairOwnerPlanDag,
};
pub use authority_affecting_execution::{
    AuthorityAffectingRepairExecutionDenial, ExecutedAuthorityAffectingRepair,
};
pub use authority_affecting_readiness::{
    AuthorityAffectingRepairReadinessDenial, ExecutionReadyAuthorityAffectingRepair,
};
