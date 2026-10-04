//! Shared post-authorization producer progression for ordinary and selected Fresh.

use super::*;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};

type MutationScope<Schema, Binding> = <<Operation<Schema, Binding> as ApplicationMutationBinding<
    Schema,
>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope;

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_authorized<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
        '_,
        Schema,
    >,
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
    mut required_output: RequiredOutputDemandContext,
    provider: &Binding::Provider,
    source: SourceValue<Schema, Binding>,
    observed_source: crate::domain_computation::primary_graph::WorthQueryObservedSource<
        SourceQuery<Schema, Binding>,
    >,
    input: <Operation<Schema, Binding> as ApplicationMutationBinding<Schema>>::Input,
    matched_predecessors: Option<super::super::demand::MatchedRequiredPredecessors<'_>>,
    resources: super::super::WorthQueryProducerDemandResources,
    successor_of: Option<[u8; 32]>,
    commit_authority: WorthQueryProducerCommitAuthority,
    edition: super::super::InstalledProducerEdition,
    limits: WorthQueryOutputDemandLimits,
    request_admission: &mut InvalidationEditAdmission,
    producer_contacts: &mut usize,
) -> Result<ProducerExecutionOutcome, ProducerExecutionStop>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, Binding>,
        >,
{
    require_selected_program(selected, &commit_authority, request_admission)?;
    let source_epoch = observed_source.idempotency_identity();
    let key = provider.idempotency_key(&source, &source_epoch.bytes());
    let identities = encode_input::<Schema, Operation<Schema, Binding>>(
        Binding::IDENTITY,
        &key,
        &input,
        request_admission,
    )?;
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
                declared.contains(super::super::WorthQueryDecisionContextDependencies::KEY),
            ) + if declared.contains(super::super::WorthQueryDecisionContextDependencies::PRINCIPAL)
            {
                10
            } else {
                0
            } + if declared.contains(super::super::WorthQueryDecisionContextDependencies::SCOPE) {
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
            .contains(super::super::WorthQueryDecisionContextDependencies::KEY)
            .then(|| *identities.key_identity());
        let principal_witness = declared
            .contains(super::super::WorthQueryDecisionContextDependencies::PRINCIPAL)
            .then(|| principal.freshness().reuse_witness());
        let scope_witness = declared
            .contains(super::super::WorthQueryDecisionContextDependencies::SCOPE)
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
    let prepared_source = runtime
        .prepare_application_source_expectation::<Operation<Schema, Binding>, _>(
            &admission,
            observed_source,
            input.input(),
            request_admission,
        )
        .map_err(|error| input_cutoff::source_preparation_denial(Binding::IDENTITY, error))?;
    required_output.retain_actual_resources(resources);
    // Only re-verifying marked facts and the full-verification fallback spend
    // the source-currentness allowance; exhausting it selects Fresh.
    let mut currentness = runtime
        .primary_provider
        .graph
        .source_owner
        .invalidation_owner
        .read_admission(limits.source_currentness_work());
    let (mut required_output, prepared_source, prepared_key, prepared_context) =
        match input_cutoff::advance_input_cutoff::<Schema, Binding, _, _, _>(
            runtime,
            selected,
            &admission,
            prepared_source,
            required_output,
            prepared_key,
            prepared_context,
            matched_predecessors,
            resources,
            request_admission,
            &mut currentness,
        )? {
            input_cutoff::ProducerInputProgression::StablePublished(published) => {
                return Ok(ProducerExecutionOutcome::Stable(published))
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
    let completed = completed_handler(
        Binding::IDENTITY,
        runtime
            .execute_mutation_handler_observing_contact::<Operation<Schema, Binding>>(
                &identities,
                principal.principal_identity(),
                admission,
                || *producer_contacts += 1,
            )
            .map_err(|error| execution_failed(Binding::IDENTITY, error))?,
    )?;
    let (mut program, _) = completed.into_parts(); // Retries bind the complete typed dependency set.
    let (key_identity, dependency_identity) = program
        .producer_idempotency_identities::<Operation<Schema, Binding>>(
            runtime,
            *identities.key_identity(),
            successor_of,
            request_admission,
        )
        .map_err(|identity_denial| {
            if identity_denial
                == crate::domain_computation::primary_graph::application_attempt::WorthQueryProducerIdentityDenial::LineageLookupBudgetExceeded
            {
                // This admitted demand cannot increase its limit; execution closes terminally.
                denial(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    Binding::IDENTITY,
                )
            } else {
                denial(
                    WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                    format!("{}: {identity_denial:?}", Binding::IDENTITY),
                )
            }
        })?;
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
        WorthQueryProducerCommitAuthority::Ordinary => {
            runtime.compare_and_commit_application(program, idempotency)
        }
        WorthQueryProducerCommitAuthority::ProgramOutput => runtime
            .compare_and_commit_application_for_program_output_producer(program, idempotency, None),
        WorthQueryProducerCommitAuthority::SelectedProgram { identity, revision } => runtime
            .compare_and_commit_application_for_program_output_producer(
                program,
                idempotency,
                Some((&identity, &revision)),
            ),
    };
    commit_receipt(Binding::IDENTITY, outcome)
        .map(ProducerExecutionOutcome::Committed)
        .map_err(ProducerExecutionStop::ExecutionStopped)
}
