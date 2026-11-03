//! Suspend an admitted selected row until canonical owner application.
use super::*;

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[inline(never)]
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn prepare_running_output_with_schedule<
        'owner,
        Family,
    >(
        &'owner self,
        phase: &'owner WorthQueryAdvancementPhase<'owner>,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        entry: &InstalledProducerProvider<Schema>,
        commit_authority: WorthQueryProducerCommitAuthority,
        schedule_progression: &ScheduleProgression<'owner, '_, '_, Schema>,

        performed: &'owner std::cell::RefCell<PerformedMembers>,
    ) -> Result<
        OutputPreparation<
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
                .map(OutputPreparation::Reached);
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
            return Ok(OutputPreparation::Reached(OwnStages::Refreshed(disclosure)));
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
                        return Ok(OutputPreparation::Reached(waiting()));
                    }
                    Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult::NoEffect(denial))
                    | Err(denial) => return Err(denial),
                }
                }
                Admission::Pending => return Ok(OutputPreparation::Reached(waiting())),
                Admission::AdvanceCheckpoint { claim, checkpoint } => {
                    return Ok(OutputPreparation::Reached(
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
                        .map(OutputPreparation::Reached);
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
                    return Ok(OutputPreparation::Reached(waiting()));
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
        Ok(OutputPreparation::Running(RunningOutputPreparation {
            selected_execution_finish,
            ordinary_readiness,
            required_execution,
            disclosure,
            successor_of,
            commit_authority,
        }))
    }
}

/// Preparation has either reached an existing row or issued one Running phase.
pub(super) enum OutputPreparation<'owner, Query, Value> {
    Reached(OwnStages<Query, Value>),
    Running(RunningOutputPreparation<'owner, Query, Value>),
}

/// Moves the real prepaid row and source into execution without another allocation.
pub(super) struct RunningOutputPreparation<'owner, Query, Value> {
    pub(super) selected_execution_finish: crate::domain_computation::primary_graph::application_output_demand::RunningSelectedExecution<'owner>,
    pub(super) ordinary_readiness: Option<super::super::required_wave::performed::PublicationReadiness>,
    pub(super) required_execution: crate::domain_computation::primary_graph::application_output_demand::RequiredOutputExecution,
    pub(super) disclosure: ValidatedOutputDisclosure<Query, Value>,
    pub(super) successor_of: Option<[u8; 32]>,
    pub(super) commit_authority: WorthQueryProducerCommitAuthority,
}
