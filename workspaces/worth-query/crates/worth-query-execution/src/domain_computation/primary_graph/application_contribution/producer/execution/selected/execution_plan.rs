//! Lower the selected producer execution into its owner-held plan.
use super::*;
pub(super) fn prepare_selected_execution<'owner, Schema, Binding>(
    this: &TypedInstalledProducer<Schema, Binding>,
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
    edition: super::super::super::InstalledProducerEdition,
    limits: WorthQueryOutputDemandLimits,
    request_admission: &mut InvalidationEditAdmission,
    producer_contacts: &mut usize,
) -> Result<super::super::outcome::PreparedProducerExecutionPlan<'owner>, ProducerExecutionStop>
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
    use super::super::super::demand::disclosure::FreshDisclosureAdmissionStop;
    // Fund the typed edition/type checks and the move-only execution split
    // before any early refusal. Producer subjects are borrowed statics.
    let entry_work = u64::try_from(
        std::mem::size_of::<RequiredOutputExecution>()
            .checked_add(std::mem::size_of::<super::super::super::InstalledProducerEdition>() * 2)
            .and_then(|work| {
                work.checked_add(std::mem::size_of::<crate::equality_type_id::EqualityTypeId>() * 8)
            })
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
    // A disclosure affinity expires on interruption too. Preserve the
    // request's typed cause before testing the selected proof's identity.
    if let Some(interruption) = request_scope.interruption() {
        return Err(super::super::denial::request_admission_rejected(denial(
                match interruption {
                    worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => WorthQueryOutputDemandDenialKind::Cancelled,
                    worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => WorthQueryOutputDemandDenialKind::TimedOut,
                }, Binding::IDENTITY)));
    }
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
    let resources = this.provider.demand_resources(&source);
    super::super::super::demand::validate_retained_resources(resources, Binding::IDENTITY, limits)?;
    let input = this.provider.operation_input(&source);
    let prepared =
        super::super::selected_public::prepare_selected_public_mutation::<Schema, Binding>(
            this,
            runtime,
            shared,
            principal,
            request_scope,
            &input,
            request_admission,
        )
        .map_err(|stop| super::super::selected_public::preparation_stop::<Schema, Binding>(stop))?;
    let plan = super::super::post_authorization::prepare_authorized::<Schema, Binding>(
        phase,
        runtime,
        shared.selected(),
        prepared.principal.principal(),
        prepared.operation,
        required_output,
        std::sync::Arc::clone(&this.provider),
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
    Ok(plan.map_result(move |outcome, _| {
        outcome.map(|outcome| PreparedProducerExecutionOutcome::new(outcome, ready_backing))
    }))
}
