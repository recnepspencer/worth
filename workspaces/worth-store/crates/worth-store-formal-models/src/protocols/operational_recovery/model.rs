use std::collections::{BTreeMap, BTreeSet};

use super::{
    OperationalRecoveryAction, OperationalRecoveryActionKind, OperationalRecoveryCounterexample,
    OperationalRecoveryInvariant,
};

#[derive(Debug, Default)]
struct OperationState {
    transitions: BTreeSet<String>,
    workflow_opened: bool,
    source_lease_persisted: bool,
    materialization_opened: bool,
    materialization_recorded: bool,
    authorized: bool,
    owner_execution_opened: bool,
    owner_effect_started: bool,
    workflow_owner_receipts: BTreeSet<u8>,
    publication_prepared: bool,
    publication_pending: bool,
    publication_disposition: bool,
    promotion_fenced: bool,
    bootstrap_transferred: bool,
    promotion_recorded: bool,
    promotion_published: bool,
    promotion_readmitted: bool,
    rejoin_planned: bool,
    terminal: bool,
}

#[derive(Debug, Default)]
pub struct OperationalRecoveryModel {
    operations: BTreeMap<([u8; 32], String), OperationState>,
    semantic_operations:
        BTreeMap<([u8; 32], String), super::semantic_state::OperationalRecoverySemanticState>,
    reached: BTreeSet<OperationalRecoveryActionKind>,
}

impl OperationalRecoveryModel {
    pub fn apply(
        &mut self,
        action: &OperationalRecoveryAction,
    ) -> Result<(), OperationalRecoveryCounterexample> {
        let state = self
            .operations
            .entry((
                action.authority_identity(),
                action.operation_identity().to_owned(),
            ))
            .or_default();
        if !state
            .transitions
            .insert(action.transition_identity().to_owned())
        {
            return Err(counterexample(
                action,
                OperationalRecoveryInvariant::UniqueTransitionIdentity,
            ));
        }
        use OperationalRecoveryActionKind as Action;
        if state.terminal {
            return Err(counterexample(
                action,
                OperationalRecoveryInvariant::TerminalOperationHasNoLaterTransition,
            ));
        }
        match action.kind() {
            Action::WorkflowOpened => {
                if state.workflow_opened {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::SingleWorkflowOpen,
                    ));
                }
                state.workflow_opened = true;
            }
            Action::SourceLeasePersisted => {
                // Ordinary backup admission opens durably with its source
                // lease record. Older selected histories and current owner
                // paths do not emit a separate WorkflowOpened record.
                state.workflow_opened = true;
                state.source_lease_persisted = true;
            }
            Action::MaterializationOpened => {
                if !state.source_lease_persisted {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::SourceLeaseBeforeMaterialization,
                    ));
                }
                state.materialization_opened = true;
            }
            Action::MaterializationRecorded => {
                if !state.materialization_opened {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::MaterializationOpenBeforeReceipt,
                    ));
                }
                state.materialization_recorded = true;
            }
            Action::IndependentVerificationRecorded => {
                if !state.materialization_recorded {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::MaterializationBeforeVerification,
                    ));
                }
                state.terminal = true;
            }
            Action::Abandoned => state.terminal = true,
            Action::AuthorizationConsumed => {
                // Destructive workflows open durably at authorization
                // consumption; the control store does not persist a
                // free-standing open record for these owners.
                state.workflow_opened = true;
                state.authorized = true;
            }
            Action::OwnerExecutionOpened => {
                require_authorized(action, state)?;
                state.owner_execution_opened = true;
            }
            Action::OwnerEffectStarted => {
                require_authorized(action, state)?;
                if !state.owner_execution_opened {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::OwnerExecutionBeforeEffect,
                    ));
                }
                state.owner_effect_started = true;
            }
            Action::OwnerReceiptPersisted => {
                if !state.owner_execution_opened {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::OwnerExecutionBeforeReceipt,
                    ));
                }
                if !state.owner_effect_started {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::OwnerEffectBeforeReceipt,
                    ));
                }
            }
            Action::WorkflowOwnerReceiptPersisted => {
                require_authorized(action, state)?;
                if !state.workflow_opened {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::WorkflowBeforeOwnerReceipt,
                    ));
                }
                let owner_tag = action.owner_tag().ok_or_else(|| {
                    counterexample(
                        action,
                        OperationalRecoveryInvariant::CompleteOwnerReceiptsBeforeStaging,
                    )
                })?;
                if !matches!(owner_tag, 1 | 2) || !state.workflow_owner_receipts.insert(owner_tag) {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::CompleteOwnerReceiptsBeforeStaging,
                    ));
                }
            }
            Action::StagingCompleted => {
                require_authorized(action, state)?;
                if state.workflow_owner_receipts != BTreeSet::from([1, 2]) {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::CompleteOwnerReceiptsBeforeStaging,
                    ));
                }
            }
            Action::ReplicaBootstrapTransferRecorded => {
                require_authorized(action, state)?;
                state.bootstrap_transferred = true;
            }
            Action::ReplicaBootstrapCompleted => {
                if !state.bootstrap_transferred {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::BootstrapTransferBeforeCompletion,
                    ));
                }
            }
            Action::PublicationPrepared => state.publication_prepared = true,
            Action::PublicationPending => {
                if !state.publication_prepared {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::PreparationBeforePublication,
                    ));
                }
                state.publication_pending = true;
            }
            Action::PublicationDisposition => {
                if !state.publication_pending {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::PendingBeforeDisposition,
                    ));
                }
                state.publication_disposition = true;
            }
            Action::FenceReleased => {
                if !state.publication_disposition {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::DispositionBeforeFenceRelease,
                    ));
                }
            }
            Action::ReplicaPromotionFenceRecorded => {
                require_authorized(action, state)?;
                state.promotion_fenced = true;
            }
            Action::ReplicaPromotionRecorded => {
                require_authorized(action, state)?;
                if !state.promotion_fenced {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::ExternalFenceBeforePromotion,
                    ));
                }
                state.promotion_recorded = true;
            }
            Action::ReplicaPromotionPublished => {
                if !state.promotion_recorded {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::PromotionBeforePublication,
                    ));
                }
                state.promotion_published = true;
            }
            Action::ReplicaPromotionReadmitted => {
                if !state.promotion_published {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::PromotionPublicationBeforeReadmission,
                    ));
                }
                state.promotion_readmitted = true;
            }
            Action::OldPrimaryRejoinPlanned => {
                if !state.promotion_readmitted {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::PromotionReadmissionBeforeRejoin,
                    ));
                }
                state.rejoin_planned = true;
            }
            Action::OldPrimaryRejoinCompleted => {
                if !state.rejoin_planned {
                    return Err(counterexample(
                        action,
                        OperationalRecoveryInvariant::RejoinPlanBeforeCompletion,
                    ));
                }
            }
            Action::DispositionRecorded => {}
        }
        let semantic_state = self
            .semantic_operations
            .entry((
                action.authority_identity(),
                action.operation_identity().to_owned(),
            ))
            .or_default();
        if let Err(invariant) = semantic_state.apply(action) {
            return Err(counterexample(action, invariant));
        }
        self.reached.insert(action.kind());
        Ok(())
    }

    pub fn reached_transitions(&self) -> &BTreeSet<OperationalRecoveryActionKind> {
        &self.reached
    }
}

fn require_authorized(
    action: &OperationalRecoveryAction,
    state: &OperationState,
) -> Result<(), OperationalRecoveryCounterexample> {
    if !state.authorized {
        return Err(counterexample(
            action,
            OperationalRecoveryInvariant::AuthorizationBeforeExecution,
        ));
    }
    Ok(())
}

fn counterexample(
    action: &OperationalRecoveryAction,
    invariant: OperationalRecoveryInvariant,
) -> OperationalRecoveryCounterexample {
    OperationalRecoveryCounterexample {
        operation_identity: action.operation_identity().to_owned(),
        transition_identity: action.transition_identity().to_owned(),
        invariant,
    }
}
