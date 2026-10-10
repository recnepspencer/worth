use worth_query_declaration::facade::application_operation::{
    ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
    ApplicationMutationIdentities, ApplicationMutationIntent, ApplicationMutationScopeBinding,
    ApplicationMutationScopeResolution, ApplicationMutationSourceExpectation,
};
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationIdempotencyBinding,
    WorthQueryPrincipalResolutionMode, WorthQuerySelectedProductOperation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    commit_binding::WorthQueryCommitExtension, request::WorthQueryMutationExpectedSource,
    staged::WorthQueryStagedMutation, WorthQueryApplicationMutationRequest,
    WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

#[path = "authorization/authorized.rs"]
mod authorized;
use authorized::AuthorizedMutation;
mod capability;
pub(in crate::application_entry) use capability::prepare_capability_selected;

type IntentBinding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type IntentPrincipal<Schema, Intent> =
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity;
type Preconditions<Schema, Intent> = TypedMutationPreconditions<
    Schema,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Operation,
    MutationScope<Schema, IntentBinding<Schema, Intent>>,
>;
type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope;

pub(in crate::application_entry) struct PreparedMutation<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub(in crate::application_entry) principal_identity: Binding::PrincipalIdentity,
    pub(in crate::application_entry) admission: WorthQueryAdmittedApplicationOperation<
        Schema,
        Binding::Operation,
        Binding::Input,
        MutationScope<Schema, Binding>,
    >,
    pub(in crate::application_entry) pending_source: Option<
        worth_query_execution::facade::primary_graph::WorthQueryPendingSourceExpectation<
            <Binding::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
    >,
    pub(in crate::application_entry) extension: WorthQueryCommitExtension,
    pub(in crate::application_entry) idempotency: WorthQueryApplicationIdempotencyBinding,
}

pub(super) fn assess<Schema, Intent, SourcePreparation>(
    phase: &AdvancementPhase<'_>,
    request: &mut WorthQueryApplicationMutationRequest<
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >,
) -> Result<(), WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<Schema, IntentPrincipal<Schema, Intent>>,
{
    let preconditions = std::mem::take(&mut request.preconditions);
    authorize(phase, request, preconditions).map(|_| ())
}

fn authorize<Schema, Intent, SourcePreparation>(
    phase: &AdvancementPhase<'_>,
    request: &WorthQueryApplicationMutationRequest<'_, '_, '_, Schema, Intent, SourcePreparation>,
    preconditions: Preconditions<Schema, Intent>,
) -> Result<
    AuthorizedMutation<Schema, IntentBinding<Schema, Intent>>,
    WorthQueryApplicationRequestMutationDenial,
>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<Schema, IntentPrincipal<Schema, Intent>>,
{
    request
        .application
        .validate_application_advancement(phase)
        .map_err(|cause| {
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause.into())
        })?;
    let selected = request
        .application
        .on_branch(request.branch)
        .select()
        .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
    authorize_selected(phase, request, preconditions, &selected)
}

fn authorize_selected<Schema, Intent, SourcePreparation>(
    _phase: &AdvancementPhase<'_>,
    request: &WorthQueryApplicationMutationRequest<'_, '_, '_, Schema, Intent, SourcePreparation>,
    preconditions: Preconditions<Schema, Intent>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
) -> Result<
    AuthorizedMutation<Schema, IntentBinding<Schema, Intent>>,
    WorthQueryApplicationRequestMutationDenial,
>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<Schema, IntentPrincipal<Schema, Intent>>,
{
    let binding = request
        .application
        .installed_schema()
        .installed_mutation_binding::<IntentBinding<Schema, Intent>>()
        .map_err(WorthQueryApplicationRequestMutationDenial::BindingInstallation)?;
    let principal = selected
        .resolve_authenticated_principal(
            binding.principal_binding(),
            request.principal,
            request.scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::PrincipalResolution)?;
    let (scope_field, scope_value) = request
        .intent
        .scope_binding()
        .into_field_parts(principal.principal_identity());
    let principal_identity = principal.principal_identity().clone();
    let scope = selected
        .resolve_entity(
            scope_field,
            scope_value,
            request.scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::ScopeResolution)?;
    let admission = selected
        .authorize_operation(
            &principal,
            &scope,
            binding.operation(),
            preconditions,
            request.scope,
        )
        .map_err(WorthQueryApplicationRequestMutationDenial::Authorization)?;
    Ok(AuthorizedMutation {
        principal_identity,
        admission,
    })
}

/// The identities of one request, encoded once by the entry point.
type Identities<'request, Schema, Intent> =
    ApplicationMutationIdentities<'request, Schema, IntentBinding<Schema, Intent>>;

type Request<'a, 'p, 's, 'k, Schema, Intent, SourcePreparation> =
    WorthQueryApplicationMutationRequestWithIdempotency<
        'a,
        'p,
        's,
        'k,
        Schema,
        Intent,
        SourcePreparation,
    >;

pub(in crate::application_entry) fn prepare<Schema, Intent, SourcePreparation>(
    phase: &AdvancementPhase<'_>,
    request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>,
    identities: &Identities<'_, Schema, Intent>,
    staged: WorthQueryStagedMutation<Schema, Intent>,
) -> Result<PreparedMutation<Schema, IntentBinding<Schema, Intent>>, WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    let authorized = authorize(phase, &request.request, staged.preconditions)?;
    prepare_authorized(request, identities, staged.source, authorized)
}

pub(super) fn prepare_selected<Schema, Intent, SourcePreparation>(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>,
    identities: &Identities<'_, Schema, Intent>,
    staged: WorthQueryStagedMutation<Schema, Intent>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
) -> Result<PreparedMutation<Schema, IntentBinding<Schema, Intent>>, WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    let authorized = authorize_selected(phase, &request.request, staged.preconditions, selected)?;
    prepare_authorized(request, identities, staged.source, authorized)
}

fn prepare_authorized<Schema, Intent, SourcePreparation>(
    request: &Request<'_, '_, '_, '_, Schema, Intent, SourcePreparation>,
    identities: &Identities<'_, Schema, Intent>,
    source: Option<
        WorthQueryMutationExpectedSource<
            <<IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
    >,
    authorized: AuthorizedMutation<Schema, IntentBinding<Schema, Intent>>,
) -> Result<PreparedMutation<Schema, IntentBinding<Schema, Intent>>, WorthQueryApplicationRequestMutationDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    let principal_identity = authorized.principal_identity;
    let mut admission = authorized.admission;
    // The request encoded its key and input once, before admission; report that
    // work in the admission phase so a fresh commit and a replayed retry both
    // carry it.
    admission.record_request_identity_work(
        &worth_query_execution::publication_boundary::program_publication_access(),
        identities.canonical_work(),
    );

    let expected_source = <<IntentBinding<Schema, Intent> as ApplicationMutationBinding<
        Schema,
    >>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::QUERY_IDENTIFIER;
    let mut pending_source = None;
    match (expected_source, source) {
        (Some(_expected), Some(source)) => {
            pending_source = Some(match source {
                WorthQueryMutationExpectedSource::Row(source) => request
                    .request
                    .application
                    .bind_application_source_expectation::<IntentBinding<Schema, Intent>, _>(
                        &mut admission,
                        source,
                        identities.mutation_input(),
                    ),
                WorthQueryMutationExpectedSource::ResultSet(source) => request
                    .request
                    .application
                    .bind_application_result_set_expectation::<IntentBinding<Schema, Intent>, _>(
                        &mut admission,
                        source,
                        identities.mutation_input(),
                    ),
            }
            .map_err(WorthQueryApplicationRequestMutationDenial::SourceExpectation)?);
        }
        (Some(expected), None) => {
            return Err(WorthQueryApplicationRequestMutationDenial::SourceExpectation(
                worth_query_execution::facade::primary_graph::WorthQuerySourceExpectationDenial::new_missing(expected),
            ));
        }
        (None, None) => {}
        (None, Some(_)) => unreachable!("a no-source binding has no constructible source marker"),
    }
    let extension = WorthQueryCommitExtension {
        source: pending_source
            .as_ref()
            .map(|pending| pending.bound_source()),
        workflow_transition: request.workflow_transition_identity,
    };
    Ok(PreparedMutation {
        principal_identity,
        admission,
        pending_source,
        extension,
        idempotency: extension
            .apply(WorthQueryApplicationIdempotencyBinding::for_mutation_identities(identities)),
    })
}
