//! Prepare managed Sources after Native prerequisites and before custom invariants.
use super::preparation::{ManagedIndexSourcePreparation, PreparedSourceProgram, SourceViews};
use super::staged::PreparedManagedIndexSources;
use super::SourcePreparationGroup;
use super::{
    ManagedIndexSourceAvailability as Availability, ManagedIndexSourcePreparationDenial as Denial,
};
use crate::authority::commit::phases::applied_mutation::AppliedMutationDraft;
use crate::branch::SelectedRelationalBranchRead;
use crate::derived_relation::DerivedRelationReadDenial;
use crate::runtime::RelationalPreparationRuntime;
use crate::storage::overlay::OverlayStateView;
use crate::transactions::data::{RecordRef, TransactionCommitError};
use crate::validation::custom_rule::CustomInvariantWorkMeter;
use crate::validation::data::{StructuralAspectStateView, StructuralRelationView};
use crate::validation::engine::state_view::InvariantStateView;
use crate::validation::engine::{InvariantObservation, InvariantRuntimeView};
use std::sync::Arc;
mod context;
mod discovery;
pub(super) mod stage;
pub(in crate::validation::candidate_source) use context::SourcePreparationContext;
pub(in crate::validation::candidate_source) use discovery::DiscoveredSource;

impl AppliedMutationDraft {
    pub(crate) fn prepare_managed_sources(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalPreparationRuntime,
        selected: SelectedRelationalBranchRead<'_>,
        proposal: &crate::mvcc::RelationalMutationProposalIdentity,
        authored: Option<&crate::transactions::data::MergedCommitPlan>,
    ) -> Result<(), TransactionCommitError> {
        execution.check_live().map_err(|cause| {
            Denial::from(DerivedRelationReadDenial::Request(cause.into()))
                .into_commit_error(self.source_interpretation.physical_basis())
        })?;
        let context = SourcePreparationContext {
            draft: self,
            execution,
            runtime,
            selected,
            proposal,
        };
        let groups = context.affected_groups();
        let history_work = authored.is_some()
            && runtime
                .schema_contract_runtime
                .custom_invariant_registries
                .source_history
                .has_programs()
            && self.source_interpretation.materialization_mode().is_none();
        let result = if groups.is_empty() && !history_work {
            execution
                .check_live()
                .map_err(|cause| Denial::from(DerivedRelationReadDenial::Request(cause.into())))
                .and_then(|()| install_history(self, None))
                .and_then(|()| unaffected(self))
        } else {
            stage::prepare(execution, |budget| {
                let (history, mut discovered) =
                    super::authored_history::prepare(&context, budget, authored)?;
                install_history(self, history)?;
                prepare(&context, budget, &groups, &mut discovered)
            })
        };
        result.map_err(|cause| cause.into_commit_error(self.source_interpretation.physical_basis()))
    }
}

fn install_history(
    draft: &AppliedMutationDraft,
    history: Option<Arc<super::authored_history::PreparedSourceImpactHistory>>,
) -> Result<(), Denial> {
    draft
        .source_interpretation
        .authored_history
        .set(history)
        .map_err(|_| Denial::IncompletePreparation)
}
fn unaffected(draft: &AppliedMutationDraft) -> Result<(), Denial> {
    draft
        .source_interpretation
        .prepared
        .set(PreparedManagedIndexSources {
            registration_generation: 0,
            programs: Vec::new(),
        })
        .map_err(|_| Denial::IncompletePreparation)
}
fn prepare(
    context: &SourcePreparationContext<'_>,
    budget: &crate::validation::custom_rule::CustomPreparationBudget,
    groups: &[&SourcePreparationGroup],
    discovered: &mut Vec<DiscoveredSource<'_>>,
) -> Result<(), Denial> {
    let SourcePreparationContext {
        draft,
        execution,
        runtime,
        selected,
        proposal,
    } = *context;
    execution
        .check_live()
        .map_err(|cause| DerivedRelationReadDenial::Request(cause.into()))?;
    if groups.is_empty() {
        return unaffected(draft);
    }
    let before = InvariantStateView::new(&selected, selected.version_id());
    let overlay = OverlayStateView::new(&selected, draft.state());
    let after = InvariantStateView::new(&overlay, proposal.proposed_version_id());
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
    let correspondence =
        super::correspondence::prepare(draft, execution, runtime, &after, generation)?;
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
    // Discover every required program before invoking provider maintenance.
    // Compatible requirements share the union of invocation scopes, including
    // an unchanged required index in a scope changed by a sibling requirement.
    let mut plans: Vec<SourcePlan<'_>> = Vec::new();
    for group in groups {
        let mut invocation = std::collections::BTreeSet::new();
        for program in group.programs() {
            program
                .require_access(group.access_contract())
                .map_err(DerivedRelationReadDenial::from)?;
            if let Some(plan) = plans.iter().find(|plan| plan.program == program) {
                invocation.extend(plan.affected_scopes.iter().copied());
                continue;
            }
            let (closure, work) =
                match DiscoveredSource::take(discovered, context, group, program, budget)? {
                    Some(discovery) => discovery,
                    None => {
                        let work = CustomInvariantWorkMeter::new(group.maximum_work_units())
                            .with_lease(budget);
                        let physical = views(
                            &runtime_view,
                            &observation,
                            before.clone(),
                            after.clone(),
                            execution,
                            group,
                            &work,
                        );
                        let source_records = changed_source_records(
                            program,
                            &draft.outcome.effect.publication.changed_records,
                            &before,
                            &after,
                        );
                        let closure = super::scope_closure::discover(
                            runtime_view.selected_registry.registry(),
                            program,
                            &source_records,
                            &draft.outcome.effect.adjacency.deltas,
                            &physical.before,
                            &physical.after,
                        )?;
                        (closure, work)
                    }
                };
            let affected_scopes = closure
                .scopes
                .keys()
                .copied()
                .collect::<std::collections::BTreeSet<_>>();
            invocation.extend(affected_scopes.iter().copied());
            plans.push(SourcePlan {
                program,
                group,
                closure,
                affected_scopes,
                work,
            });
        }
        for plan in plans
            .iter_mut()
            .filter(|plan| group.programs().contains(plan.program))
        {
            for scope in &invocation {
                plan.closure.scopes.entry(*scope).or_default();
            }
        }
    }
    let mut programs = Vec::new();
    for SourcePlan {
        program,
        group,
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
            group,
            &work,
        );
        let source = ManagedIndexSourcePreparation {
            interpretation: &draft.source_interpretation,
            correspondence: correspondence.as_ref(),
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
    group: &'a SourcePreparationGroup,
    closure: super::scope_closure::PreparedManagedIndexScopeClosure,
    affected_scopes: std::collections::BTreeSet<crate::identity::data::EntityId>,
    work: CustomInvariantWorkMeter,
}

/// Routing reads Native metadata, then grants projection only to the installed
/// Source audience. Unrelated kinds in a mixed mutation are not Source reads.
fn changed_source_records(
    program: &super::ManagedIndexSourceProgram,
    records: &[RecordRef],
    before: &InvariantStateView<'_>,
    after: &InvariantStateView<'_>,
) -> Vec<RecordRef> {
    records
        .iter()
        .filter(|record| source_record_is_relevant(program, record, before, after))
        .cloned()
        .collect()
}
fn source_record_is_relevant(
    program: &super::ManagedIndexSourceProgram,
    record: &RecordRef,
    before: &InvariantStateView<'_>,
    after: &InvariantStateView<'_>,
) -> bool {
    let RecordRef::Entity(id) = record else {
        return false;
    };
    [before, after]
        .into_iter()
        .filter_map(|view| view.entity_metadata(*id))
        .any(|metadata| {
            metadata.kind_id == program.scope_kind()
                || metadata.kind_id == program.row_kind()
                || program
                    .physical_sources()
                    .any(|&(_, from, to)| metadata.kind_id == from || metadata.kind_id == to)
        })
}

fn views<'a>(
    runtime: &InvariantRuntimeView<'a>,
    observation: &InvariantObservation<'a>,
    before: InvariantStateView<'a>,
    after: InvariantStateView<'a>,
    execution: worth_execution::ExecutionRequest<'a, 'a>,
    group: &SourcePreparationGroup,
    work: &CustomInvariantWorkMeter,
) -> SourceViews<'a> {
    let access = Arc::new(group.access_contract().clone());
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
