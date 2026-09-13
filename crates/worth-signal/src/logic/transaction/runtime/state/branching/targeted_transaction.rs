use std::convert::Infallible;

use serde::{Deserialize, Serialize};
use worth_proof::TransitionOutcome;

use crate::clock::RuntimeInstant;
use crate::data::error::SignalError;
use crate::logic::transaction::runtime::transaction::{
    SignalTransaction, TransactionCommitPosture, TransactionExecutionState, TransactionResult,
    TransactionRollbackPacketSet, TransactionScratch,
};
use crate::state::{SignalBranchHandle, SignalBranchId};

use super::super::runtime_state::{
    AuthorityTransferPacket, BranchLifecycleTransfer, SignalRuntime,
};
use super::transaction_head::SignalBranchTransactionHead;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchTargetedTransactionRequest {
    target_branch: SignalBranchHandle,
    expected_head: SignalBranchTransactionHead,
}

impl BranchTargetedTransactionRequest {
    pub fn new(
        target_branch: SignalBranchHandle,
        expected_head: SignalBranchTransactionHead,
    ) -> Self {
        Self {
            target_branch,
            expected_head,
        }
    }

    pub fn target_branch(&self) -> &SignalBranchHandle {
        &self.target_branch
    }

    pub(crate) fn expected_head(&self) -> &SignalBranchTransactionHead {
        &self.expected_head
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchTargetedTransactionDenial {
    UnknownTargetBranch {
        branch_id: SignalBranchId,
    },
    ActiveBranchTarget {
        branch_id: SignalBranchId,
    },
    CrossBranchHead {
        target_branch_id: SignalBranchId,
        head_branch_id: SignalBranchId,
    },
    StaleTargetHead {
        expected: SignalBranchTransactionHead,
        observed: SignalBranchTransactionHead,
    },
    CanonicalBasisMismatch,
}

#[derive(Debug, Clone)]
pub struct ValidatedBranchTargetedTransactionRequest {
    request: BranchTargetedTransactionRequest,
    observed_head: SignalBranchTransactionHead,
}

impl ValidatedBranchTargetedTransactionRequest {
    pub(crate) fn request(&self) -> &BranchTargetedTransactionRequest {
        &self.request
    }

    pub(crate) fn observed_head(&self) -> &SignalBranchTransactionHead {
        &self.observed_head
    }
}

#[derive(Debug, Clone)]
pub struct LoweredBranchTargetedTransactionPlan {
    validated: ValidatedBranchTargetedTransactionRequest,
    active_branch_at_plan: SignalBranchHandle,
}

impl LoweredBranchTargetedTransactionPlan {
    pub(crate) fn validated(&self) -> &ValidatedBranchTargetedTransactionRequest {
        &self.validated
    }

    pub fn active_branch_at_plan(&self) -> &SignalBranchHandle {
        &self.active_branch_at_plan
    }
}

#[derive(Debug)]
pub struct ExecutedBranchTargetedTransactionReceipt {
    plan: LoweredBranchTargetedTransactionPlan,
    before_head: SignalBranchTransactionHead,
    after_head: SignalBranchTransactionHead,
    active_branch_before: SignalBranchHandle,
    active_branch_after: SignalBranchHandle,
    transaction: TransactionResult,
}

impl ExecutedBranchTargetedTransactionReceipt {
    pub(crate) fn plan(&self) -> &LoweredBranchTargetedTransactionPlan {
        &self.plan
    }

    pub(crate) fn before_head(&self) -> &SignalBranchTransactionHead {
        &self.before_head
    }

    pub(crate) fn after_head(&self) -> &SignalBranchTransactionHead {
        &self.after_head
    }

    pub fn active_branch_before(&self) -> &SignalBranchHandle {
        &self.active_branch_before
    }

    pub fn active_branch_after(&self) -> &SignalBranchHandle {
        &self.active_branch_after
    }

    pub fn transaction(&self) -> &TransactionResult {
        &self.transaction
    }
}

pub type BranchTargetedTransactionExecutionOutcome = TransitionOutcome<
    ExecutedBranchTargetedTransactionReceipt,
    BranchTargetedTransactionDenial,
    Infallible,
    Infallible,
    Infallible,
    SignalError,
>;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn branch_transaction_head(
        &self,
        branch: SignalBranchHandle,
    ) -> TransitionOutcome<SignalBranchTransactionHead, BranchTargetedTransactionDenial> {
        match self.observe_branch_transaction_head(&branch) {
            Ok(head) => TransitionOutcome::success(head),
            Err(denial) => TransitionOutcome::denied(denial),
        }
    }

    pub(crate) fn plan_branch_targeted_transaction(
        &mut self,
        request: BranchTargetedTransactionRequest,
    ) -> TransitionOutcome<LoweredBranchTargetedTransactionPlan, BranchTargetedTransactionDenial>
    {
        self.with_telemetry(|telemetry| {
            telemetry.transaction.branch_targeted_transaction_plan_count += 1;
        });
        let observed_head = match self.validate_targeted_request(&request) {
            Ok(head) => head,
            Err(denial) => {
                self.record_targeted_denial(&denial);
                return TransitionOutcome::denied(denial);
            }
        };
        TransitionOutcome::success(LoweredBranchTargetedTransactionPlan {
            validated: ValidatedBranchTargetedTransactionRequest {
                request,
                observed_head,
            },
            active_branch_at_plan: self.graph.current_branch(),
        })
    }

    pub(crate) fn execute_branch_targeted_transaction<F>(
        &mut self,
        runtime_ctx: &mut Ctx,
        plan: LoweredBranchTargetedTransactionPlan,
        apply: F,
    ) -> BranchTargetedTransactionExecutionOutcome
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        let request = plan.validated.request();
        let before_head = match self.validate_targeted_request(request) {
            Ok(head) => head,
            Err(denial) => {
                self.record_targeted_denial(&denial);
                return TransitionOutcome::denied(denial);
            }
        };
        let active_branch_before = self.graph.current_branch();
        let target_branch_id = request.target_branch.id;
        let target_state = self
            .branches
            .branch_state(target_branch_id)
            .expect("validated targeted transaction must retain target branch state");
        if let Err(error) = self.ensure_branch_state_managed_queue_transfer_allowed(target_state) {
            return TransitionOutcome::failed(error);
        }
        let target_packet = self
            .branches
            .take_stored_branch_transfer(target_branch_id)
            .expect("validated targeted transaction must own stored target state");
        let active_state = self
            .take_heavy_active_branch_state()
            .expect("managed-queue preflight must make active-state transfer infallible");
        self.apply_branch_lifecycle_transfer(BranchLifecycleTransfer::Move(target_packet))
            .expect("validated targeted transaction transfer must preserve branch identity");

        let transaction_result = self.execute_branch_local_transaction(runtime_ctx, apply);
        let targeted_transaction_telemetry = self.telemetry_snapshot().transaction;
        let target_state = self
            .take_heavy_active_branch_state()
            .expect("branch-local transaction cannot mint managed queue authority");
        self.branches.store_branch_state(target_state);
        self.apply_branch_lifecycle_transfer(BranchLifecycleTransfer::Move(
            AuthorityTransferPacket::new(active_branch_before.id, active_state),
        ))
        .expect("targeted transaction must restore the caller's active branch");
        self.with_telemetry(|telemetry| {
            Self::merge_global_transaction_telemetry(
                targeted_transaction_telemetry,
                &mut telemetry.transaction,
            );
        });
        let active_branch_after = self.graph.current_branch();

        let transaction = match transaction_result {
            Ok(result) => result,
            Err(error) => return TransitionOutcome::failed(error),
        };
        let after_head = self
            .observe_branch_transaction_head(&plan.validated.request.target_branch)
            .expect("executed target branch must remain live after restoration");
        let touched_nodes = u64::from(transaction.touched_nodes);
        self.with_telemetry(|telemetry| {
            telemetry
                .transaction
                .branch_targeted_transaction_execution_count += 1;
            telemetry
                .transaction
                .branch_targeted_transaction_active_switch_avoided_count += 1;
            telemetry
                .transaction
                .branch_targeted_transaction_touched_node_count += touched_nodes;
        });
        TransitionOutcome::success(ExecutedBranchTargetedTransactionReceipt {
            plan,
            before_head,
            after_head,
            active_branch_before,
            active_branch_after,
            transaction,
        })
    }

    fn execute_branch_local_transaction<F>(
        &mut self,
        runtime_ctx: &mut Ctx,
        apply: F,
    ) -> Result<TransactionResult, SignalError>
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        let captures_telemetry = self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        );
        self.with_telemetry(|telemetry| telemetry.transaction.transaction_begin_count += 1);
        self.config.sync_graph_capacity(&self.graph);
        let current_branch = self.graph.current_branch();
        let (branch_mutation_ledger, branch_head_generation, branch_restore_snapshot_id) = self
            .branches
            .transaction_branch_state_mut(current_branch.id, current_branch.head_snapshot_id);
        let mut transaction = SignalTransaction {
            runtime_ctx,
            observations: &self.observations,
            config: &mut self.config,
            graph: &mut self.graph,
            checkpoint: &mut self.checkpoint,
            event_bus: &mut self.event_bus,
            resource: &mut self.resource,
            temporal: &mut self.temporal,
            telemetry: captures_telemetry.then_some(&mut self.telemetry),
            branch_mutation_ledger,
            branch_head_generation,
            branch_restore_snapshot_id,
            conditional_operation_scope: None,
            scratch: TransactionScratch::new(),
            rollback_packets: TransactionRollbackPacketSet::default(),
            poisoned: false,
            finished: false,
            execution_state: TransactionExecutionState::default(),
            started_at: RuntimeInstant::now(),
            commit_posture: TransactionCommitPosture::BranchLocal,
        };
        match apply(&mut transaction) {
            Ok(()) => transaction.commit(),
            Err(error) => match transaction.rollback() {
                Ok(_) => Err(error),
                Err(rollback_error) => Err(rollback_error),
            },
        }
    }

    fn validate_targeted_request(
        &self,
        request: &BranchTargetedTransactionRequest,
    ) -> Result<SignalBranchTransactionHead, BranchTargetedTransactionDenial> {
        if request.target_branch.id == self.graph.current_branch().id {
            return Err(BranchTargetedTransactionDenial::ActiveBranchTarget {
                branch_id: request.target_branch.id,
            });
        }
        if request.expected_head.branch_id() != request.target_branch.id {
            return Err(BranchTargetedTransactionDenial::CrossBranchHead {
                target_branch_id: request.target_branch.id,
                head_branch_id: request.expected_head.branch_id(),
            });
        }
        let observed = self.observe_branch_transaction_head(&request.target_branch)?;
        if observed != request.expected_head {
            return Err(BranchTargetedTransactionDenial::StaleTargetHead {
                expected: request.expected_head.clone(),
                observed,
            });
        }
        Ok(observed)
    }

    pub(super) fn observe_branch_transaction_head(
        &self,
        branch: &SignalBranchHandle,
    ) -> Result<SignalBranchTransactionHead, BranchTargetedTransactionDenial> {
        let live_branch = self.branches.branch_handle(branch.id).ok_or(
            BranchTargetedTransactionDenial::UnknownTargetBranch {
                branch_id: branch.id,
            },
        )?;
        Ok(SignalBranchTransactionHead::new(
            live_branch.id,
            live_branch.head_snapshot_id,
            self.branches.branch_head_generation(live_branch.id),
        ))
    }

    fn record_targeted_denial(&mut self, denial: &BranchTargetedTransactionDenial) {
        self.with_telemetry(|telemetry| {
            telemetry
                .transaction
                .branch_targeted_transaction_denial_count += 1;
        });
        if matches!(
            denial,
            BranchTargetedTransactionDenial::StaleTargetHead { .. }
        ) {
            self.with_telemetry(|telemetry| {
                telemetry
                    .transaction
                    .branch_targeted_transaction_stale_count += 1;
            });
        }
    }
}
