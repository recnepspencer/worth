//! Suspend an admitted selected row until canonical owner application.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::execution;
use crate::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::OwnerPlan;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint;

pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression)
enum ExecutionPreparation<'owner, Query, Value> {
    Reached(OwnStages<Query, Value>),
    Planned(OwnerPlan<'owner, CheckpointProgress, CompletionRefusal>),
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression)
struct CompletionRefusal
{
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) denial:
        WorthQueryOutputDemandDenial,
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) checkpoint:
        Option<WorthQueryOutputCheckpoint>,
}
impl CompletionRefusal {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn denied(
        denial: WorthQueryOutputDemandDenial,
    ) -> Self {
        Self {
            denial,
            checkpoint: None,
        }
    }
    pub(super) fn unpublished(
        denial: WorthQueryOutputDemandDenial,
        checkpoint: WorthQueryOutputCheckpoint,
    ) -> Self {
        Self {
            denial,
            checkpoint: Some(checkpoint),
        }
    }
}

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn prepare_validated_output_demand_with_schedule<
        'owner,
        Family,
    >(
        &'owner self,
        phase: &'owner WorthQueryAdvancementPhase<'owner>,

        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        entry: &InstalledProducerProvider<Schema>,
        commit_authority: WorthQueryProducerCommitAuthority,
        schedule_progression: ScheduleProgression<'owner, '_, '_, Schema>,
        request_admission: &mut InvalidationEditAdmission,
        performed: &'owner std::cell::RefCell<PerformedMembers>,
    ) -> Result<
        ExecutionPreparation<
            'owner,
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        WorthQueryOutputDemandDenial,
    >
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let waiting = || OwnStages::Answer(WorthQueryOutputDemandAdvance::Pending);
        let disclosed_value = disclosure.value();
        let disclosed_source = disclosure.source();
        if let SourceGuard::Replaced = self.prepare_progression_entry(
            demand,
            disclosed_source,
            &commit_authority,
            entry,
            &schedule_progression,
            request_admission,
        )? {
            // Decided before the row is begun, so nothing is claimed.
            return self
                .leave_replaced_source(
                    demand,
                    disclosure,
                    schedule_progression.is_selected(),
                    request_admission,
                )
                .map(ExecutionPreparation::Reached);
        }
        if schedule_progression.is_selected() {
            // Validate the immutable selected input before changing registry state.
            entry::validate_progression_resources(
                &mut demand.resources,
                &mut demand.resources_validated,
                disclosed_value,
                entry,
                &demand.selected.identity,
                demand.limits,
            )?;
        }
        let interest = demand
            .interest
            .as_ref()
            .expect("progression entry checked live Interest");
        // The receipt carrier is admitted before Running can commit. Nothing
        // chargeable may strand a performed publication without its checkpoint.
        request_admission
            .charge_external_work(std::mem::size_of::<OwnPublication>() as u64 + 1)
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
        // One selected finish is prepaid before either initial begin or the
        // subsequent Scheduled begin can enter Running. The actual outcome
        // consumes it; dropping an unused token never releases a peer's row.
        let selected_execution_finish = self
            .output_demands
            .prepare_selected_execution_finish(interest, request_admission)?;
        let mut selected_published_mode = if schedule_progression.is_selected() {
            // Execute moves the invocation-local matched identity out of its
            // selected schedule slot after Running. Admit both the vacant
            // slot and returned carrier before the first begin.
            let work = std::mem::size_of::<WorthQueryProducerCommitAuthority>()
                .checked_add(3)
                .and_then(|work| {
                    std::mem::size_of::<Option<MatchedRequiredPredecessors<'_>>>()
                        .checked_mul(2)
                        .and_then(|move_work| work.checked_add(move_work))
                })
                .and_then(|work| u64::try_from(work).ok())
                .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
            request_admission
                .charge_external_work(work)
                .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
            Some(commit_authority.clone())
        } else {
            None
        };
        use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandAdvanceAdmission as Admission;
        let (mut admission_phase, mut selected_finish) = if schedule_progression.is_selected() {
            self.output_demands
                .begin_admitted(interest, request_admission)?
        } else {
            (self.output_demands.begin(interest), None)
        };
        let successor_of = loop {
            match admission_phase {
                Admission::Schedule(performed_source) => {
                    let mut result = match &schedule_progression {
                        ScheduleProgression::Ordinary => self.schedule_selected_output_producer(
                            phase,
                            &demand.selected,
                            delivery_branch,
                            &demand.observed_source,
                            performed_source.as_ref(),
                        ),
                        ScheduleProgression::Selected { shared, .. } => self
                            .schedule_selected_output_producer_on_selected(
                                phase,
                                &demand.selected,
                                delivery_branch,
                                &demand.observed_source,
                                performed_source.as_ref(),
                                shared,
                                request_admission,
                            ),
                    };
                    if let Some(finish) = selected_finish.take() {
                        finish.finish(performed_source, &mut result);
                    } else {
                        self.output_demands.finish_scheduling(
                            interest,
                            performed_source,
                            &mut result,
                        );
                    }
                    match result {
                    // The scheduled row is this call's to execute.
                    Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult::Scheduled) => {
                        admission_phase = if schedule_progression.is_selected() {
                            self.output_demands
                                .begin_scheduled_admitted(interest, request_admission)?
                        } else {
                            self.output_demands.begin(interest)
                        };
                        continue;
                    }
                    Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult::Deferred) => {
                        return Ok(ExecutionPreparation::Reached(waiting()));
                    }
                    Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult::NoEffect(denial))
                    | Err(denial) => return Err(denial),
                }
                }
                Admission::Pending => return Ok(ExecutionPreparation::Reached(waiting())),
                Admission::AdvanceCheckpoint { claim, checkpoint } => {
                    return Ok(ExecutionPreparation::Reached(
                        if self.advance_output_checkpoint(
                            phase,
                            interest,
                            &demand.selected.identity,
                            claim,
                            checkpoint,
                        )? {
                            OwnStages::Checkpoint(CheckpointProgress::Advanced)
                        } else {
                            waiting()
                        },
                    ));
                }
                Admission::Ready(completion) => {
                    return self
                        .advance_validated_ready::<Family>(
                            phase,
                            demand,
                            disclosure,
                            completion,
                            if schedule_progression.is_selected() {
                                ReadyCertification::SelectedWave
                            } else {
                                ReadyCertification::OnBranch(delivery_branch)
                            },
                            request_admission,
                        )
                        .map(ExecutionPreparation::Reached);
                }
                Admission::Failed(denial) => return Err(denial),
                Admission::Execute { successor_of } => break successor_of,
            }
        };
        let selected_execution_finish = crate::domain_computation::primary_graph::application_output_demand::RunningSelectedExecution::new(selected_execution_finish);
        let ordinary_readiness = if schedule_progression.is_selected() {
            None
        } else {
            let readiness = disclosed_source
                .output_source_epoch()
                .ok_or_else(|| {
                    denial(
                        WorthQueryOutputDemandDenialKind::ForeignSource,
                        Family::IDENTITY,
                    )
                })
                .and_then(|source| {
                    performed.borrow_mut().ordinary(
                        interest.key(),
                        source,
                        self,
                        delivery_branch,
                        request_admission,
                    )
                });
            match readiness {
                Ok(Some(readiness)) => Some(readiness),
                Ok(None) => {
                    selected_execution_finish.relinquish();
                    return Ok(ExecutionPreparation::Reached(waiting()));
                }
                Err(stop) => {
                    selected_execution_finish.relinquish();
                    return Err(stop);
                }
            }
        };
        let mut required_output = match interest.required_context(
            &self.primary_provider.graph.source_owner.invalidation_owner,
            request_admission,
        ) {
            Ok(context) => context,
            Err(denial) => {
                selected_execution_finish.relinquish();
                return Err(denial);
            }
        };
        if !schedule_progression.is_selected() {
            entry::validate_resources_after_begin(
                &self.output_demands,
                interest,
                &mut demand.resources,
                &mut demand.resources_validated,
                disclosed_value,
                entry,
                &demand.selected.identity,
                demand.limits,
            )?;
        }
        let published_mode = if let Some(mode) = selected_published_mode.take() {
            mode
        } else {
            entry::clone_published_mode(
                &self.output_demands,
                interest,
                &commit_authority,
                false,
                Family::IDENTITY,
                request_admission,
            )?
        };
        if demand.selected.reuses_live_output_only {
            required_output.reuse_live_output_only();
        }
        let required_execution = required_output.prepare_execution(published_mode);
        let retained_resources = demand.resources;
        let runtime_identity = self.runtime.authority_identity().as_u64();
        let root = demand.observed_source.source_root();
        let finish = move |result, request_admission: &mut InvalidationEditAdmission, readiness| {
            super::execution_completion::finish(
                result,
                selected_execution_finish,
                retained_resources,
                &self.primary_provider.application_attempt_operation_control,
                request_admission,
                performed,
                readiness,
                runtime_identity,
                root,
            )
        };
        let result = disclosure.with_erased(|input| match schedule_progression {
            ScheduleProgression::Ordinary => {
                let result = entry.executor.execute(
                    phase,
                    self,
                    principal,
                    request_scope,
                    delivery_branch,
                    required_execution,
                    input,
                    successor_of,
                    commit_authority,
                    entry.edition,
                    demand.limits,
                    request_admission,
                    &mut demand.producer_contacts_in_this_demand,
                );
                match finish(
                    result,
                    request_admission,
                    ordinary_readiness.expect("ordinary effect has publication readiness"),
                ) {
                    Ok(progress) => Ok(ExecutionPreparation::Reached(OwnStages::Checkpoint(
                        progress,
                    ))),
                    Err(refusal) => {
                        demand.unpublished_selected_checkpoint = refusal.checkpoint;
                        Err(execution::ProducerExecutionStop::ExecutionStopped(
                            refusal.denial,
                        ))
                    }
                }
            }
            ScheduleProgression::Selected {
                shared,
                matched_predecessors,
                readiness,
            } => {
                let prepared = entry.executor.execute_on_selected(
                    phase,
                    self,
                    principal,
                    request_scope,
                    shared,
                    matched_predecessors,
                    required_execution,
                    input,
                    successor_of,
                    commit_authority,
                    entry.edition,
                    demand.limits,
                    request_admission,
                    &mut demand.producer_contacts_in_this_demand,
                );
                // A preparation refusal still owns this Running invocation.
                // Canonical completion applies its normal failure protocol;
                // abandonment alone cannot substitute for that typed outcome.
                let plan = match prepared {
                    Ok(plan) => plan,
                    Err(stop) => OwnerPlan::immediate(Err(stop)),
                };
                Ok(ExecutionPreparation::Planned(plan.map_result(
                    move |result, admission| finish(result, admission, readiness),
                )))
            }
        });
        result.map_err(|stop| match stop {
            execution::ProducerExecutionStop::RequestAdmissionDenied(rejection) => {
                rejection.into_denial()
            }
            execution::ProducerExecutionStop::ExecutionStopped(denial) => denial,
            execution::ProducerExecutionStop::LiveOutputNotReused { producer, reason } => {
                execution::ProducerExecutionStop::live_output_not_reused(producer, reason)
            }
        })
    }
}
