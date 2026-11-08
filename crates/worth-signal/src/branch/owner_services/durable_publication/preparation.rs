//! Candidate execution finishes before any physical publication can begin.
use super::composite::CompositePublicationAuthorization;
use super::*;
use crate::branch::owner_services::{SignalDurableBranchHandle, SignalDurableSourceDenial};
use crate::data::error::SignalError;
use crate::logic::transaction::SignalTransaction;
use worth_execution::ExecutionRequest;

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
        self.begin_prepared(execution, expected, runtime_ctx, apply, None, None)
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
            None,
        )
    }
    pub(super) fn begin_prepared<F>(
        &self,
        execution: ExecutionRequest<'_, '_>,
        expected: &SignalDurableBranchHandle,
        runtime_ctx: &mut Ctx,
        apply: F,
        change: Option<Box<crate::durability::SignalDurableConditionalDefinitionChange>>,
        authorization: Option<CompositePublicationAuthorization>,
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
        let mut registered = state.register_prepared(
            &execution,
            ReservedSignalBranches::Advanced(branch),
            authorization,
        )?;
        let original = registered.original;
        // Only this target cell's lock is held by the existing canonical kernel.
        // No publication table, source-index or physical-turn lock spans apply.
        let prepared = cell.prepare_durable_advance(
            &owner,
            &admission,
            execution,
            expected.basis(),
            runtime_ctx,
            registered.recorded_nodes.clone(),
            change.as_deref(),
            apply,
        )?;
        let phase = state.prepare_frames(
            original,
            registered.new_admission,
            prepared,
            retention,
            change.map(|change| change.publication),
            registered.association.take(),
            &execution,
        )?;
        registered.seal(phase)
    }
}
