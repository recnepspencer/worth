//! A candidate program commits only under the mutation binding whose handler
//! built it and the input that handler decided on, and one key and input never replay across bindings.
//!
//! `SetRetentionBinding` and `ReviewedSetRetentionBinding` act on one operation
//! with one input type. The guard tells them apart at commit; their key
//! namespaces keep a replay from crossing between them.

pub use worth_query_decl::facade::application_operation::ApplicationMutationIdentities;
pub use worth_query_host::facade::{
    application_installation::WorthQueryProgramOwner,
    primary_graph::{
        HandlerResult, WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
        WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationIdempotencyResolution,
        WorthQueryPrincipalResolutionMode,
    },
};

pub use worth_query_testkit_document_retention::facade::document_retention_model::host::publish_on_first_program;
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::readback::read_retention;
pub use worth_query_testkit_document_retention::facade::document_retention_model::retention_entry::{
    ReviewedSetRetentionBinding, SetRetentionBinding, DOCUMENT_IDENTITY,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::schema::{
    DocumentIdentityField, DocumentPrincipalBinding, DocumentRetentionSchema, SetRetention,
    SetRetentionInput,
};

/// Admits one `SetRetention` operation on the branch, expanding to the
/// admission with the request's identities for `$binding`.
#[macro_export]
macro_rules! worlds_certification_application_graph_mutation_binding_guard_rs_admit {
    ($runtime:expr, $branch:expr, $binding:ty, $key:expr, $input:expr) => {{
        let runtime = $runtime;
        let scope = request_scope();
        let external = authenticate_operator(runtime.installed_schema(), &scope);
        runtime
            .with_host_advancement(|phase| {
                let selected = runtime
                    .on_branch($branch)
                    .select(&phase)
                    .expect("branch selects");
                let principal_binding = runtime
                    .installed_schema()
                    .principal_binding(DocumentPrincipalBinding::reference())
                    .expect("principal binding installs");
                let principal = selected
                    .resolve_authenticated_principal(
                        &phase,
                        &principal_binding,
                        &external,
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .expect("operator resolves");
                let document = selected
                    .resolve_entity(
                        &phase,
                        DocumentIdentityField::reference(),
                        DOCUMENT_IDENTITY.to_owned(),
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .expect("document resolves");
                let operation = runtime
                    .installed_schema()
                    .installed_operation(SetRetention::reference())
                    .expect("operation installs");
                let admission = selected
                    .authorize_operation(
                        &phase,
                        &principal,
                        &document,
                        &operation,
                        Default::default(),
                        &scope,
                    )
                    .expect("the operation admits");
                let identities =
                    ApplicationMutationIdentities::<DocumentRetentionSchema, $binding>::encode(
                        &$key, &$input,
                    )
                    .expect("key and input encode");
                (principal, admission, identities)
            })
            .expect("admission host call opens")
    }};
}
pub use crate::worlds_certification_application_graph_mutation_binding_guard_rs_admit as admit;

/// Runs the installed handler for `$binding` and yields its candidate program.
#[macro_export]
macro_rules! worlds_certification_application_graph_mutation_binding_guard_rs_candidate {
    ($runtime:expr, $branch:expr, $binding:ty, $key:expr, $input:expr) => {{
        let (principal, admission, identities) = admit!($runtime, $branch, $binding, $key, $input);
        let HandlerResult::Completed(completed) = $runtime
            .with_application_advancement(
                &super::document_retention_model::operator_identity::request_scope(),
                |phase| {
                    $runtime.execute_mutation_handler::<$binding>(
                        &phase,
                        &identities,
                        principal.principal_identity(),
                        admission,
                     worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,)
                },
            )
            .expect("the fixture policy admits its handler advancement")
            .expect("the handler runs")
        else {
            panic!("the handler must produce a candidate");
        };
        completed.into_parts().0
    }};
}
pub use crate::worlds_certification_application_graph_mutation_binding_guard_rs_candidate as candidate;

pub fn input(days: u64) -> SetRetentionInput {
    SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: days,
    }
}

pub fn identities<'a, Binding>(
    key: &'a u64,
    input: &'a SetRetentionInput,
) -> ApplicationMutationIdentities<'a, DocumentRetentionSchema, Binding>
where
    Binding: worth_query_decl::facade::application_operation::ApplicationMutationBinding<
        DocumentRetentionSchema,
        IdempotencyKey = u64,
        Input = SetRetentionInput,
    >,
{
    ApplicationMutationIdentities::<DocumentRetentionSchema, Binding>::encode(key, input)
        .expect("key and input encode")
}

pub fn idempotency<Binding>(
    key: &u64,
    input: &SetRetentionInput,
) -> WorthQueryApplicationIdempotencyBinding
where
    Binding: worth_query_decl::facade::application_operation::ApplicationMutationBinding<
        DocumentRetentionSchema,
        IdempotencyKey = u64,
        Input = SetRetentionInput,
    >,
{
    WorthQueryApplicationIdempotencyBinding::for_mutation_identities(&identities::<Binding>(
        key, input,
    ))
}
