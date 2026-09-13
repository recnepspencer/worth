use super::authorization_control_replay::observe_authorization_consumption;
use super::recovery_staging_control_replay::{
    observe_authorized_staging, observe_staging_completed, ReplayedRecoveryStaging,
};
use super::repair_control_replay::ReplayedRepairJournal;
use super::replica_operation_control_replay::{
    observe_authorization as observe_replica_authorization, ReplayedReplicaBootstrap,
    ReplayedReplicaPromotion,
};
use super::selected_control_replay_state::{ReplayedBackup, ReplayedWorkflow};
use super::{
    archived_workflow_index::{ArchivedWorkflowIndex, ArchivedWorkflowKind},
    selected_control_replay_contract::{
        invalid, state_denial, wrong_workflow, OperationalControlHistoryViolationKind,
        SelectedControlReplayDenial,
    },
    OperationalControlRecord, OperationalControlRecordKind, OperationalControlReplayBudget,
    OperationalControlReplayResource, OperationalOperationId,
};
use std::collections::HashMap;
use worth_store_physical_backend::ControlMediaFault;

pub(crate) struct SelectedControlReplay {
    pub(super) workflows: HashMap<OperationalOperationId, ReplayedWorkflow>,
    pub(super) archived: ArchivedWorkflowIndex,
    pub(super) completed_backups: u64,
    pub(super) abandoned_backups: u64,
    pub(super) budget: OperationalControlReplayBudget,
    pub(super) active_recovery_object_bytes: u64,
    pub(super) consumed_authorizations: HashMap<[u8; 32], ([u8; 32], OperationalOperationId)>,
    pub(super) repair_journals: HashMap<OperationalOperationId, ReplayedRepairJournal>,
    pub(super) recovery_staging: HashMap<OperationalOperationId, ReplayedRecoveryStaging>,
    pub(super) replica_bootstraps: HashMap<OperationalOperationId, ReplayedReplicaBootstrap>,
    pub(super) replica_promotions: HashMap<OperationalOperationId, ReplayedReplicaPromotion>,
}

impl SelectedControlReplay {
    pub(crate) fn new(budget: OperationalControlReplayBudget) -> Self {
        Self {
            workflows: HashMap::new(),
            archived: ArchivedWorkflowIndex::empty(),
            completed_backups: 0,
            abandoned_backups: 0,
            budget,
            active_recovery_object_bytes: 0,
            consumed_authorizations: HashMap::new(),
            repair_journals: HashMap::new(),
            recovery_staging: HashMap::new(),
            replica_bootstraps: HashMap::new(),
            replica_promotions: HashMap::new(),
        }
    }

    pub(crate) fn observe(
        &mut self,
        record_index: u64,
        record: OperationalControlRecord,
    ) -> Result<(), SelectedControlReplayDenial> {
        let authority_identity = record.authority_identity();
        let (operation, kind) = record.into_replay_parts();
        if self.observe_replica_transition(record_index, &operation, &kind)? {
            return Ok(());
        }
        match kind {
            OperationalControlRecordKind::WorkflowOpened { workflow } => {
                self.observe_workflow_open(record_index, operation, workflow)?;
            }
            OperationalControlRecordKind::SourceLeasePersisted {
                recovery,
                recovery_object,
            } => {
                if recovery.authority_identity() != authority_identity {
                    return invalid(
                        record_index,
                        operation,
                        OperationalControlHistoryViolationKind::SourceLeaseAuthorityMismatch,
                    );
                }
                let required = self
                    .active_recovery_object_bytes
                    .checked_add(recovery_object.bytes())
                    .ok_or(SelectedControlReplayDenial::CounterOverflow)?;
                let active_recovery_limit = self.budget.max_active_recovery_object_bytes();
                if required > active_recovery_limit {
                    return Err(SelectedControlReplayDenial::BudgetExceeded {
                        resource: OperationalControlReplayResource::ActiveRecoveryObjectBytes,
                        required,
                        limit: active_recovery_limit,
                    });
                }
                let active = ReplayedWorkflow::BackupActive(ReplayedBackup {
                    recovery,
                    materialization_plan: None,
                    materialized: false,
                    recovery_object_bytes: recovery_object.bytes(),
                });
                if self.workflows.contains_key(&operation) {
                    let state = self.workflows.get_mut(&operation).ok_or(
                        SelectedControlReplayDenial::DerivedIndex(
                            ControlMediaFault::DerivedTransitionIndexCorrupt,
                        ),
                    )?;
                    match state {
                        ReplayedWorkflow::BackupAwaitingSourceLease { .. } => *state = active,
                        ReplayedWorkflow::BackupActive(_) => {
                            return invalid(
                                record_index,
                                operation,
                                OperationalControlHistoryViolationKind::DuplicateSourceLease,
                            );
                        }
                    }
                } else {
                    match self
                        .archived
                        .lookup(&operation)
                        .map_err(SelectedControlReplayDenial::DerivedIndex)?
                    {
                        Some(ArchivedWorkflowKind::BackupTerminal) => {
                            return invalid(
                                record_index,
                                operation,
                                OperationalControlHistoryViolationKind::RecordAfterTerminal,
                            );
                        }
                        Some(ArchivedWorkflowKind::NonBackup(workflow)) => {
                            return invalid(record_index, operation, wrong_workflow(workflow));
                        }
                        None => {}
                    }
                    let active_workflows = self.workflows.len().saturating_add(1);
                    if active_workflows > self.budget.max_active_workflows() {
                        return Err(SelectedControlReplayDenial::BudgetExceeded {
                            resource: OperationalControlReplayResource::ActiveWorkflows,
                            required: u64::try_from(active_workflows).unwrap_or(u64::MAX),
                            limit: u64::try_from(self.budget.max_active_workflows())
                                .unwrap_or(u64::MAX),
                        });
                    }
                    self.workflows
                        .try_reserve(1)
                        .map_err(|_| SelectedControlReplayDenial::AllocationFailed)?;
                    self.workflows.insert(operation, active);
                }
                self.active_recovery_object_bytes = required;
            }
            OperationalControlRecordKind::BackupMaterializationOpened { plan } => {
                let active = match self.active_backup(
                    &operation,
                    OperationalControlHistoryViolationKind::MaterializationBeforeSourceLease,
                ) {
                    Ok(active) => active,
                    Err(denial) => return state_denial(record_index, operation, denial),
                };
                if active.recovery.cut_identity() != plan.cut_identity() {
                    return invalid(
                        record_index,
                        operation,
                        OperationalControlHistoryViolationKind::MaterializationCutMismatch,
                    );
                }
                if active.materialization_plan.is_some() {
                    return invalid(
                        record_index,
                        operation,
                        OperationalControlHistoryViolationKind::DuplicateMaterializationPlan,
                    );
                }
                active.materialization_plan = Some(plan);
            }
            OperationalControlRecordKind::BackupMaterializationRecorded { .. } => {
                let active = match self.active_backup(
                    &operation,
                    OperationalControlHistoryViolationKind::MaterializationReceiptBeforePlan,
                ) {
                    Ok(active) => active,
                    Err(denial) => return state_denial(record_index, operation, denial),
                };
                if active.materialization_plan.is_none() {
                    return invalid(
                        record_index,
                        operation,
                        OperationalControlHistoryViolationKind::MaterializationReceiptBeforePlan,
                    );
                }
                if active.materialized {
                    return invalid(
                        record_index,
                        operation,
                        OperationalControlHistoryViolationKind::DuplicateMaterializationReceipt,
                    );
                }
                active.materialized = true;
            }
            OperationalControlRecordKind::IndependentBackupVerificationRecordedAndSourceLeaseReleased {
                release,
                ..
            } => {
                if let Err(denial) = self.finish_backup(
                    &operation,
                    OperationalControlHistoryViolationKind::VerificationBeforeMaterialization,
                    release.cut_identity(),
                    true,
                ) {
                    return state_denial(record_index, operation, denial);
                }
                self.completed_backups = self
                    .completed_backups
                    .checked_add(1)
                    .ok_or(SelectedControlReplayDenial::CounterOverflow)?;
            }
            OperationalControlRecordKind::BackupAbandoned {
                released_source_lease,
                ..
            } => {
                if let Err(denial) = self.finish_backup(
                    &operation,
                    OperationalControlHistoryViolationKind::TerminalBeforeSourceLease,
                    released_source_lease.cut_identity(),
                    false,
                ) {
                    return state_denial(record_index, operation, denial);
                }
                self.abandoned_backups = self
                    .abandoned_backups
                    .checked_add(1)
                    .ok_or(SelectedControlReplayDenial::CounterOverflow)?;
            }
            OperationalControlRecordKind::AuthorizationConsumed {
                authorization_identity,
                plan_fingerprint,
                operation_tag,
                execution_plan_fingerprint,
                ..
            } => {
                observe_authorization_consumption(
                    &mut self.consumed_authorizations,
                    &operation,
                    authorization_identity,
                    plan_fingerprint,
                )
                .map_err(|kind| SelectedControlReplayDenial::Invalid(
                    super::OperationalControlHistoryViolation::new(
                        record_index, operation.clone(), kind)))?;
                observe_replica_authorization(
                    &mut self.replica_bootstraps,
                    &mut self.replica_promotions,
                    &operation,
                    authority_identity,
                    operation_tag,
                    authorization_identity,
                    plan_fingerprint,
                    execution_plan_fingerprint,
                )
                .map_err(|kind| SelectedControlReplayDenial::Invalid(
                    super::OperationalControlHistoryViolation::new(
                        record_index, operation.clone(), kind)))?;
                observe_authorized_staging(
                    &mut self.recovery_staging,
                    &operation,
                    authority_identity,
                    operation_tag,
                    authorization_identity,
                    plan_fingerprint,
                    execution_plan_fingerprint,
                )
                .map_err(|kind| SelectedControlReplayDenial::Invalid(
                    super::OperationalControlHistoryViolation::new(record_index, operation, kind)))?;
            }
            OperationalControlRecordKind::RepairExecutionOpened {
                authorization_identity,
                plan_fingerprint,
                owner_node_count,
                topology_tag,
            } => self.observe_repair_open(
                record_index,
                &operation,
                authority_identity,
                authorization_identity,
                plan_fingerprint,
                owner_node_count,
                topology_tag,
            )?,
            OperationalControlRecordKind::RepairOwnerReceiptPersisted {
                plan_fingerprint,
                node_fingerprint,
                receipt_fingerprint,
                owner_tag,
            } => self.observe_repair_receipt(
                record_index,
                &operation,
                plan_fingerprint,
                node_fingerprint,
                receipt_fingerprint,
                owner_tag,
            )?,
            OperationalControlRecordKind::RepairOwnerEffectStarted {
                plan_fingerprint,
                node_fingerprint,
                owner_tag,
            } => self.observe_repair_start(
                record_index,
                &operation,
                plan_fingerprint,
                node_fingerprint,
                owner_tag,
            )?,
            OperationalControlRecordKind::OperationalOwnerReceiptPersisted {
                workflow,
                plan_fingerprint,
                receipt_fingerprint,
                owner_kind,
            } => self.observe_recovery_owner_receipt(
                record_index,
                &operation,
                workflow,
                plan_fingerprint,
                receipt_fingerprint,
                owner_kind,
            )?,
            OperationalControlRecordKind::ReplicaBootstrapTransferRecorded { .. }
            | OperationalControlRecordKind::ReplicaBootstrapCompleted { .. }
            | OperationalControlRecordKind::ReplicaBootstrapAbandoned { .. }
            | OperationalControlRecordKind::ReplicaPromotionFenceRecorded { .. }
            | OperationalControlRecordKind::ReplicaPromotionRecorded { .. }
            | OperationalControlRecordKind::ReplicaPromotionPublished { .. }
            | OperationalControlRecordKind::ReplicaPromotionReadmitted { .. }
            | OperationalControlRecordKind::OldPrimaryRejoinPlanned { .. }
            | OperationalControlRecordKind::OldPrimaryRejoinCompleted { .. } => unreachable!(
                "replica transitions are consumed before the general replay dispatch"
            ),
            OperationalControlRecordKind::RepairDispositionRecorded {
                plan_fingerprint,
                disposition_tag,
                disposition_basis,
            } => self.observe_repair_disposition(
                record_index,
                &operation,
                plan_fingerprint,
                disposition_tag,
                disposition_basis,
            )?,
            OperationalControlRecordKind::RecoveryStagingCompleted {
                authorization_identity, plan_fingerprint, execution_plan_fingerprint,
                staged_media_identity,
            } => observe_staging_completed(
                &mut self.recovery_staging,
                &operation,
                authorization_identity,
                plan_fingerprint,
                execution_plan_fingerprint,
                staged_media_identity,
            )
            .map_err(|kind| SelectedControlReplayDenial::Invalid(
                super::OperationalControlHistoryViolation::new(record_index, operation, kind)))?,
        }
        Ok(())
    }
}
