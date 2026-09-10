use crate::clock::RuntimeInstant;
use crate::data::error::SignalError;

use super::super::computation::{DefinedComputation, Recipe};
use super::super::transaction::{
    SignalTransaction, TransactionCommitPosture, TransactionExecutionState, TransactionResult,
    TransactionScratch,
};
use super::runtime_state::SignalRuntime;

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn define<F>(
        &mut self,
        recipe: Recipe<T, F>,
    ) -> Result<DefinedComputation<T, F>, SignalError> {
        self.config.define_computation(
            recipe.family.clone(),
            recipe.contract.clone(),
            recipe.tier,
            recipe.comparator.clone(),
        )?;
        Ok(DefinedComputation::from_recipe(recipe))
    }

    pub(crate) fn begin<'a>(
        &'a mut self,
        runtime_ctx: &'a mut Ctx,
    ) -> SignalTransaction<'a, D, I, E, Ctx, T> {
        let captures_telemetry = self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        );
        if captures_telemetry {
            self.telemetry.transaction.transaction_begin_count += 1;
        }
        self.config.sync_graph_capacity(&self.graph);
        let current_branch = self.graph.current_branch();
        let (branch_mutation_ledger, branch_head_generation, branch_restore_snapshot_id) = self
            .branches
            .transaction_branch_state_mut(current_branch.id, current_branch.head_snapshot_id);
        SignalTransaction {
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
            rollback_packets: super::super::transaction::TransactionRollbackPacketSet::default(),
            poisoned: false,
            finished: false,
            execution_state: TransactionExecutionState::default(),
            started_at: RuntimeInstant::now(),
            commit_posture: TransactionCommitPosture::Visible,
        }
    }

    pub(crate) fn transaction<F>(
        &mut self,
        runtime_ctx: &mut Ctx,
        apply: F,
    ) -> Result<TransactionResult, SignalError>
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        let mut transaction = self.begin(runtime_ctx);
        match apply(&mut transaction) {
            Ok(()) => transaction.commit(),
            Err(err) => {
                let rollback_result = transaction.rollback();
                match rollback_result {
                    Ok(_) => Err(err),
                    Err(rollback_err) => Err(rollback_err),
                }
            }
        }
    }
}
