//! Installed producer execution on the required wave's one selected Product.

use super::super::demand::{DecisionInput, PerformedMembers, SelectedDecisionInput};
use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::{
    application_contribution::producer::demand::{
        MatchedRequiredPredecessors, ResolvedRequiredPredecessors,
    },
    application_output_demand::{RetainedOutputReadmissionSource, SelectedReadyReadmission},
    output_lineage::AcceptedCurrentCandidate,
};
use worth_relational::facade::runtime::PositionedRelationalSnapshot;
mod execution_plan;

mod contract;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use contract::InstalledSelectedProducerExecutor;

impl<Schema, Binding> InstalledSelectedProducerExecutor<Schema>
    for TypedInstalledProducer<Schema, Binding>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    Binding::Provider: WorthQueryApplicationProducerProvider<Schema, Binding>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                Schema,
                SourceQuery<Schema, Binding>,
            > + 'static,
    SourceQuery<Schema, Binding>: 'static,
{
    fn preserved_readiness_output_admitted(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        fn denied(
            stop: worth_relational::facade::mvcc::CompanionPreflightStop,
        ) -> WorthQueryOutputDemandDenial {
            use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
            WorthQueryOutputDemandDenial::new(
                match stop {
                    Stop::WorkExhausted { .. }
                    | Stop::WorkExhaustedWithoutRequirement { .. }
                    | Stop::WorkCounterOverflow => {
                        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
                    }
                    _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
                },
                "",
            )
        }
        admission.charge_external_work(1).map_err(denied)?;
        receipt
            .output_correspondence()
            .admit_selected_role_lookup(
                <Binding::OutputRole as WorthQueryApplicationOutputRole>::NAME,
                admission,
            )
            .map_err(denied)?;
        Ok(self.preserved_readiness_output(receipt))
    }

    #[allow(clippy::too_many_arguments)]
    fn advance_ready_on_selected<'basis, 'owner>(
        &self,
        phase: &'owner WorthQueryAdvancementPhase<'owner>,

        runtime: &'owner WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        shared: &'owner crate::domain_computation::primary_graph::SharedSelectedProductOperation<
            'owner,
            Schema,
        >,
        positioned: &'basis PositionedRelationalSnapshot,
        selected: &SelectedReadyReadmission,
        candidate: Option<&'basis AcceptedCurrentCandidate>,
        resolved: Option<&ResolvedRequiredPredecessors<'_, Schema>>,
        installed: &super::super::registry::InstalledProducerProvider<Schema>,
        delivery_branch: WorthQueryProductBranch,
        producer_contacts_in_this_demand: usize,
        refresh_permission: Result<(), WorthQueryOutputDemandDenial>,
        performed: &'owner std::cell::RefCell<PerformedMembers>,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<
        super::required_cue::RequiredCueProgress<'basis, 'owner, Schema>,
        ProducerExecutionStop,
    >
    where
        'owner: 'basis,
    {
        use super::required_cue::RequiredCueProgress;
        let execution = phase
            .execution_request_for(&runtime.product_runtime)
            .map_err(|cause| {
                denial(
                    WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                    "producer advancement phase",
                )
            })?;

        // The registry selects this entry by the exact Ready producer, and
        // cold registration seals its sole typed executor to that entry.
        request_admission
            .charge_external_work(4)
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
        if let Some(candidate) = candidate {
            let same_ready = candidate
                .matches_ready_admitted(selected.completion(), request_admission)
                .map_err(|stop| source_readmission::ready_resource_denial("", stop))?;
            if !same_ready {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSettlement,
                    Binding::IDENTITY,
                )
                .into());
            }
        }
        let retained = selected
            .source()
            .downcast_ref::<RetainedOutputReadmissionSource<SourceQuery<Schema, Binding>>>()
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    Binding::IDENTITY,
                )
            })?
            .source();
        source_readmission::certify_ready_on_selected_with_disclosure::<_, Schema, Binding>(
            &execution,
            runtime,
            &self.source_query,
            &self.principal_validation,
            principal,
            request_scope,
            shared,
            positioned,
            retained,
            candidate,
            resolved,
            selected.limits(),
            producer_contacts_in_this_demand,
            request_admission,
            |permission, matched_predecessors, admission| {
                refresh_permission.map_err(ProducerExecutionStop::ExecutionStopped)?;
                admission
                    .charge_external_work(
                        std::mem::size_of::<super::super::InstalledProducerEdition>() as u64,
                    )
                    .map_err(|stop| source_readmission::ready_resource_denial("", stop))?;
                let claim = runtime
                    .output_demands
                    .claim_required_ready_refresh(selected, admission)
                    .map_err(ProducerExecutionStop::ExecutionStopped)?
                    // The selected Ready moved before its refresh was claimed:
                    // a later wave selects the row as it is now.
                    .ok_or_else(|| {
                        WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::PublicationStale,
                            Binding::IDENTITY,
                        )
                        .with_recovery_posture(
                            crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable,
                        )
                    })?;
                let fresh = source_readmission::disclose_prepared_on_selected::<Schema, Binding>(
                    &execution,
                    runtime,
                    permission,
                    principal,
                    request_scope,
                    delivery_branch,
                    retained,
                    shared,
                    installed.edition,
                    admission,
                )?;
                let Some(readiness) = performed.borrow_mut().fresh(
                    &execution,
                    selected.key(),
                    fresh.source().output_source_epoch().ok_or_else(|| {
                        denial(
                            WorthQueryOutputDemandDenialKind::ForeignSource,
                            Binding::IDENTITY,
                        )
                    })?,
                    &DecisionInput::Selected(&SelectedDecisionInput {
                        shared,
                        positioned,
                        runtime,
                    }),
                    admission,
                )?
                else {
                    return Ok(RequiredCueProgress::PendingUnresolved);
                };
                let progress = runtime
                    .prepare_required_fresh::<Binding::OutputFamily>(
                        execution,
                        phase,
                        fresh,
                        readiness,
                        shared,
                        claim,
                        installed,
                        principal,
                        request_scope,
                        delivery_branch,
                        matched_predecessors,
                        admission,
                        performed,
                    )
                    .map_err(ProducerExecutionStop::ExecutionStopped)?;
                Ok(RequiredCueProgress::Fresh(progress))
            },
        )
    }

    fn inspect_frozen_ready<'basis>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        inspection: super::required_cue::FrozenInspection<'basis, '_, Schema>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<super::required_cue::FrozenCurrentInspection<'basis>, ProducerExecutionStop> {
        let super::required_cue::FrozenInspection {
            runtime,
            principal,
            request: request_scope,
            shared,
            positioned,
            selected,
            candidate,
            contacts: producer_contacts,
        } = inspection;
        admission
            .charge_external_work(4)
            .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
        let same_ready = candidate
            .map(|candidate| {
                candidate
                    .matches_ready_admitted(selected.completion(), admission)
                    .map_err(|stop| source_readmission::ready_resource_denial("", stop))
            })
            .transpose()?
            .unwrap_or(true);
        if !same_ready {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                Binding::IDENTITY,
            )
            .into());
        }
        let retained = selected
            .source()
            .downcast_ref::<RetainedOutputReadmissionSource<SourceQuery<Schema, Binding>>>()
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    Binding::IDENTITY,
                )
            })?
            .source();
        // The same native certification as Live. Its disclosure continuation
        // only reports not-current: no refresh claim, source read or plan.
        source_readmission::certify_ready_on_selected_with_disclosure::<_, Schema, Binding>(
            execution,
            runtime,
            &self.source_query,
            &self.principal_validation,
            principal,
            request_scope,
            shared,
            positioned,
            retained,
            candidate,
            None,
            selected.limits(),
            producer_contacts,
            admission,
            |_, _, _| Ok(super::required_cue::FrozenCurrentInspection::NotCurrent),
        )
    }

    fn execute_on_selected<'owner>(
        &self,
        phase: &'owner WorthQueryAdvancementPhase<'owner>,

        runtime: &'owner WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        shared: &'owner crate::domain_computation::primary_graph::SharedSelectedProductOperation<
            '_,
            Schema,
        >,
        matched_predecessors: Option<MatchedRequiredPredecessors<'_>>,
        required_output: RequiredOutputExecution,
        input: ValidatedProducerInput<'_>,
        successor_of: Option<[u8; 32]>,
        commit_authority: WorthQueryProducerCommitAuthority,
        edition: super::super::InstalledProducerEdition,
        limits: WorthQueryOutputDemandLimits,
        request_admission: &mut InvalidationEditAdmission,
        producer_contacts: &mut usize,
    ) -> Result<super::outcome::PreparedProducerExecutionPlan<'owner>, ProducerExecutionStop> {
        execution_plan::prepare_selected_execution::<Schema, Binding>(
            self,
            phase,
            runtime,
            principal,
            request_scope,
            shared,
            matched_predecessors,
            required_output,
            input,
            successor_of,
            commit_authority,
            edition,
            limits,
            request_admission,
            producer_contacts,
        )
    }
}
