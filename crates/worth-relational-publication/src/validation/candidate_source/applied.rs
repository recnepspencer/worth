//! Prepare managed Sources after Native prerequisites and before custom invariants.
use crate::authority::commit::phases::applied_mutation::AppliedMutationDraft;
use crate::authority::commit::pipeline::candidate_invariant_inputs::CandidateInvariantInputs;
use crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding;
use contracts::derived_relation::read_denial::DerivedRelationReadDenial;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
use contracts::validation::candidate_source::availability::ManagedIndexSourceAvailability as Availability;
use contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial as Denial;
use contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenialOwnerPort as _;
use contracts::validation::candidate_source::program::ManagedIndexSourceProgramOwnerPort as _;
use mutation::validation::candidate_source::applied::context::SourcePreparationContext;
use mutation::validation::candidate_source::applied::discovery::source_record_is_relevant;
use mutation::validation::candidate_source::applied::discovery::DiscoveredSource;
pub(crate) use mutation::validation::candidate_source::applied::stage;
use mutation::validation::candidate_source::preparation::invariant_views::invariant_source_views;
use mutation::validation::candidate_source::preparation::prepare_source_loan;
use mutation::validation::candidate_source::preparation::ManagedIndexSourcePreparationInputs;
use mutation::validation::candidate_source::preparation::ManagedIndexSourcePreparationOwnerPort as _;
use mutation::validation::candidate_source::preparation::PreparedSourceProgram;
use mutation::validation::candidate_source::preparation_group::SourcePreparationGroup;
use mutation::validation::candidate_source::staged::PreparedManagedIndexSources;
use mutation::validation::custom_rule::work_meter::CustomInvariantWorkMeter;
use mutation::validation::engine::observation::InvariantObservation;
use mutation::validation::engine::state_view::InvariantStateView;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::selected_read::SelectedRelationalBranchRead;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView;

/// The selected invariant and publication owners for the real Source preparation.
pub struct SourcePreparationInputs<'a> {
    pub validation: CandidateInvariantInputs<'a>,
    pub publication: RelationalRuntimePublicationBinding,
    pub symbols: &'a RuntimeSymbolTable,
}

impl AppliedMutationDraft {
    pub fn prepare_managed_sources<'a>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        inputs: impl Into<SourcePreparationInputs<'a>>,
        selected: SelectedRelationalBranchRead<'_>,
        proposal: &mutation::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
        authored: Option<
            &contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan,
        >,
    ) -> Result<(), TransactionCommitError> {
        let inputs = inputs.into();
        contracts::execution::request_work::transaction_stage(
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
        proposal: &mutation::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
        authored: Option<
            &contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan,
        >,
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
                let (history, mut discovered) =
                    mutation::validation::candidate_source::authored_history::preparation::prepare(
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
    history: Option<Arc<worth_relational_branch::internal_port::validation::candidate_source::authored_history::PreparedSourceImpactHistory>>,
) -> Result<(), Denial> {
    draft
        .source_interpretation
        .authored_history
        .set(history)
        .map_err(|_| Denial::IncompletePreparation)
}
fn unaffected(
    interpretation: &mutation::validation::candidate_source::interpretation::NativeManagedSourceInterpretation,
) -> Result<(), Denial> {
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
    budget: &mutation::validation::custom_rule::work_meter::CustomPreparationBudget,
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
    let correspondence = mutation::validation::candidate_source::correspondence::prepare(
        mutation::validation::candidate_source::correspondence::AppliedCorrespondenceInputs {
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
            let (closure, work) = match DiscoveredSource::take(
                discovered, context, group, program, budget,
            )? {
                Some(discovery) => discovery,
                None => {
                    let work = CustomInvariantWorkMeter::new(group.maximum_work_units())
                        .with_lease(budget);
                    let physical = invariant_source_views(
                        &runtime_view,
                        &observation,
                        before.clone(),
                        after.clone(),
                        execution,
                        group,
                        &work,
                    );
                    let source_records = changed_source_records(program, records, &before, &after);
                    let closure = if home_target {
                        mutation::validation::candidate_source::scope_closure::home_target::discover_home_target(
                                runtime_view.selected_registry.registry(),
                                program,
                                records,
                                &physical.after,
                            )?
                    } else {
                        mutation::validation::candidate_source::scope_closure::discover(
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
        let logical = invariant_source_views(
            &runtime_view,
            &observation,
            before.clone(),
            logical_after,
            execution,
            group,
            &work,
        );
        let source = prepare_source_loan(ManagedIndexSourcePreparationInputs {
            interpretation,
            correspondence: correspondence.as_ref(),
            program,
            closure: &closure,
            views: logical,
            availability,
            home_target,
        });
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
    program: &'a contracts::validation::candidate_source::program::ManagedIndexSourceProgram,
    group: &'a SourcePreparationGroup,
    closure:
        mutation::validation::candidate_source::scope_closure::PreparedManagedIndexScopeClosure,
    affected_scopes: std::collections::BTreeSet<contracts::identity::data::record_ids::EntityId>,
    work: CustomInvariantWorkMeter,
}

/// Routing reads Native metadata, then grants projection only to the installed
/// Source audience. Unrelated kinds in a mixed mutation are not Source reads.
fn changed_source_records(
    program: &contracts::validation::candidate_source::program::ManagedIndexSourceProgram,
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
