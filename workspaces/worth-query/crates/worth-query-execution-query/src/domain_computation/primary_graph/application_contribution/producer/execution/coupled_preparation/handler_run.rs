//! Isolated coupled producer handler execution keeping `prepare` frames bounded.
use super::*;

#[inline(never)]
#[allow(clippy::too_many_arguments)]
pub(super) fn run_coupled_producer_handler<Schema, Binding>(
    installed: &TypedInstalledProducer<Schema, Binding>,
    phase: &WorthQueryAdvancementPhase<'_>,
    coordination: &crate::domain_computation::primary_graph::provider::WorthQueryApplicationBranchCommitCoordination<'_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    shared: &crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation<'_, Schema>,
    before_pin: &RelationalPinnedRead,
    candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
    participants: &dyn crate::domain_computation::primary_graph::provider::AppliedOutputParticipants,
    source: &AppliedQuerySource<SourceQuery<Schema, Binding>, SourceValue<Schema, Binding>>,
    qualified: crate::domain_computation::primary_graph::application_contribution::QualifiedOutputSource,
    selected_publication: worth_relational::facade::runtime::PositionedRelationalSnapshot,
    identities: &worth_query_declaration::facade::application_operation::ApplicationMutationIdentities<
        '_,
        Schema,
        Operation<Schema, Binding>,
    >,
    principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
        Schema,
        <Operation<Schema, Binding> as worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>>::Principal,
        <Operation<Schema, Binding> as worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>>::PrincipalIdentity,
    >,
    scope: Option<
        &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            Schema,
            <<Operation<Schema, Binding> as worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>>::ScopeBinding as worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
    >,
    mut admission: crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        <Operation<Schema, Binding> as worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>>::Operation,
        <Operation<Schema, Binding> as worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>>::Input,
        <<Operation<Schema, Binding> as worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>>::ScopeBinding as worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding<Schema>>::Scope,
    >,
    edition: super::super::super::InstalledProducerEdition,
    captured: Option<(
        super::super::input_preparation::PreparedProducerInputs<Schema, Binding>,
        std::any::TypeId,
    )>,
) -> Result<
    (
        crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<
            Schema,
            <Operation<Schema, Binding> as worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>>::Operation,
            <Operation<Schema, Binding> as worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>>::Input,
            <<Operation<Schema, Binding> as worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>>::ScopeBinding as worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        <Operation<Schema, Binding> as worth_query_declaration::facade::application_operation::ApplicationMutationBinding<Schema>>::Result,
    ),
    ProducerExecutionStop,
>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    SourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, Binding>,
        >,
{
    let computation_prior = input_cutoff::prepare_computation_prior::<Schema, Binding>(
        execution,
        runtime,
        shared.selected(),
        admission.operation_scope_binding(),
        source.partition(),
        edition,
    )?;
    let prepared_key = source.retain_selected_membership().map(|selection| {
        crate::domain_computation::primary_graph::output_lineage::PreparedInputReuseKey::new(
            selection,
            *identities.input_identity(),
            edition,
        )
    });
    let prepared_context = input_reuse::prepared_context(
        Binding::INPUT_REUSE,
        *identities.key_identity(),
        || principal.freshness().reuse_witness(),
        || {
            admission
                .operation_scope_binding()
                .decision_reuse_witness(*admission.operation_authority_identity_bytes())
        },
    );
    use crate::domain_computation::provider_session::ManagedGraphWorkSessionPrimaryExt;
    admission.graph_work_mut().arm_coupled_handler();
    let handler = admission
        .graph_work_mut()
        .coupled_handler_typed_mut()
        .expect("armed handler");
    if let Some(prior) = computation_prior {
        handler.retain_computation_prior(prior);
    }
    if let Some(key) = prepared_key {
        handler.retain_prepared_input_reuse_key(key);
    }
    if let Some(context) = prepared_context {
        handler.retain_prepared_decision_reuse(context);
    }

    let predecessor_reads =
        super::super::super::observation_reads::ProducerPredecessorReadAccess::prepare_coupled::<
            Schema,
            Binding,
        >(
            admission
                .graph_work_mut()
                .coupled_handler_typed_mut()
                .expect("armed handler"),
            source.checkpoint_identity(),
            edition,
        );
    let producer_request = admission.retain_admitted_request();
    let producer_principal = admission.operation_scope_binding().principal();
    let read_admission = scope.map(|scope| {
        crate::domain_computation::authorization::BorrowedReadOperationAdmission::new(
            runtime,
            principal,
            scope,
            request,
            admission.graph_work().identity(),
        )
    });
    let source_query = crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess::ordinary(
        runtime, shared, external, request,
    ).with_coordinated_control(
        crate::domain_computation::primary_graph::product_operation::current_read::CurrentReadControl::new(
            shared.selected().product().read_lease_ref(), &producer_request, &producer_principal,
        ), coordination,
    ).map_err(super::super::denial::request_admission_rejected)?
        .with_producer_reads::<Binding>(read_admission, predecessor_reads)
        .with_applied_participants(participants)
        .with_applied_source_publication(qualified, selected_publication);
    let prepared_inputs = match captured {
        Some((inputs, component)) => Some(inputs.into_mutation_inputs(
            installed,
            identities,
            source.root(),
            edition,
            component,
            execution,
            candidate,
        )?),
        None => {
            installed.refuse_uncaptured_production()?;
            None
        }
    };
    let prepared_handler = runtime
        .prepare_mutation_handler_plan_on_basis::<Operation<Schema, Binding>>(
            phase,
            identities,
            principal.principal_identity(),
            Some(&source_query),
            admission,
            Some((before_pin, candidate)),
            own_outputs,
            prepared_inputs,
            || {
                contact_observation::record(
                    runtime.runtime.authority_identity().as_u64(),
                    source.root(),
                );
            },
            execution.allocation_policy(),
        );
    let mut report = match prepared_handler {
        Ok(handler) => handler.run(runtime, identities),
        Err(report) => report.into_uncompleted_report(),
    };
    let refusal_proofs = report.take_refusal_proofs();
    if let Some(stop) = source_query.take_control_failure() {
        return Err(super::super::denial::request_admission_rejected(stop));
    }
    let completed = completed_handler::<
        _,
        _,
        <Binding::Provider as WorthQueryApplicationProducerProvider<Schema, Binding>>::DenialValue,
    >(
        Binding::IDENTITY,
        report
            .into_outcome()
            .map_err(|cause| execution_failed(Binding::IDENTITY, cause))?,
        |cause| installed.provider.domain_denial_reason(cause),
    )
    .map_err(|denial| denial.with_refusal_proofs(refusal_proofs))?;
    Ok(completed.into_parts())
}
