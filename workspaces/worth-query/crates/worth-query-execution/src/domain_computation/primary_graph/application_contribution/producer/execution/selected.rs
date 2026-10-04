//! Installed producer execution on the required wave's one selected Product.

use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::producer::demand::{
        MatchedRequiredPredecessors, ResolvedRequiredPredecessors,
    },
    application_output_demand::{RetainedOutputReadmissionSource, SelectedReadyReadmission},
    output_lineage::AcceptedCurrentCandidate,
};
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

/// The selected Execute entry is a separate object-safe installed executor
/// obligation. Its owner implementation can use the retained mutation binding
/// and cold graph template without changing ordinary producer execution.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) trait InstalledSelectedProducerExecutor<
    Schema,
>:
    Send + Sync
{
    #[allow(clippy::too_many_arguments)]
    fn advance_ready_on_selected<'basis>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        shared: &'basis crate::domain_computation::primary_graph::SharedSelectedProductOperation<
            '_,
            Schema,
        >,
        positioned: &'basis PositionedRelationalSnapshot,
        selected: &SelectedReadyReadmission,
        candidate: Option<&'basis AcceptedCurrentCandidate>,
        resolved: Option<&ResolvedRequiredPredecessors<'_, Schema>>,
        installed: &super::super::registry::InstalledProducerProvider<Schema>,
        delivery_branch: WorthQueryProductBranch,
        producer_contacts_in_this_demand: usize,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<super::required_cue::RequiredCueProgress<'basis, Schema>, ProducerExecutionStop>
    where
        Schema: ApplicationSchema;

    #[allow(clippy::too_many_arguments)]
    fn execute_on_selected(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        shared: &crate::domain_computation::primary_graph::SharedSelectedProductOperation<
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
    ) -> Result<PreparedProducerExecutionOutcome, ProducerExecutionStop>;

    fn preserved_readiness_output_admitted(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial>;
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
                    Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
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
            .admit_selected_role_lookup(Binding::OUTPUT_ROLE, admission)
            .map_err(denied)?;
        Ok(self.preserved_readiness_output(receipt))
    }

    #[allow(clippy::too_many_arguments)]
    fn advance_ready_on_selected<'basis>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        shared: &'basis crate::domain_computation::primary_graph::SharedSelectedProductOperation<
            '_,
            Schema,
        >,
        positioned: &'basis PositionedRelationalSnapshot,
        selected: &SelectedReadyReadmission,
        candidate: Option<&'basis AcceptedCurrentCandidate>,
        resolved: Option<&ResolvedRequiredPredecessors<'_, Schema>>,
        installed: &super::super::registry::InstalledProducerProvider<Schema>,
        delivery_branch: WorthQueryProductBranch,
        producer_contacts_in_this_demand: usize,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<super::required_cue::RequiredCueProgress<'basis, Schema>, ProducerExecutionStop>
    {
        use super::required_cue::RequiredCueProgress;
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
        source_readmission::certify_ready_on_selected_with_disclosure::<Schema, Binding>(
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
                let progress = runtime
                    .continue_required_fresh::<Binding::OutputFamily>(
                        fresh,
                        shared,
                        claim,
                        installed,
                        principal,
                        request_scope,
                        delivery_branch,
                        matched_predecessors,
                        admission,
                    )
                    .map_err(ProducerExecutionStop::ExecutionStopped)?;
                Ok(RequiredCueProgress::Fresh(progress))
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_on_selected(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        shared: &crate::domain_computation::primary_graph::SharedSelectedProductOperation<
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
    ) -> Result<PreparedProducerExecutionOutcome, ProducerExecutionStop> {
        use super::super::demand::disclosure::FreshDisclosureAdmissionStop;
        // Fund the typed edition/type checks and the move-only execution split
        // before any early refusal. Producer subjects are borrowed statics.
        let entry_work = u64::try_from(
            std::mem::size_of::<RequiredOutputExecution>()
                .checked_add(std::mem::size_of::<super::super::InstalledProducerEdition>() * 2)
                .and_then(|work| work.checked_add(std::mem::size_of::<std::any::TypeId>() * 8))
                .and_then(|work| work.checked_add(8))
                .ok_or_else(|| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?,
        )
        .map_err(|_| denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, ""))?;
        request_admission
            .charge_external_work(entry_work)
            .map_err(|stop| source_readmission::ready_resource_denial("", stop))?;
        let (required_output, ready_backing) = required_output.into_parts();
        if !edition.admits_binding::<Schema, Binding>() {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Binding::IDENTITY,
            )
            .into());
        }
        let proof = input
            .take::<SourceQuery<Schema, Binding>, SourceValue<Schema, Binding>>(edition)
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    Binding::IDENTITY,
                )
            })?;
        let ValidatedOutputDisclosure::Fresh(proof) = proof else {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                Binding::IDENTITY,
            )
            .into());
        };
        let admits = proof
            .admits_selected_admitted(
                principal,
                request_scope,
                shared.selected().product().observation(),
                request_admission,
            )
            .map_err(|stop| match stop {
                FreshDisclosureAdmissionStop::Admission(stop) => {
                    source_readmission::ready_resource_denial("", stop)
                }
                FreshDisclosureAdmissionStop::AccountingOverflow => {
                    denial(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "").into()
                }
            })?;
        if !admits {
            return Err(denial(
                WorthQueryOutputDemandDenialKind::Superseded,
                Binding::IDENTITY,
            )
            .into());
        }
        let (source, observed_source) = proof.into_parts();
        let resources = self.provider.demand_resources(&source);
        super::super::demand::validate_retained_resources(resources, Binding::IDENTITY, limits)?;
        let input = self.provider.operation_input(&source);
        let prepared = super::selected_public::prepare_selected_public_mutation::<Schema, Binding>(
            self,
            runtime,
            shared,
            principal,
            request_scope,
            &input,
            request_admission,
        )
        .map_err(|stop| super::selected_public::preparation_stop::<Schema, Binding>(stop))?;
        let outcome = super::post_authorization::execute_authorized::<Schema, Binding>(
            runtime,
            shared.selected(),
            prepared.principal.principal(),
            prepared.operation,
            required_output,
            self.provider.as_ref(),
            source,
            observed_source,
            input,
            matched_predecessors,
            resources,
            successor_of,
            commit_authority,
            edition,
            limits,
            request_admission,
            producer_contacts,
        )?;
        Ok(PreparedProducerExecutionOutcome::new(
            outcome,
            ready_backing,
        ))
    }
}
