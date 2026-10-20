//! Prepare a producer once and retain owner completion until canonical apply.
use super::*;
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
    commit_authority: WorthQueryProducerCommitAuthority,
    edition: super::super::super::InstalledProducerEdition,
    limits: WorthQueryOutputDemandLimits,
    request_admission: &mut InvalidationEditAdmission,
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
    require_selected_program(selected, &commit_authority, request_admission)?;

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
    let owned_identities = super::super::input_identity::encode_owned_input::<
        Schema,
        Operation<Schema, Binding>,
    >(Binding::IDENTITY, key, input, request_admission)?;
    let identities = owned_identities.borrow();
    let prepared_key = prepared_key(
        &observed_source,
        *identities.input_identity(),
        edition,
        request_admission,
    )?;
    let prepared_context = if let Some(contract) = Binding::INPUT_REUSE.filter(|contract| {
        contract.determinism() == worth_foundational::facade::DeterminismContract::CanonicalBitwise
    }) {
        let declared = contract.context();
        let copy_visits =
            1 + u64::from(
                declared.contains(super::super::super::WorthQueryDecisionContextDependencies::KEY),
            ) + if declared
                .contains(super::super::super::WorthQueryDecisionContextDependencies::PRINCIPAL)
            {
                10
            } else {
                0
            } + if declared
                .contains(super::super::super::WorthQueryDecisionContextDependencies::SCOPE)
            {
                5
            } else {
                0
            };
        request_admission
            .charge_external_work(copy_visits)
            .map_err(|_| {
                denial(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    Binding::IDENTITY,
                )
            })?;
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
            &admission,
            observed_source,
            identities.mutation_input(),
            request_admission,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(|error| input_cutoff::source_preparation_denial(Binding::IDENTITY, error))?;
    let mut currentness = runtime
        .primary_provider
        .graph
        .source_owner
        .invalidation_owner
        .edit_admission_with_work_limit(limits.source_currentness_work());
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
            request_admission,
            &mut currentness,
        )? {
            input_cutoff::ProducerInputProgression::StablePrepared(publish) => {
                return Ok(super::super::prepared_plan::ProducerExecutionPlan::new(
                    Vec::new(),
                    move |slots, admission_work| {
                        assert!(slots.is_empty(), "stable reuse has no compute slots");
                        let published = publish(&admission, admission_work)?;

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
    admission.bind_required_output_demand(required_output);
    let bound_source = prepared_source
        .consume_into(&mut admission)
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let source_query =
        crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess {
            application: runtime,
            shared,
            principal: external_principal,
            request: request_scope,
        };
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
    let (requests, completion, inputs) = match prepared {
        Ok(prepared) => prepared.into_parts(
            &runtime.output_demands,
            &runtime
                .primary_provider
                .graph
                .source_owner
                .invalidation_owner,
            request_admission,
        )?,
        Err(report) => {
            return Ok(super::super::prepared_plan::ProducerExecutionPlan::new(
                Vec::new(),
                move |slots, _| {
                    assert!(
                        slots.is_empty(),
                        "a refused preparation has no compute slots"
                    );
                    let result = report
                        .into_outcome()
                        .map_err(|error| execution_failed(Binding::IDENTITY, error))?;
                    completed_handler(Binding::IDENTITY, result, |denial| {
                        provider.domain_denial_reason(denial)
                    })
                    .map(|_| unreachable!("a refused preparation cannot produce a candidate"))
                    .map_err(ProducerExecutionStop::ExecutionStopped)
                },
            ))
        }
    };
    Ok(super::super::prepared_plan::ProducerExecutionPlan::new(
        requests,
        move |slots, request_admission| {

            let _observation = application_identity.map(crate::domain_computation::primary_graph::application_contribution::request_execution::advancement::observation::application::apply_identity);
            let identities = owned_identities.borrow();
            let completed = completed_handler(
                Binding::IDENTITY,
                completion
                    .finish(runtime, &identities, slots)
                    .into_outcome()
                    .map_err(|error| execution_failed(Binding::IDENTITY, error))?,
                |denial| provider.domain_denial_reason(denial),
            )?;
            let (mut program, _) = completed.into_parts(); // Retries bind the complete typed dependency set.
            let identity_execution = phase.execution_request_for(&runtime.product_runtime)
                .map_err(|cause| denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()), "producer content phase"))?;
            let (key_identity, dependency_identity) = program
        .producer_idempotency_identities::<Operation<Schema, Binding>>(
            runtime,
            &phase.checked_execution_request_for(&runtime.product_runtime).map_err(|stop| denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(stop), Binding::IDENTITY))?,
            *identities.key_identity(),
            successor_of,
            request_admission,
            identity_execution.allocation_policy(),
        )
        .map_err(|identity_denial| super::super::denial::identity_failed(Binding::IDENTITY, identity_denial))?;
            let idempotency = bound_source
                .bind_idempotency(
                    WorthQueryApplicationIdempotencyBinding::new(
                        key_identity,
                        *identities.input_identity(),
                    )
                    .bind_mutation::<Schema, Operation<Schema, Binding>>(),
                )
                .bind_producer_dependency(&dependency_identity);
            let program = program
                .with_output_demand_observation()
                .with_producer_required_invariants(Binding::REQUIRED_INVARIANTS);
            let outcome = match commit_authority {
                WorthQueryProducerCommitAuthority::Ordinary => runtime
                    .compare_and_commit_application_in_advancement(
                        phase,
                        program,
                        idempotency,
                        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                    ),
                WorthQueryProducerCommitAuthority::ProgramOutput => runtime
                    .compare_and_commit_application_for_program_output_producer(
                        &phase.execution_request_for(&runtime.product_runtime).map_err(|cause| denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()), "producer commit phase"))?,
                        phase,
                        program,
                        idempotency,
                        None,
                    ),
                WorthQueryProducerCommitAuthority::SelectedProgram { identity, revision } => {
                    runtime.compare_and_commit_application_for_program_output_producer(
                        &phase.execution_request_for(&runtime.product_runtime).map_err(|cause| denial(WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()), "producer commit phase"))?,
                        phase,
                        program,
                        idempotency,
                        Some((&identity, &revision)),
                    )
                }
            };
            commit_receipt(Binding::IDENTITY, outcome)
                .map(ProducerExecutionOutcome::Committed)
                .map_err(ProducerExecutionStop::ExecutionStopped)
        },
    ).with_inputs(inputs))
}
