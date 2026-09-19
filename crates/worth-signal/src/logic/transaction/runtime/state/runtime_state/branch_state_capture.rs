use crate::logic::checkpoint::CheckpointRuntime;

use super::super::branching::{BranchAncestryState, BranchState};

use super::super::merge::BranchMutationLedger;

use super::super::reconstructability::{AuthorityState, DerivedState};

use super::super::resource::ResourceRuntimeState;

use super::SignalRuntime;

#[derive(Debug)]
pub(in crate::logic::transaction::runtime) struct HeavyCaptureWitness(());

impl<D, I, E, Ctx, T> SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::logic::transaction::runtime::state) fn capture_full_authority_state(
        &self,
    ) -> AuthorityState<T> {
        AuthorityState::capture(&self.graph, &self.config)
    }

    pub(in crate::logic::transaction::runtime::state) fn capture_full_derived_state(
        &self,
    ) -> DerivedState<D, I> {
        let telemetry = if self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        ) {
            &self.telemetry
        } else {
            &crate::data::telemetry::RuntimeTelemetry::default()
        };
        DerivedState::capture(&self.checkpoint, &self.resource, &self.temporal, telemetry)
    }

    fn heavy_capture_witness(&mut self) -> Option<HeavyCaptureWitness> {
        if self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        ) {
            self.telemetry.transaction.heavy_capture_count += 1;
            Some(HeavyCaptureWitness(()))
        } else {
            None
        }
    }

    pub(in crate::logic::transaction::runtime::state) fn ensure_managed_queue_branch_transfer_allowed(
        resource: &ResourceRuntimeState,
    ) -> Result<(), crate::data::error::SignalError> {
        let bound_queue_count = resource.bound_managed_queue_count();
        if bound_queue_count == 0 {
            return Ok(());
        }
        Err(
            crate::data::error::SignalError::managed_queue_branch_transfer_denied(
                bound_queue_count,
            ),
        )
    }

    pub(in crate::logic::transaction::runtime::state) fn ensure_branch_state_managed_queue_transfer_allowed(
        &self,
        state: &BranchState<D, I, T>,
    ) -> Result<(), crate::data::error::SignalError> {
        Self::ensure_managed_queue_branch_transfer_allowed(&self.resource)?;
        Self::ensure_managed_queue_branch_transfer_allowed(state.resource())
    }

    pub(in crate::logic::transaction::runtime::state) fn capture_heavy_branch_state(
        &mut self,
    ) -> Result<BranchState<D, I, T>, crate::data::error::SignalError> {
        Self::ensure_managed_queue_branch_transfer_allowed(&self.resource)?;
        let _witness = self.heavy_capture_witness();
        let handle = self.graph.current_branch();
        let ancestry = self
            .branches
            .branch_ancestry_state(handle.id)
            .cloned()
            .unwrap_or(BranchAncestryState::new(
                handle.id,
                handle.parent_branch_id,
                handle.head_snapshot_id,
            ));
        let mut mutation_ledger = self
            .branches
            .branch_mutation_ledger(handle.id)
            .cloned()
            .unwrap_or_else(|| {
                BranchMutationLedger::default().with_baseline_snapshot(handle.head_snapshot_id)
            });
        mutation_ledger.absorb_records(self.graph.pending_branch_mutation_records());
        self.graph.clear_branch_mutation_nodes();
        Ok(self.branches.capture_active_state(
            self.capture_full_authority_state(),
            self.capture_full_derived_state(),
            ancestry,
            mutation_ledger,
        ))
    }

    pub(in crate::logic::transaction::runtime::state) fn take_heavy_active_branch_state(
        &mut self,
    ) -> Result<BranchState<D, I, T>, crate::data::error::SignalError> {
        Self::ensure_managed_queue_branch_transfer_allowed(&self.resource)?;
        let _witness = self.heavy_capture_witness();
        let captures_telemetry = self.graph.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::OptionalTelemetry,
        );
        let handle = self.graph.current_branch();
        let ancestry = self
            .branches
            .branch_ancestry_state(handle.id)
            .cloned()
            .unwrap_or(BranchAncestryState::new(
                handle.id,
                handle.parent_branch_id,
                handle.head_snapshot_id,
            ));
        let mut mutation_ledger = self
            .branches
            .branch_mutation_ledger(handle.id)
            .cloned()
            .unwrap_or_else(|| {
                BranchMutationLedger::default().with_baseline_snapshot(handle.head_snapshot_id)
            });
        mutation_ledger.absorb_records(self.graph.pending_branch_mutation_records());
        self.graph.clear_branch_mutation_nodes();

        let authority = AuthorityState {
            graph: *std::mem::take(&mut self.graph),
            config: std::mem::take(&mut self.config),
        };
        let checkpoint_policy = self.checkpoint.policy().clone();
        let derived = DerivedState {
            checkpoint: std::mem::replace(
                &mut self.checkpoint,
                CheckpointRuntime::new(checkpoint_policy),
            ),
            resource: std::mem::take(&mut self.resource),
            temporal: std::mem::take(&mut self.temporal),
            telemetry: if captures_telemetry {
                std::mem::take(&mut self.telemetry)
            } else {
                crate::data::telemetry::RuntimeTelemetry::default()
            },
        };
        Ok(self
            .branches
            .capture_active_state(authority, derived, ancestry, mutation_ledger))
    }
}
