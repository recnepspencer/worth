use super::*;

pub(super) fn execute_fresh<Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    external_principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request_scope: &WorthQueryRequestScope,
    branch: WorthQueryProductBranch,
    required_output: RequiredOutputDemandContext,
    provider: &Binding::Provider,
    disclosure: FreshOutputDisclosure<SourceQuery<Schema, Binding>, SourceValue<Schema, Binding>>,
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
    let selected = runtime
        .on_branch(branch)
        .select()
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    if !disclosure.admits(
        external_principal,
        request_scope,
        selected.product().observation(),
    ) {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::Superseded,
            Binding::IDENTITY,
        )
        .into());
    }
    let (source, observed_source) = disclosure.into_parts();
    let resources = provider.demand_resources(&source);
    super::super::demand::validate_retained_resources(resources, Binding::IDENTITY, limits)?;
    let input = provider.operation_input(&source);
    let binding = runtime
        .installed_schema()
        .installed_mutation_binding::<Operation<Schema, Binding>>()
        .map_err(|error| failed(Binding::IDENTITY, error))?;
    let principal = selected
        .resolve_authenticated_principal(
            binding.principal_binding(),
            external_principal,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|error| request_admission_denied(Binding::IDENTITY, error))?;
    let (scope_field, scope_value) = input
        .scope_binding()
        .into_field_parts(principal.principal_identity());
    let scope = selected
        .resolve_entity(
            scope_field,
            scope_value,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(|error| request_admission_denied(Binding::IDENTITY, error))?;
    let admission = selected
        .authorize_operation(
            &principal,
            &scope,
            binding.operation(),
            TypedMutationPreconditions::default(),
            request_scope,
        )
        .map_err(|error| request_admission_denied(Binding::IDENTITY, error))?;
    post_authorization::execute_authorized::<Schema, Binding>(
        runtime,
        &selected,
        &principal,
        admission,
        required_output,
        provider,
        source,
        observed_source,
        input,
        None,
        resources,
        successor_of,
        commit_authority,
        edition,
        limits,
        request_admission,
        producer_contacts,
    )
}
