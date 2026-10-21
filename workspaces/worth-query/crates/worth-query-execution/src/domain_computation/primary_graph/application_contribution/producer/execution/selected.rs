//! Installed producer execution on the required wave's one selected Product.

use super::super::demand::DecisionInput;
use super::super::demand::PerformedMembers;
use super::super::demand::ProducerPublicationStale;
use super::super::demand::SelectedDecisionInput;
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::MatchedRequiredPredecessors;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::ResolvedRequiredPredecessors;
use crate::domain_computation::primary_graph::application_output_demand::RetainedOutputReadmissionSource;
use crate::domain_computation::primary_graph::application_output_demand::SelectedReadyReadmission;
use crate::domain_computation::primary_graph::output_lineage::AcceptedCurrentCandidate;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;
mod execution_plan;

/// The selected Execute entry is a separate object-safe installed executor
/// obligation. Its owner implementation can use the retained mutation binding
/// and cold graph template without changing ordinary producer execution.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) trait InstalledSelectedProducerExecutor<
    Schema,
>:
    Send + Sync
{
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
    ) -> Result<
        super::required_cue::RequiredCueProgress<'basis, 'owner, Schema>,
        ProducerExecutionStop,
    >
    where
        'owner: 'basis,
        Schema: ApplicationSchema;

    fn inspect_frozen_ready<'basis>(
        &self,
        inspection: super::required_cue::FrozenInspection<'basis, '_, Schema>,
    ) -> Result<super::required_cue::FrozenCurrentInspection<'basis>, ProducerExecutionStop>
    where
        Schema: ApplicationSchema;

    #[allow(clippy::too_many_arguments)]
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
        producer_contacts: &mut usize,
    ) -> Result<super::outcome::PreparedProducerExecutionPlan<'owner>, ProducerExecutionStop>;

    fn preserved_readiness_output(&self, receipt: &WorthQueryApplicationCommitReceipt) -> bool;
}

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
    fn preserved_readiness_output(&self, receipt: &WorthQueryApplicationCommitReceipt) -> bool {
        receipt
            .output_correspondence()
            .posture_for_binding_role::<Binding::Operation>(
                <Binding::OutputRole as WorthQueryApplicationOutputRole>::NAME,
            )
            == Ok(
                worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture::Preserve,
            )
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

        if let Some(candidate) = candidate {
            let same_ready = candidate
                .matches_ready_admitted(selected.completion())
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
            producer_contacts_in_this_demand,
            |permission, matched_predecessors| {
                refresh_permission.map_err(ProducerExecutionStop::ExecutionStopped)?;

                let claim = runtime
                    .output_demands
                    .claim_required_ready_refresh(selected)
                    .map_err(ProducerExecutionStop::ExecutionStopped)?
                    // A later wave must select the Ready row after this claim moved.
                    .map_err(|rejection| {
                        WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::PublicationStale,
                            Binding::IDENTITY,
                        )
                        .with_recovery_posture(
                            crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable,
                        )
                        .with_publication_stale(Some(ProducerPublicationStale::ready_refresh_claim_moved(
                            shared.selected().product().observation(), rejection,
                        )))
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
                )?
                else {
                    return Ok(RequiredCueProgress::PendingUnresolved);
                };
                let progress = runtime
                    .prepare_required_fresh::<Binding::OutputFamily>(
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
                        performed,
                    )
                    .map_err(ProducerExecutionStop::ExecutionStopped)?;
                Ok(RequiredCueProgress::Fresh(progress))
            },
        )
    }

    fn inspect_frozen_ready<'basis>(
        &self,
        inspection: super::required_cue::FrozenInspection<'basis, '_, Schema>,
    ) -> Result<super::required_cue::FrozenCurrentInspection<'basis>, ProducerExecutionStop> {
        let super::required_cue::FrozenInspection {
            execution,
            runtime,
            principal,
            request: request_scope,
            shared,
            positioned,
            selected,
            candidate,
            contacts: producer_contacts,
        } = inspection;

        let same_ready = candidate
            .map(|candidate| {
                candidate
                    .matches_ready_admitted(selected.completion())
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
            producer_contacts,
            |_, _| Ok(super::required_cue::FrozenCurrentInspection::NotCurrent),
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
            producer_contacts,
        )
    }
}
