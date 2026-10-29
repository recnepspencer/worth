//! Install the original-request preparation phase before any invariant family.
use super::preparation::{ManagedIndexSourcePreparation, PreparedSourceProgram, SourceViews};
use super::staged::PreparedManagedIndexSources;
use super::{
    ManagedIndexSourceAvailability as Availability, ManagedIndexSourcePreparationDenial as Denial,
};
use crate::authority::commit::phases::applied_mutation::AppliedMutationDraft;
use crate::branch::SelectedRelationalBranchRead;
use crate::derived_relation::{DerivedRelationDenial, DerivedRelationReadDenial};
use crate::identity::data::KindId;
use crate::runtime::RelationalPreparationRuntime;
use crate::storage::overlay::OverlayStateView;
use crate::transactions::data::{RecordRef, TransactionCommitError};
use crate::validation::custom_rule::{CustomInvariantWorkMeter, CustomPreparationBudget};
use crate::validation::data::{
    CustomInvariantRegistration, StructuralAspectStateView, StructuralRelationView,
};
use crate::validation::engine::state_view::InvariantStateView;
use crate::validation::engine::{InvariantObservation, InvariantRuntimeView};
use std::sync::Arc;

impl AppliedMutationDraft {
    pub(crate) fn prepare_managed_sources(
        &mut self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalPreparationRuntime,
        selected: SelectedRelationalBranchRead<'_>,
        proposal: &crate::mvcc::RelationalMutationProposalIdentity,
    ) -> Result<(), TransactionCommitError> {
        prepare(self, execution, runtime, selected, proposal).map_err(|cause| {
            TransactionCommitError::preparation(
                crate::transactions::data::CommitPreparationError::managed_index_source(
                    self.source_interpretation.physical_basis(),
                    cause,
                ),
            )
        })
    }
}

fn prepare(
    draft: &AppliedMutationDraft,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    selected: SelectedRelationalBranchRead<'_>,
    proposal: &crate::mvcc::RelationalMutationProposalIdentity,
) -> Result<(), Denial> {
    execution
        .check_live()
        .map_err(|cause| DerivedRelationReadDenial::Request(cause.into()))?;
    let before = InvariantStateView::new(&selected, selected.version_id());
    let overlay = OverlayStateView::new(&selected, draft.state());
    let after = InvariantStateView::new(&overlay, proposal.proposed_version_id());
    let mut kinds = Vec::new();
    for record in &draft.outcome.effect.publication.changed_records {
        let RecordRef::Entity(id) = record else {
            continue;
        };
        for view in [&before, &after] {
            if let Some(metadata) = view.entity_metadata(*id) {
                kinds.push(metadata.kind_id);
            }
        }
    }
    let registrations = runtime
        .schema_contract_runtime
        .custom_invariant_registries
        .source_registrations(
            kinds.into_iter(),
            draft
                .outcome
                .effect
                .adjacency
                .deltas
                .iter()
                .map(|delta| delta.kind_id),
        );
    if registrations.is_empty() {
        // An actual pre-first stage proves an empty closure. Absence of that
        // stage must remain distinguishable from complete, unrelated changes.
        draft
            .source_interpretation
            .prepared
            .set(PreparedManagedIndexSources {
                registration_generation: 0,
                programs: Vec::new(),
            })
            .map_err(|_| Denial::IncompletePreparation)?;
        return Ok(());
    }
    let publication = runtime.publication_binding();
    let epoch = publication
        .companion_epoch()
        .map_err(Denial::Registration)?;
    let (generation, participant) = epoch
        .active()
        .map_err(Denial::Registration)?
        .ok_or(Denial::MissingParticipant)?;
    // A private replay mode is not live suspension/rematerialization custody.
    if draft.source_interpretation.materialization_mode().is_some()
        && draft
            .source_interpretation
            .materialization_source()
            .is_none()
    {
        return Err(Denial::ForeignSource);
    }
    let runtime_view =
        InvariantRuntimeView::from_preparation_for_state(runtime, selected.selected().state());
    let observation = InvariantObservation::applied_with_proposal(
        OverlayStateView::new(&selected, draft.state()),
        selected.version_id(),
        Some(proposal.clone()),
        &draft.outcome.created_entities,
    );
    let availability = if observation.custom_uses_suspension_source() {
        Availability::Unavailable
    } else {
        Availability::Available
    };
    let budget = crate::authority::commit::pipeline::request_lease(&execution)
        .map(CustomPreparationBudget::tracked)
        .transpose()
        .map_err(|cause| {
            DerivedRelationReadDenial::Request(crate::execution::denial::lease_denial(
                worth_execution::LeaseDenial::MemoryExhausted(cause),
            ))
        })?;
    // Discover every required program before invoking provider maintenance.
    // Compatible requirements share the union of invocation scopes, including
    // an unchanged required index in a scope changed by a sibling requirement.
    let mut plans: Vec<SourcePlan<'_>> = Vec::new();
    for registration in registrations {
        let mut invocation = std::collections::BTreeSet::new();
        for program in registration.managed_sources.iter() {
            program
                .require_access(registration.access_contract())
                .map_err(DerivedRelationReadDenial::from)?;
            if let Some(plan) = plans.iter().find(|plan| plan.program == program) {
                invocation.extend(plan.affected_scopes.iter().copied());
                continue;
            }
            let mut work = CustomInvariantWorkMeter::new(registration.maximum_work_units());
            if let Some(budget) = &budget {
                work = work.with_lease(budget);
            }
            let physical = views(
                &runtime_view,
                &observation,
                before.clone(),
                after.clone(),
                execution,
                registration,
                &work,
            );
            let closure = super::scope_closure::discover(
                runtime_view.selected_registry.registry(),
                program,
                &draft.outcome.effect.publication.changed_records,
                &draft.outcome.effect.adjacency.deltas,
                &physical.before,
                &physical.after,
            )?;
            let affected_scopes = closure
                .scopes
                .keys()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            invocation.extend(affected_scopes.iter().copied());
            plans.push(SourcePlan {
                program,
                registration,
                closure,
                affected_scopes,
                work,
            });
        }
        for plan in plans
            .iter_mut()
            .filter(|plan| registration.managed_sources.contains(plan.program))
        {
            for scope in &invocation {
                plan.closure.scopes.entry(*scope).or_default();
            }
        }
    }
    let mut programs = Vec::new();
    for SourcePlan {
        program,
        registration,
        closure,
        affected_scopes,
        work,
    } in plans
    {
        let logical_after = InvariantStateView::new(
            observation.custom_enforcement_partition_access(),
            observation.custom_enforcement_version_id(proposal.proposed_version_id()),
        );
        let logical = views(
            &runtime_view,
            &observation,
            before.clone(),
            logical_after,
            execution,
            registration,
            &work,
        );
        let source = ManagedIndexSourcePreparation {
            program,
            closure: &closure,
            views: logical,
            availability,
            basis: draft.source_interpretation.physical_basis(),
            committed_available: draft
                .source_interpretation
                .physical_basis()
                .materialization_is_complete(),
        };
        let roots = participant.prepare_managed_source(&execution, &source)?;
        source.check_complete(&roots)?;
        programs.push(PreparedSourceProgram {
            program: program.clone(),
            closure,
            affected_scopes,
            roots,
            _work: work,
        });
    }
    draft
        .source_interpretation
        .prepared
        .set(PreparedManagedIndexSources {
            registration_generation: generation,
            programs,
        })
        .map_err(|_| Denial::IncompletePreparation)?;
    Ok(())
}

struct SourcePlan<'a> {
    program: &'a super::ManagedIndexSourceProgram,
    registration: &'a CustomInvariantRegistration,
    closure: super::scope_closure::PreparedManagedIndexScopeClosure,
    affected_scopes: std::collections::BTreeSet<crate::identity::data::EntityId>,
    work: CustomInvariantWorkMeter,
}

fn views<'a>(
    runtime: &InvariantRuntimeView<'a>,
    observation: &InvariantObservation<'a>,
    before: InvariantStateView<'a>,
    after: InvariantStateView<'a>,
    execution: worth_execution::ExecutionRequest<'a, 'a>,
    registration: &CustomInvariantRegistration,
    work: &CustomInvariantWorkMeter,
) -> SourceViews<'a> {
    let access = Arc::new(registration.access_contract().clone());
    SourceViews {
        before: StructuralRelationView::new(
            before.clone(),
            execution,
            work.clone(),
            access.clone(),
        )
        .with_structural_source(runtime, observation, true),
        after: StructuralRelationView::new(after.clone(), execution, work.clone(), access.clone())
            .with_structural_source(runtime, observation, false),
        before_fields: StructuralAspectStateView::new(before, work.clone(), access.clone()),
        after_fields: StructuralAspectStateView::new(after, work.clone(), access),
    }
}
