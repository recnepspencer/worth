use super::super::super::registry::InstalledProducerProvider;
use super::super::disclosure::ValidatedOutputDisclosure;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
mod entry;
mod ready;
use ready::ReadyCertification;
mod schedule_progression;
mod source_guard;
pub(super) use schedule_progression::OwnPublication;
use schedule_progression::{CheckpointProgress, OwnStages, ScheduleProgression};
use source_guard::SourceGuard;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    fn advance_validated_output_demand_with_schedule<Family>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,

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
        mut schedule_progression: ScheduleProgression<'_, '_, Schema>,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<
        OwnStages<FamilySourceQuery<Schema, Family>, FamilySourceValue<Schema, Family>>,
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
            return self.leave_replaced_source(
                demand,
                disclosure,
                schedule_progression.is_selected(),
                request_admission,
            );
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
                        return Ok(waiting());
                    }
                    Ok(crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputSchedulingResult::NoEffect(denial))
                    | Err(denial) => return Err(denial),
                }
                }
                Admission::Pending => return Ok(waiting()),
                Admission::AdvanceCheckpoint { claim, checkpoint } => {
                    return Ok(
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
                    );
                }
                Admission::Ready(completion) => {
                    return self.advance_validated_ready::<Family>(
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
                    );
                }
                Admission::Failed(denial) => return Err(denial),
                Admission::Execute { successor_of } => break successor_of,
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
        let result = disclosure.with_erased(|input| match &mut schedule_progression {
            ScheduleProgression::Ordinary => entry.executor.execute(
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
            ),
            ScheduleProgression::Selected {
                shared,
                matched_predecessors,
            } => entry.executor.execute_on_selected(
                phase,
                self,
                principal,
                request_scope,
                shared,
                matched_predecessors.take(),
                required_execution,
                input,
                successor_of,
                commit_authority,
                entry.edition,
                demand.limits,
                request_admission,
                &mut demand.producer_contacts_in_this_demand,
            ),
        });
        let mut receipt = match result.map(|prepared| prepared.into_parts()) {
            Ok((
                super::super::super::execution::ProducerExecutionOutcome::Committed(receipt),
                ready_backing,
            )) => (receipt, ready_backing),
            Ok((
                super::super::super::execution::ProducerExecutionOutcome::Stable(stable),
                ready_backing,
            )) => {
                let completion = ready_backing.complete(
                    crate::domain_computation::primary_graph::application_output_demand::WorthQueryCompletedOutputDemand {
                        authority: crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputAuthority::Stable(stable),
                        readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::without_execution(),
                        resources: demand.resources,
                    },
                );
                let checkpoint = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint::Ready(completion);
                if let Err((denial, checkpoint)) = selected_execution_finish.publish(checkpoint) {
                    demand.unpublished_selected_checkpoint = Some(checkpoint);
                    return Err(denial);
                }
                return Ok(OwnStages::Checkpoint(CheckpointProgress::Advanced));
            }
            Err(super::super::super::execution::ProducerExecutionStop::RequestAdmissionDenied(
                denial,
            )) => {
                selected_execution_finish.relinquish();
                return Err(denial.into_denial());
            }
            Err(super::super::super::execution::ProducerExecutionStop::LiveOutputNotReused {
                producer,
                reason,
            }) => {
                selected_execution_finish.relinquish();
                return Err(
                    super::super::super::execution::ProducerExecutionStop::live_output_not_reused(
                        producer, reason,
                    ),
                );
            }
            Err(super::super::super::execution::ProducerExecutionStop::ExecutionStopped(
                mut denial,
            )) => {
                selected_execution_finish.failure(&mut denial);
                return Err(denial);
            }
        };
        let publication = OwnPublication(receipt.0.committed_product_publication().clone());
        #[cfg(feature = "test-query-execution-observer")]
        super::caller_pass_observation::exhaust_after_commit(request_admission);
        let delivery = receipt.0
            .take_performed_relational_product_change()
            .map_or(
                crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery::NoChange,
                crate::domain_computation::primary_graph::application_output_demand::WorthQueryPendingOutputDelivery::Change,
            );
        let checkpoint = crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputCheckpoint::Published {
            receipt: receipt.0,
            delivery,
            ready_backing: receipt.1,
        };
        if let Err((denial, checkpoint)) = selected_execution_finish.publish(checkpoint) {
            demand.unpublished_selected_checkpoint = Some(checkpoint);
            return Err(denial);
        }
        Ok(OwnStages::Checkpoint(CheckpointProgress::Published(
            publication,
        )))
    }
}
