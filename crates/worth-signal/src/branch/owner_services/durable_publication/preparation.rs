//! Candidate execution finishes before any physical publication can begin.
use super::*;
use crate::branch::owner_services::{SignalDurableBranchHandle, SignalDurableSourceDenial};
use crate::data::error::SignalError;
use crate::durability::{SignalAuthoredBranchChange, SignalDurableBranchDescriptor};
use crate::logic::transaction::SignalTransaction;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{DurableSignalBranchIdentity, DurableSignalGeneration};

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
            apply,
        )?;
        let phase =
            state.prepare_frames(original, new_admission, prepared, retention, &execution)?;
        let slot = state.task(original)?;
        slot.lock()
            .map_err(|_| Denial::PublicationStatePoisoned)?
            .phase = phase;
        reservation.armed = false;
        Ok(state.handle(original))
    }
}
impl<D, I, T> SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    fn prepare_frames(
        &self,
        original: SignalBranchAdvanceIdentity,
        admission: DurableSignalAdmissionIdentity,
        prepared: SignalPreparedDurableAdvance<D, I, T>,
        retention: SignalBranchAdmissionReservation,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<TaskPhase<D, I, T>, SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        let branch = prepared.predecessor.branch();
        let location =
            SignalAdvanceLocationReservation::reserve(execution).map_err(Dn::Readmission)?;
        if prepared.source_changed {
            let (identity, previous_source) = {
                let mut source = self
                    .source
                    .lock()
                    .map_err(|_| Dn::PublicationStatePoisoned)?;
                let identity = source
                    .frontiers
                    .issue_definition()
                    .map_err(Dn::Readmission)?;
                source
                    .frontiers
                    .observe_graph(branch, prepared.state.graph(), execution)
                    .map_err(Dn::Readmission)?;
                let previous = source
                    .definition_record(&prepared.predecessor)
                    .map_err(Dn::Readmission)?;
                (identity, previous)
            };
            let previous = previous_source
                .read(self.medium.as_ref(), execution)
                .map_err(Dn::Readmission)?;
            let image = prepared
                .state
                .graph()
                .capture_authored_image(execution, branch)
                .map_err(Dn::Image)?;
            let runtime = prepared
                .state
                .config()
                .capture_authored_image(prepared.state.graph(), &self.vocabulary, branch, execution)
                .map_err(Dn::Image)?;
            let occurrences = previous.conditional_occurrences().collect::<Vec<_>>();
            let definition = SignalDefinitionTransitionArtifact::capture(
                execution,
                identity,
                branch,
                Some(prepared.predecessor.definition()),
                &image,
                &runtime,
                &occurrences,
                &previous.bridge_section(),
            )
            .map_err(Dn::Frame)?;
            let descriptor = SignalDurableBranchDescriptor::from_recorded(
                DurableSignalGeneration::from_recorded(
                    prepared.predecessor.cell(),
                    prepared.generation,
                ),
                prepared.predecessor.snapshot(),
                admission,
                definition.reference(),
            )
            .map_err(Dn::Frame)?;
            let artifact = SignalBranchAdvanceArtifact::capture(
                execution,
                original,
                descriptor,
                None,
                SignalAuthoredBranchChange::Advanced,
            )
            .map_err(Dn::Frame)?;
            Ok(TaskPhase::Definition(Box::new(DefinitionPending {
                advance: ReadyAdvance {
                    prepared,
                    artifact,
                    retention,
                    location,
                },
                definition,
                location: SignalDefinitionLocationReservation::reserve(execution)
                    .map_err(Dn::Readmission)?,
            })))
        } else {
            let definition = self
                .source
                .lock()
                .map_err(|_| Dn::PublicationStatePoisoned)?
                .definition_record(&prepared.predecessor)
                .map_err(Dn::Readmission)?;
            let descriptor = SignalDurableBranchDescriptor::from_recorded(
                DurableSignalGeneration::from_recorded(
                    prepared.predecessor.cell(),
                    prepared.generation,
                ),
                prepared.predecessor.snapshot(),
                admission,
                definition.reference(),
            )
            .map_err(Dn::Frame)?;
            let artifact = SignalBranchAdvanceArtifact::capture(
                execution,
                original,
                descriptor,
                None,
                SignalAuthoredBranchChange::Advanced,
            )
            .map_err(Dn::Frame)?;
            Ok(TaskPhase::Branch(Box::new(BranchPending {
                advance: ReadyAdvance {
                    prepared,
                    artifact,
                    retention,
                    location,
                },
                definition,
            })))
        }
    }
}
