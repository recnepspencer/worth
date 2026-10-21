//! Suspend an admitted selected row until canonical owner application.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::execution;
use crate::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::OwnerPlan;

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
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "advancement phase",
                )
            })?;
        let waiting = || OwnStages::Answer(WorthQueryOutputDemandAdvance::Pending);
        let disclosed_source = disclosure.source();
        if let SourceGuard::Replaced = self.prepare_progression_entry(
            demand,
            disclosed_source,
            &commit_authority,
            entry,
            &schedule_progression,
        )? {
            // Decided before the row is begun, so nothing is claimed.
            return self
                .leave_replaced_source(
                    &execution,
                    demand,
                    disclosure,
                    schedule_progression.is_selected(),
                )
                .map(ExecutionPreparation::Reached);
        }
        if !schedule_progression.is_selected()
            && self.released_predecessor_needs_source(demand, disclosed_source)?
        {
            self.refresh_output_demand(
                &execution,
                demand,
                disclosure.value(),
                disclosed_source.clone(),
                None,
            )?;
            return Ok(ExecutionPreparation::Reached(OwnStages::Refreshed(
                disclosure,
            )));
        }
        let interest = demand
            .interest
            .as_ref()
            .expect("progression entry checked live Interest");
        // The receipt carrier is admitted before Running can commit. Nothing
        // chargeable may strand a performed publication without its checkpoint.

        // One selected finish is prepaid before either initial begin or the
        // subsequent Scheduled begin can enter Running. The actual outcome
        // consumes it; dropping an unused token never releases a peer's row.
        let selected_execution_finish = self
            .output_demands
            .prepare_selected_execution_finish(interest)?;
        let mut selected_published_mode = if schedule_progression.is_selected() {
            // Execute moves the invocation-local matched identity out of its
            // selected schedule slot after Running. Admit both the vacant
            // slot and returned carrier before the first begin.

            Some(commit_authority.clone())
        } else {
            None
        };
        use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandAdvanceAdmission as Admission;
        let (mut admission_phase, mut selected_finish) = if schedule_progression.is_selected() {
            self.output_demands.begin_admitted(interest)?
        } else {
            (self.output_demands.begin(interest), None)
        };
        let successor_of = loop {
            match admission_phase {
                Admission::Schedule(performed_source) => {
                    let mut result = match &schedule_progression {
                        ScheduleProgression::Ordinary => self.schedule_selected_output_producer(
                            &execution,
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
                            ),
                    };
                    match selected_finish.take() {
                        Some(finish) => {
                            finish.finish(performed_source, &mut result);
                        }
                        _ => {
                            self.output_demands.finish_scheduling(
                                interest,
                                performed_source,
                                &mut result,
                            );
                        }
                    }
                    match result {
                    // The scheduled row is this call's to execute.
                    Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult::Scheduled) => {
                        admission_phase = if schedule_progression.is_selected() {
                            self.output_demands
                                .begin_scheduled_admitted(interest)?
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
                            &execution,
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
                            &execution,
                            phase,
                            demand,
                            disclosure,
                            completion,
                            if schedule_progression.is_selected() {
                                ReadyCertification::SelectedWave
                            } else {
                                ReadyCertification::OnBranch(delivery_branch)
                            },
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
                        &execution,
                        interest.key(),
                        source,
                        self,
                        delivery_branch,
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
            Family::IDENTITY,
            &self.primary_provider.graph.source_owner.invalidation_owner,
        ) {
            Ok(context) => context,
            Err(denial) => {
                selected_execution_finish.relinquish();
                return Err(denial);
            }
        };
        let published_mode = if let Some(mode) = selected_published_mode.take() {
            mode
        } else {
            commit_authority.clone()
        };
        if demand.selected.reuses_live_output_only {
            required_output.reuse_live_output_only();
        }
        let required_execution = required_output.prepare_execution(published_mode);
        let runtime_identity = self.runtime.authority_identity().as_u64();
        let root = demand.observed_source.source_root();
        let finish = move |result, readiness| {
            super::execution_completion::finish(
                result,
                selected_execution_finish,
                performed,
                readiness,
                runtime_identity,
                root,
            )
        };
        let prepared_input = demand
            .prepare_disclosed_input(disclosure)
            .map_err(execution::ProducerExecutionStop::ExecutionStopped);
        let result = match schedule_progression {
            ScheduleProgression::Ordinary => {
                let result = prepared_input.and_then(|proof| {
                    proof.with_erased(|input| {
                        entry.executor.execute(
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
                            &mut demand.producer_contacts_in_this_demand,
                        )
                    })
                });
                match finish(
                    result,
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
                let prepared = prepared_input.and_then(|proof| {
                    proof.with_erased(|input| {
                        entry.executor.execute_on_selected(
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
                            &mut demand.producer_contacts_in_this_demand,
                        )
                    })
                });
                // A preparation refusal still owns this Running invocation.
                // Canonical completion applies its normal failure protocol;
                // abandonment alone cannot substitute for that typed outcome.
                let plan = match prepared {
                    Ok(plan) => plan,
                    Err(stop) => OwnerPlan::immediate(Err(stop)),
                };
                Ok(ExecutionPreparation::Planned(
                    plan.map_result(move |result| finish(result, readiness)),
                ))
            }
        };
        result.map_err(execution::ProducerExecutionStop::into_output_denial)
    }
}
