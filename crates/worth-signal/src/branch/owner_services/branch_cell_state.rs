use crate::data::error::SignalError;
use crate::logic::transaction::BranchState;
use crate::logic::transaction::{
    SignalCanonicalCallerUnwind, SignalTransaction, SnapshotBranchState, SnapshotStatePacket,
    TransactionResult,
};
use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId, SignalSnapshotV1};
use worth_foundational::{FoundationalBranchReferenceGeneration, FoundationalBranchTarget};

use crate::branch::{signal_branch_observation, SignalBranchObservation, SignalBranchTarget};

use super::SignalOwnerMovementPermit;
mod committed_patch;
mod conditional_capture;
mod conditional_execution;
mod conditional_owned_async;
use conditional_capture::SignalPublishedConditionalExecutionBasis;

/// The one canonical mutable state payload for a sealed live branch.
///
/// Owner-service issuance moves every active and stored branch into exactly one
/// execution cell. The legacy active/stored representation is empty after that
/// transition, so this payload never mirrors graph, head, snapshot, retention,
/// or allocator truth.
pub(crate) struct SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    handle: SignalBranchHandle,
    owner_runtime_instance_id: u64,
    definition_basis: u64,
    state: BranchState<D, I, T>,
    head_generation: u64,
    restore_snapshot_id: Option<SignalSnapshotId>,
    conditional_execution_basis: Option<SignalPublishedConditionalExecutionBasis>,
}

pub(crate) struct SignalPreparedBranchSnapshot<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) state: BranchState<D, I, T>,
    pub(crate) snapshot: SignalSnapshotV1,
    pub(crate) packet: SnapshotStatePacket<D, I, T>,
    pub(crate) observation: SignalBranchObservation,
    pub(crate) generation: u64,
}

pub(crate) struct SignalPreparedBranchRestore<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) state: BranchState<D, I, T>,
    pub(crate) observation: SignalBranchObservation,
    pub(crate) generation: u64,
    pub(crate) snapshot_id: SignalSnapshotId,
}

pub(crate) struct SignalForkBranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) state: BranchState<D, I, T>,
    pub(crate) work: crate::data::graph::signal_graph::SignalGraphForkWork,
}

impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn new(
        handle: SignalBranchHandle,
        owner_runtime_instance_id: u64,
        definition_basis: u64,
        state: BranchState<D, I, T>,
        head_generation: u64,
        restore_snapshot_id: Option<SignalSnapshotId>,
    ) -> Self {
        debug_assert_eq!(handle.id, state.branch_id());
        let definition_basis = state
            .installed_definition()
            .map_or(definition_basis, |binding| binding.definition_basis());
        Self {
            handle,
            owner_runtime_instance_id,
            definition_basis,
            state,
            head_generation,
            restore_snapshot_id,
            conditional_execution_basis: None,
        }
    }

    pub(crate) fn branch_id(&self) -> SignalBranchId {
        self.handle.id
    }

    pub(crate) fn handle(&self) -> &SignalBranchHandle {
        &self.handle
    }

    pub(crate) fn state(&self) -> &BranchState<D, I, T> {
        &self.state
    }

    #[cfg(test)]
    pub(crate) fn state_mut(&mut self) -> &mut BranchState<D, I, T> {
        &mut self.state
    }

    pub(crate) fn head_generation(&self) -> u64 {
        self.head_generation
    }

    pub(crate) fn restore_snapshot_id(&self) -> Option<SignalSnapshotId> {
        self.restore_snapshot_id
    }

    pub(crate) fn observation(&self) -> Result<SignalBranchObservation, SignalError> {
        self.observation_for(
            self.head_generation,
            self.handle.head_snapshot_id,
            self.restore_snapshot_id,
        )
    }

    pub(crate) fn next_advance_observation(&self) -> Result<SignalBranchObservation, SignalError> {
        let next_generation = self.next_generation()?;
        self.observation_for(next_generation, self.handle.head_snapshot_id, None)
    }

    pub(crate) fn prepare_snapshot(
        &self,
        snapshot_id: SignalSnapshotId,
    ) -> Result<SignalPreparedBranchSnapshot<D, I, T>, SignalError> {
        let mut state = self.state.clone();
        let (snapshot, snapshot_state) = state.capture_for_owner_cell(snapshot_id)?;
        let generation = self.next_generation()?;
        let observation = self.observation_for(generation, Some(snapshot_id), None)?;
        Ok(SignalPreparedBranchSnapshot {
            state,
            packet: snapshot_state.packet(snapshot_id),
            snapshot,
            observation,
            generation,
        })
    }

    pub(crate) fn commit_snapshot(&mut self, prepared: &mut SignalPreparedBranchSnapshot<D, I, T>) {
        std::mem::swap(&mut self.state, &mut prepared.state);
        self.handle.head_snapshot_id = Some(prepared.snapshot.meta.snapshot_id);
        self.head_generation = prepared.generation;
        self.restore_snapshot_id = None;
        self.conditional_execution_basis = None;
    }

    pub(crate) fn prepare_restore(
        &self,
        snapshot_state: SnapshotBranchState<D, I, T>,
        snapshot: &SignalSnapshotV1,
    ) -> Result<SignalPreparedBranchRestore<D, I, T>, SignalError> {
        let snapshot_id = snapshot.meta.snapshot_id;
        let generation = self.next_generation()?;
        let observation = self.observation_for(generation, Some(snapshot_id), Some(snapshot_id))?;
        let state = self
            .state
            .prepare_owner_cell_restore(snapshot_state, snapshot)?;
        Ok(SignalPreparedBranchRestore {
            state,
            observation,
            generation,
            snapshot_id,
        })
    }

    pub(crate) fn commit_restore(&mut self, prepared: SignalPreparedBranchRestore<D, I, T>) {
        self.state = prepared.state;
        self.handle.head_snapshot_id = Some(prepared.snapshot_id);
        self.head_generation = prepared.generation;
        self.restore_snapshot_id = Some(prepared.snapshot_id);
        self.conditional_execution_basis = None;
    }

    pub(crate) fn fork_state(
        &mut self,
        destination: &SignalBranchHandle,
    ) -> SignalForkBranchState<D, I, T> {
        let forked = self
            .state
            .fork_for_owner_cell(&self.handle, destination.clone());
        SignalForkBranchState {
            state: forked.state,
            work: forked.work,
        }
    }

    pub(crate) fn commit_fork_source_boundary(&mut self) {
        self.state
            .mutation_ledger_mut()
            .clear_all(self.handle.head_snapshot_id);
    }

    fn next_generation(&self) -> Result<u64, SignalError> {
        FoundationalBranchReferenceGeneration::new(self.head_generation)
            .checked_advance()
            .map(FoundationalBranchReferenceGeneration::get)
            .map_err(|denial| {
                SignalError::internal(format!(
                    "Signal branch reference generation cannot advance: {denial:?}"
                ))
            })
    }

    fn observation_for(
        &self,
        generation: u64,
        head_snapshot_id: Option<SignalSnapshotId>,
        restore_snapshot_id: Option<SignalSnapshotId>,
    ) -> Result<SignalBranchObservation, SignalError> {
        let target = SignalBranchTarget::new(
            self.owner_runtime_instance_id.to_string(),
            self.definition_basis,
            head_snapshot_id.map(|snapshot_id| snapshot_id.0),
            restore_snapshot_id.map(|snapshot_id| snapshot_id.0),
        )
        .map_err(|denial| {
            SignalError::internal(format!(
                "owner cell could not construct its canonical branch target: {denial:?}"
            ))
        })?;
        signal_branch_observation(
            self.owner_runtime_instance_id.to_string(),
            self.handle.id.0,
            &self.handle.name,
            FoundationalBranchTarget::Basis(target),
            FoundationalBranchReferenceGeneration::new(generation),
        )
        .map_err(|denial| {
            SignalError::internal(format!(
                "owner cell could not construct its canonical observation: {denial:?}"
            ))
        })
    }

    pub(crate) fn execute_canonical_transaction<E, Ctx, F>(
        &mut self,
        _permit: &SignalOwnerMovementPermit<'_>,
        conditional_operation_scope: super::conditional_execution::SignalConditionalOperationScopeBinding,
        runtime_ctx: &mut Ctx,
        apply: F,
    ) -> Result<Result<TransactionResult, SignalError>, SignalCanonicalCallerUnwind>
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        self.state.execute_canonical_transaction(
            &mut self.head_generation,
            &mut self.restore_snapshot_id,
            Some(conditional_operation_scope),
            runtime_ctx,
            apply,
        )
    }
}
