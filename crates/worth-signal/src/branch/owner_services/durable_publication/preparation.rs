//! Candidate execution finishes before any physical publication can begin.
use super::*;
use crate::branch::owner_services::{SignalDurableBranchHandle, SignalDurableSourceDenial};
use crate::data::error::SignalError;
use crate::logic::transaction::SignalTransaction;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurableSignalBranchIdentity;

/// An executing callback never leaves a nameless reservation after unwinding.
struct PreparingAdvance<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    state: Arc<SignalDurablePublicationState<D, I, T>>,
    original: SignalBranchAdvanceIdentity,
    armed: bool,
}
impl<D, I, T> Drop for PreparingAdvance<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    fn drop(&mut self) {
        if self.armed {
            // No media method is reachable while this reservation is Preparing.
            // Poison recovery here releases owned resources only; subsequent
            // operation entry still reports the poisoned publication state.
            let mut table = self
                .state
                .tasks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let slot = table.tasks.remove(&self.original);
            if let Some(slot) = &slot {
                let guard = slot
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                table.branches.remove(&guard.branch);
            }
            drop(table);
            drop(slot);
        }
    }
}
impl<D, I, E, Ctx, T> SignalDurableBranchMutationPort<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Standalone Signal advance. Composite publication has a separate original
    /// intent authorization boundary and cannot inject an association here.
    pub fn begin_advance<F>(
        &self,
        execution: ExecutionRequest<'_, '_>,
        expected: &SignalDurableBranchHandle,
        runtime_ctx: &mut Ctx,
        apply: F,
    ) -> Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial>
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        self.begin_prepared(execution, expected, runtime_ctx, apply, None)
    }
    /// Publish one conditional successor on its actual owner; no conditional
    /// evaluation service or mutable graph is issued to the caller.
    pub fn begin_definition_publication(
        &self,
        execution: ExecutionRequest<'_, '_>,
        expected: &SignalDurableBranchHandle,
        runtime_ctx: &mut Ctx,
        claimant: crate::data::aspect::SignalAspectLoweringOwner,
        predecessor: crate::data::conditional_execution::InstalledSignalConditionalContract,
        definition: crate::data::conditional_execution::SignalConditionalContractDefinition,
        publication: Box<dyn crate::durability::SignalDurableDefinitionPublication>,
    ) -> Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial> {
        self.begin_prepared(
            execution,
            expected,
            runtime_ctx,
            |_| Ok(()),
            Some(Box::new(
                crate::durability::SignalDurableConditionalDefinitionChange {
                    claimant,
                    predecessor,
                    definition,
                    publication,
                },
            )),
        )
    }
    fn begin_prepared<F>(
        &self,
        execution: ExecutionRequest<'_, '_>,
        expected: &SignalDurableBranchHandle,
        runtime_ctx: &mut Ctx,
        apply: F,
        change: Option<Box<crate::durability::SignalDurableConditionalDefinitionChange>>,
    ) -> Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial>
    where
        F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
    {
        use SignalDurableAdvanceDenial as Denial;
        let state = self.state.upgrade().ok_or(Denial::OwnerUnavailable)?;
        if let Some(original) = *state
            .unsettled
            .lock()
            .map_err(|_| Denial::PublicationStatePoisoned)?
        {
            return Err(Denial::PublicationUnsettled { original });
        }
        if expected.descriptor().owner() != state.medium.binding().owner() {
            return Err(Denial::Source(SignalDurableSourceDenial::ForeignOwner));
        }
        state
            .medium
            .binding()
            .incarnation()
            .fence()
            .admit(expected.fence())
            .map_err(|denial| Denial::Source(SignalDurableSourceDenial::HomeIncarnation(denial)))?;
        let owner = state.owner()?;
        if !owner.basis_has_owner_affinity(expected.basis()) {
            return Err(Denial::Source(SignalDurableSourceDenial::ForeignBasis));
        }
        execution.consult().map_err(Denial::Request)?;
        let admission = owner
            .admit()
            .map_err(super::super::mutation_port::map_advance_admission_denial)
            .map_err(Denial::Mutation)?;
        let branch = expected.basis().owner_branch_id();
        let cell = owner
            .lookup_cell(&admission, branch)
            .map_err(|denial| {
                super::super::mutation_port::map_advance_registry_denial(denial, branch)
            })
            .map_err(Denial::Mutation)?;
        let retention = owner
            .reserve_admitted_retention(&admission, branch, 1)
            .map_err(|denial| Denial::Source(SignalDurableSourceDenial::Retention(denial)))?;
        let _memory = task_storage::reserve::<D, I, T>(&execution)?;
        let (original, new_admission, recorded_nodes) = {
            let mut table = state
                .tasks
                .lock()
                .map_err(|_| Denial::PublicationStatePoisoned)?;
            if let Some(original) = table.branches.get(&branch) {
                return Err(Denial::BranchReserved {
                    original: *original,
                });
            }
            let mut source = state
                .source
                .lock()
                .map_err(|_| Denial::PublicationStatePoisoned)?;
            let (original, new_admission) = source
                .frontiers
                .issue_advance_and_admission()
                .map_err(Denial::Readmission)?;
            let durable_branch = DurableSignalBranchIdentity::from_recorded(
                state.medium.binding().owner(),
                branch.0,
            );
            let recorded_nodes = source.frontiers.node_issuance_root(durable_branch);
            let slot = Arc::new(Mutex::new(TaskSlot {
                branch,
                resource: Some(LinearResource::mint(original, &state.authority)),
                phase: TaskPhase::Preparing,
                _memory,
            }));
            table.branches.insert(branch, original);
            table.tasks.insert(original, slot);
            (original, new_admission, recorded_nodes)
        };
        let mut reservation = PreparingAdvance {
            state: Arc::clone(&state),
            original,
            armed: true,
        };
        // Only this target cell's lock is held by the existing canonical kernel.
        // No publication table, source-index or physical-turn lock spans apply.
        let prepared = cell.prepare_durable_advance(
            &owner,
            &admission,
            execution,
            expected.basis(),
            runtime_ctx,
            recorded_nodes,
            change.as_deref(),
            apply,
        )?;
        let phase = state.prepare_frames(
            original,
            new_admission,
            prepared,
            retention,
            change.map(|change| change.publication),
            &execution,
        )?;
        let slot = state.task(original)?;
        slot.lock()
            .map_err(|_| Denial::PublicationStatePoisoned)?
            .phase = phase;
        reservation.armed = false;
        Ok(state.handle(original))
    }
}
