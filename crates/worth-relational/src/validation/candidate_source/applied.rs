//! Prepare managed Sources after Native prerequisites and before custom invariants.
use super::preparation::{ManagedIndexSourcePreparation, PreparedSourceProgram, SourceViews};
use super::staged::PreparedManagedIndexSources;
use super::SourcePreparationGroup;
use super::{
    ManagedIndexSourceAvailability as Availability, ManagedIndexSourcePreparationDenial as Denial,
};
use crate::authority::commit::phases::applied_mutation::AppliedMutationDraft;
use crate::authority::commit::pipeline::candidate_invariant_inputs::CandidateInvariantInputs;
use crate::branch::selected_read::SelectedRelationalBranchRead;
use crate::derived_relation::read_denial::DerivedRelationReadDenial;
use crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::storage::overlay::OverlayStateView;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use crate::validation::custom_rule::structural_views::StructuralAspectStateView;
use crate::validation::custom_rule::structural_views::StructuralRelationView;
use crate::validation::custom_rule::CustomInvariantWorkMeter;
use crate::validation::engine::state_view::InvariantStateView;
use crate::validation::engine::{InvariantObservation, InvariantRuntimeView};
use std::sync::Arc;
pub(super) mod context;
pub(super) mod discovery;
pub(super) mod stage;
use context::SourcePreparationContext;
use discovery::{source_record_is_relevant, DiscoveredSource};

/// The selected invariant and publication owners for the real Source preparation.
pub(crate) struct SourcePreparationInputs<'a> {
    pub(crate) validation: CandidateInvariantInputs<'a>,
    pub(crate) publication: RelationalRuntimePublicationBinding,
    pub(crate) symbols: &'a RuntimeSymbolTable,
}

impl AppliedMutationDraft {
    pub(crate) fn prepare_managed_sources<'a>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        inputs: impl Into<SourcePreparationInputs<'a>>,
        selected: SelectedRelationalBranchRead<'_>,
        proposal: &crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
        authored: Option<&crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan>,
    ) -> Result<(), TransactionCommitError> {
        let inputs = inputs.into();
        crate::execution::transaction_stage(
            &execution,
            inputs.validation.commit_work_budget,
            || {
                self.prepare_managed_sources_in_original_stage(
                    execution, &inputs, selected, proposal, authored,
                )
            },
        )
    }
    fn prepare_managed_sources_in_original_stage(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        inputs: &SourcePreparationInputs<'_>,
        selected: SelectedRelationalBranchRead<'_>,
        proposal: &crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
        authored: Option<&crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan>,
    ) -> Result<(), TransactionCommitError> {
        let basis = self.source_interpretation.physical_basis();
        let commit_error = |cause: Denial| {
            cause.into_commit_error(
                basis.identity().branch_id().clone(),
                basis.observation().commit_id().map(|id| id.0),
                basis.observation().version_id(),
            )
        };
        execution.check_live().map_err(|cause| {
            commit_error(Denial::from(DerivedRelationReadDenial::Request(
                cause.into(),
            )))
        })?;
        let coverage = self
            .source_interpretation
            .home_coverage
            .lock()
            .unwrap_or_else(|cause| cause.into_inner())
            .clone();
        let full_records = coverage
            .as_ref()
            .map(|coverage| {
                coverage
                    .include_changed(&execution, &self.outcome.effect.publication.changed_records)
            })
            .transpose()?;
        let context = SourcePreparationContext {
            state: self.state(),
            outcome: &self.outcome,
            allocations: self.identities.records.canonical(),
            interpretation: &self.source_interpretation,
            registry: &inputs
                .validation
                .schema_contract_runtime
                .custom_invariant_registries,
            home_target: inputs.validation.home_target,
            execution,
            selected,
            proposal,
            records: full_records
                .as_deref()
                .unwrap_or(&self.outcome.effect.publication.changed_records),
        };
        let groups = context.affected_groups();
        let history_work = authored.is_some()
            && context.registry.source_history.has_programs()
            && self.source_interpretation.materialization_mode().is_none();
        let result = if groups.is_empty() && !history_work {
            execution
                .check_live()
                .map_err(|cause| Denial::from(DerivedRelationReadDenial::Request(cause.into())))
                .and_then(|()| install_history(self, None))
                .and_then(|()| unaffected(&self.source_interpretation))
        } else {
            stage::prepare(execution, |budget| {
                let (history, mut discovered) = super::authored_history::preparation::prepare(
                    &context,
                    budget,
                    authored,
                    || inputs.validation.select(selected.selected().state()),
                )?;
                install_history(self, history)?;
                prepare(&context, inputs, budget, &groups, &mut discovered)
            })
        };
        if result.is_ok() {
            let original = self
                .source_interpretation
                .home_coverage
                .lock()
                .unwrap_or_else(|cause| cause.into_inner())
                .take();
            drop(original);
        }
        result.map_err(commit_error)
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
fn unaffected(interpretation: &super::NativeManagedSourceInterpretation) -> Result<(), Denial> {
    interpretation
        .prepared
        .set(PreparedManagedIndexSources {
            registration_generation: 0,
            programs: Vec::new(),
        })
        .map_err(|_| Denial::IncompletePreparation)
}
fn prepare(
    context: &SourcePreparationContext<'_>,
    inputs: &SourcePreparationInputs<'_>,
    budget: &crate::validation::custom_rule::CustomPreparationBudget,
    groups: &[&SourcePreparationGroup],
    discovered: &mut Vec<DiscoveredSource<'_>>,
) -> Result<(), Denial> {
    let SourcePreparationContext {
        state,
        outcome,
        interpretation,
        execution,
        selected,
        proposal,
        records,
        home_target,
        ..
    } = *context;
    execution
        .check_live()
        .map_err(|cause| DerivedRelationReadDenial::Request(cause.into()))?;
    if groups.is_empty() {
        return unaffected(interpretation);
    }
    let before = InvariantStateView::new(&selected, selected.version_id());
    let overlay = OverlayStateView::new(&selected, state);
    let after = InvariantStateView::new(&overlay, proposal.proposed_version_id());
    let epoch = inputs
        .publication
        .companion_epoch()
        .map_err(Denial::Registration)?;
    let (generation, participant) = epoch
        .active()
        .map_err(Denial::Registration)?
        .ok_or(Denial::MissingParticipant)?;
    // A private replay mode is not live suspension/rematerialization custody.
    if interpretation.materialization_mode().is_some()
        && interpretation.materialization_source().is_none()
    {
        return Err(Denial::ForeignSource);
    }
    let runtime_view = inputs.validation.select(selected.selected().state());
    let correspondence = super::correspondence::prepare(
        super::correspondence::AppliedCorrespondenceInputs {
            owner: interpretation,
            changed_records: &outcome.effect.publication.changed_records,
            symbols: inputs.symbols,
        },
        execution,
        &after,
        generation,
        |commit| inputs.validation.history.canonical_stream_position(commit),
    )?;
    let observation = InvariantObservation::applied_with_proposal(
        OverlayStateView::new(&selected, state),
        selected.version_id(),
        Some(proposal.clone()),
        &outcome.created_entities,
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
                        let source_records =
                            changed_source_records(program, records, &before, &after);
                        let closure = if home_target {
                            super::scope_closure::discover_home_target(
                                runtime_view.selected_registry.registry(),
                                program,
                                records,
                                &physical.after,
                            )?
                        } else {
                            super::scope_closure::discover(
                                runtime_view.selected_registry.registry(),
                                program,
                                &source_records,
                                &outcome.effect.adjacency.deltas,
                                &physical.before,
                                &physical.after,
                            )?
                        };
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
            interpretation,
            correspondence: correspondence.as_ref(),
            program,
            closure: &closure,
            views: logical,
            availability,
            basis: interpretation.physical_basis(),
            committed_available: !home_target
                && interpretation
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
    interpretation
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
