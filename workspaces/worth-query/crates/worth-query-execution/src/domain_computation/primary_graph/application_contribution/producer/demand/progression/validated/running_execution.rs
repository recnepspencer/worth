//! Consume an admitted Running phase after its preparation frame has unwound.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::execution;
use crate::domain_computation::primary_graph::application_contribution::producer::execution::prepared_plan::OwnerPlan;
impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[inline(never)]
    pub(super) fn execute_running_output<'owner, Family>(
        &'owner self,
        phase: &'owner WorthQueryAdvancementPhase<'owner>,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        entry: &InstalledProducerProvider<Schema>,
        schedule_progression: ScheduleProgression<'owner, '_, '_, Schema>,

        performed: &'owner std::cell::RefCell<PerformedMembers>,
        running: RunningOutputPreparation<
            'owner,
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
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
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "advancement phase",
                )
            })?;
        let RunningOutputPreparation {
            selected_execution_finish,
            ordinary_readiness,
            required_execution,
            disclosure,
            successor_of,
            commit_authority,
        } = running;
        let runtime_identity = self.runtime.authority_identity().as_u64();
        let root = demand.observed_source.source_root();
        let original_publication = demand.original_producing_publication.clone();
        let finish = move |result, readiness| {
            super::execution_completion::finish(
                result,
                selected_execution_finish,
                performed,
                readiness,
                runtime_identity,
                root,
                original_publication,
                &execution,
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
