//! Prepare a producer once and retain owner completion until canonical apply.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;
#[allow(clippy::too_many_arguments)]
pub(in crate::domain_computation::primary_graph::application_contribution::producer::execution) fn prepare_authorized<
    'owner,
    Schema,
    Binding,
>(
    phase: &'owner WorthQueryAdvancementPhase<'owner>,
    runtime: &'owner WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    shared: &'owner crate::domain_computation::primary_graph::SharedSelectedProductOperation<
        '_,
        Schema,
    >,
    control: Option<
        &crate::domain_computation::primary_graph::SharedSelectedProductOperation<'_, Schema>,
    >,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        Schema,
        <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Principal,
        <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
    >,
    mut admission: crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
        Schema,
        <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Operation,
        <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Input,
        MutationScope<Schema, Binding>,
    >,
    required_output: RequiredOutputDemandContext,
    provider: std::sync::Arc<Binding::Provider>,
    source: SourceValue<Schema, Binding>,
    observed_source: crate::domain_computation::primary_graph::WorthQueryObservedSource<
        SourceQuery<Schema, Binding>,
    >,
    input: <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Input,
    matched_predecessors: Option<super::super::super::demand::MatchedRequiredPredecessors<'_>>,
    successor_of: Option<[u8; 32]>,
    commit_authority: ProducerInvocationAuthority<'owner>,
    edition: super::super::super::InstalledProducerEdition,
    producer_contacts: &mut usize,
) -> Result<super::super::prepared_plan::ProducerExecutionPlan<'owner>, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, Binding>,
        >,
{
    let selected = shared.selected();
    require_selected_program(selected, commit_authority.mode())?;

    let application_identity = crate::domain_computation::execution_observation::is_installed().then(|| {
        crate::domain_computation::primary_graph::application_contribution::request_execution::advancement::observation::WorthQueryAdvancementIdentityForTest::Producer {
        producer: Binding::IDENTITY.to_owned(),
        query: *observed_source.query_identity.as_bytes(),
        parameters: *observed_source.parameter_binding_identity.bytes(),
        root: observed_source.source_root(),
        source: observed_source.checkpoint_observation_identity(),
        }
    });
    let source_epoch = observed_source.idempotency_identity();
    let key = provider.idempotency_key(&source, &source_epoch.bytes());
    let input_execution = phase
        .checked_execution_request_for(&runtime.runtime.product_runtime)
        .map_err(|stop| {
            denial(
                WorthQueryOutputDemandDenialKind::ExecutionRequest(stop),
                Binding::IDENTITY,
            )
        })?;
    let (owned_identities, recovery_input) =
        super::super::unpublished::prepare_input::<Schema, Binding>(
            &input_execution,
            key,
            input,
            edition,
            commit_authority.mode().clone(),
        )?;
    let identities = owned_identities.borrow();
    let prepared_key = prepared_key(&observed_source, *identities.input_identity(), edition)?;
    let prepared_context = if let Some(contract) = Binding::INPUT_REUSE.filter(|contract| {
        contract.determinism() == worth_foundational::facade::DeterminismContract::CanonicalBitwise
    }) {
        let declared = contract.context();

        let key = declared
            .contains(super::super::super::WorthQueryDecisionContextDependencies::KEY)
            .then(|| *identities.key_identity());
        let principal_witness = declared
            .contains(super::super::super::WorthQueryDecisionContextDependencies::PRINCIPAL)
            .then(|| principal.freshness().reuse_witness());
        let scope_witness = declared
            .contains(super::super::super::WorthQueryDecisionContextDependencies::SCOPE)
            .then(|| {
                admission
                    .operation_scope_binding()
                    .decision_reuse_witness(*admission.operation_authority_identity_bytes())
            });
        crate::domain_computation::primary_graph::output_lineage::PreparedDecisionReuseContext::new(
            contract,
            key,
            principal_witness,
            scope_witness,
        )
    } else {
        None
    };

    let contact_root = observed_source.model_root;
    let prepared_source = runtime
        .prepare_application_source_expectation::<Operation<Schema, Binding>, _>(
            phase,
            &admission,
            observed_source,
            identities.mutation_input(),
        )
        .map_err(|error| input_cutoff::source_preparation_denial(Binding::IDENTITY, error))?;

    let (mut required_output, prepared_source, prepared_key, prepared_context) =
        match input_cutoff::advance_input_cutoff::<Schema, Binding, _, _, _>(
            phase,
            runtime,
            selected,
            &admission,
            prepared_source,
            required_output,
            prepared_key,
            prepared_context,
            matched_predecessors,
            edition,
        )? {
            input_cutoff::ProducerInputProgression::StablePrepared(publish) => {
                return Ok(super::super::prepared_plan::ProducerExecutionPlan::new(
                    Vec::new(),
                    move |slots| {
                        assert!(slots.is_empty(), "stable reuse has no compute slots");
                        let published = publish(&admission)?;

                        if let Some(identity) = application_identity {
                            crate::domain_computation::primary_graph::application_contribution::request_execution::record_event(
                                crate::domain_computation::primary_graph::application_contribution::request_execution::advancement::observation::WorthQueryAdvancementEventForTest {
                                    kind: crate::domain_computation::primary_graph::application_contribution::request_execution::advancement::observation::WorthQueryAdvancementEventKindForTest::Publication,
                                    identity,
                                });
                        }
                        Ok(ProducerExecutionOutcome::Stable(published))
                    },
                ));
            }
            input_cutoff::ProducerInputProgression::FreshPrepared {
                required_output,
                source,
                key,
                context,
            } => (required_output, source, key, context),
        };
    if let Some(key) = prepared_key {
        required_output.retain_prepared_input_reuse_key(key);
    }
    if let Some(context) = prepared_context {
        required_output.retain_prepared_decision_reuse(context);
    }
    let publication_scope = admission.operation_scope_binding().clone();
    admission.bind_required_output_demand(required_output);
    let bound_source = prepared_source
        .consume_into(&mut admission)
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let source_query =
        crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess::new(
            runtime,
            shared,
            control,
            commit_authority.history_control(),
            external_principal,
            request_scope,
        )
        .map_err(super::super::denial::request_admission_rejected)?;
    let prepared = runtime
        .prepare_mutation_handler_plan_with_source_access::<Operation<Schema, Binding>>(
            phase,
            &identities,
            principal.principal_identity(),
            Some(&source_query),
            admission,
            || {
                *producer_contacts += 1;

                super::super::contact_observation::record(
                    runtime.runtime.authority_identity().as_u64(),
                    contact_root,
                );
            },
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    if let Some(stop) = source_query.take_history_failure() {
        return Err(super::super::denial::request_admission_rejected(stop));
    }
    let (requests, completion, inputs) = match prepared {
        Ok(prepared) => prepared.into_parts(&runtime.runtime.output_demands)?,
        Err(report) => {
            return Ok(super::super::prepared_plan::ProducerExecutionPlan::new(
                Vec::new(),
                move |slots| {
                    assert!(
                        slots.is_empty(),
                        "a refused preparation has no compute slots"
                    );
                    let result = report
                        .into_outcome()
                        .map_err(|error| execution_failed(Binding::IDENTITY, error))?;
                    completed_handler::<_, _, DeclaredDenial<Schema, Binding>>(
                        Binding::IDENTITY,
                        result,
                        |denial| provider.domain_denial_reason(denial),
                    )
                    .map(|impossible| match impossible {})
                    .map_err(ProducerExecutionStop::ExecutionStopped)
                },
            ))
        }
    };
    Ok(super::super::prepared_plan::ProducerExecutionPlan::new(
        requests,
        move |slots| {

            let _observation = application_identity.map(crate::domain_computation::primary_graph::application_contribution::request_execution::advancement::observation::application::apply_identity);
            let identities = owned_identities.borrow();
            let (program, idempotency) = super::completion::prepare::<Schema, Binding>(
                phase, runtime, &identities, bound_source, edition, successor_of,
                || {
            let completed = completed_handler::<_, _, DeclaredDenial<Schema, Binding>>(
                Binding::IDENTITY,
                completion
                    .finish(runtime, &identities, slots, phase.execution_for(&runtime.runtime.product_runtime)
                        .map_err(|cause| denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()), "producer completion phase"))?)
                    .into_outcome()
                    .map_err(|error| execution_failed(Binding::IDENTITY, error))?,
                |denial| provider.domain_denial_reason(denial),
            )?;
            Ok(completed.into_parts().0)
                },
            )?;
            let outcome = match commit_authority.mode() {
                WorthQueryProducerCommitAuthority::Ordinary => runtime
                    .compare_and_commit_application_in_advancement_with_history(
                        phase,
                        program,
                        idempotency,
                        commit_authority.history_control(),
                        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                    ),
                WorthQueryProducerCommitAuthority::ProgramOutput => runtime
                    .compare_and_commit_application_for_program_output_producer(
                        &phase.execution_request_for(&runtime.runtime.product_runtime).map_err(|cause| denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()), "producer commit phase"))?,
                        phase,
                        program,
                        idempotency,
                        None,
                        commit_authority.history_control(),
                    ),
                WorthQueryProducerCommitAuthority::SelectedProgram { identity, revision } => {
                    runtime.compare_and_commit_application_for_program_output_producer(
                        &phase.execution_request_for(&runtime.runtime.product_runtime).map_err(|cause| denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()), "producer commit phase"))?,
                        phase,
                        program,
                        idempotency,
                        Some((identity, revision)),
                        commit_authority.history_control(),
                    )
                }
            };
            if let crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome::RecoveredCommitted(receipt) = outcome {
                let execution = phase.execution_request_for(&runtime.runtime.product_runtime)
                    .map_err(|cause| denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()), "original producer publication"))?;
                // Refresh only the same actual branch occurrence for output-owner
                // selection. The handler's original source/dependency binding stays intact.
                let current = runtime.runtime.product_runtime.owner.observation_port()
                    .observe_product_branch_occurrence(selected.product().observation().lifecycle_incarnation())
                    .map_err(|_| denial(WorthQueryOutputDemandDenialKind::PublicationStale, "original producer occurrence unavailable")
                        .with_recovered_publication(receipt.clone())
                        .with_recovery_posture(crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable))?;
                let stable = runtime.runtime.primary_provider.graph.output_lineage.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .resolve_original_publication::<Operation<Schema, Binding>>(
                        &execution, &publication_scope, &current, idempotency, &receipt,
                    ).map_err(|stop| super::super::source_readmission::ready_resource_denial(Binding::IDENTITY, stop))?;
                return stable.map(ProducerExecutionOutcome::Stable).ok_or_else(||
                    denial(WorthQueryOutputDemandDenialKind::ProducerUnavailable, Binding::IDENTITY)
                        .with_recovered_publication(receipt).into());
            }
            if let WorthQueryApplicationCommitOutcome::ProductUnpublished(original) = outcome {
                return Ok(ProducerExecutionOutcome::Unpublished(
                    recovery_input.unpublished(idempotency, original),
                ));
            }
            commit_receipt(Binding::IDENTITY, outcome)
                .map(ProducerExecutionOutcome::Committed)
                .map_err(ProducerExecutionStop::ExecutionStopped)
        },
    ).with_inputs(inputs))
}

/// The domain-denial value declaration of the binding's provider.
type DeclaredDenial<Schema, Binding> = <<Binding as WorthQueryApplicationProducerBinding<
    Schema,
>>::Provider as WorthQueryApplicationProducerProvider<Schema, Binding>>::DenialValue;
