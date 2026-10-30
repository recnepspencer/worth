//! The public program-owner commit door cannot bypass workflow authority.

pub use super::*;
#[path = "commit_boundary/authenticated_signer.rs"]
pub mod authenticated_signer;
pub use worth_query_decl::facade::application_operation::ApplicationMutationIdentities;
pub use worth_query_host::facade::{
    application_installation::WorthQueryProgramOwner,
    primary_graph::{HandlerResult, WorthQueryPrincipalResolutionMode},
};

pub use super::super::document_retention_model::retention_entry::SetRetentionBinding;
pub use super::super::document_retention_model::schema::{
    DocumentIdentityField, DocumentRetentionSchema, SetRetention, SetRetentionInput,
};

pub fn reviewed_identities<'a>(
    key: &'a u64,
    input: &'a SetRetentionInput,
) -> ApplicationMutationIdentities<'a, DocumentRetentionSchema, ReviewedSetRetentionBinding> {
    ApplicationMutationIdentities::encode(key, input).expect("key and input must encode")
}

#[path = "commit_boundary/guarded_program.rs"]
pub mod guarded_program;

// This court alone exercises the workflow owner below publication.
pub mod advancement_identity {
    //! The real workflow publication adapter accepts only its installed owner's phase.
    pub use super::super::super::document_retention_model::{
        schema::{Document, DocumentPrincipalBinding, DocumentRetentionSchema},
        workflow::{WorkflowAdvanceCapability, WorkflowAdvanceInput, WorkflowAdvanceOperation},
    };
    pub use super::*;
    pub use worth_query_execution::publication_boundary::workflow_advance::{
        PreparedWorkflowAdvance, WorthQueryWorkflowAdvanceAdapter as Adapter,
    };
    pub use worth_query_host::facade::{
        application_contribution::WorthQueryManagedComputationResourceDenial as Resource,
        primary_graph::{
            advancement_requests_on_this_thread_for_test as reports,
            installed_source_reads_on_this_thread_for_test as reads,
            place_managed_computations_on_this_thread_for_test as place,
            WorthQueryApplicationIdempotencyBinding,
            WorthQueryExecutionPlacementForTest as Placement, WorthQueryPrincipalResolutionMode,
        },
    };
    pub type Prepared = PreparedWorkflowAdvance<
        DocumentRetentionSchema,
        WorkflowAdvanceOperation,
        WorkflowAdvanceInput,
        Document,
    >;
    pub fn fixture(
        key: u64,
    ) -> (
        DocumentWorkflowRuntime,
        Prepared,
        WorthQueryApplicationIdempotencyBinding,
    ) {
        let (application, _, instance, _, _, _) = approval_journey("identity", key);
        let runtime = application.runtime();
        let scope = request_scope();
        let external = authenticate_operator(runtime.installed_schema(), &scope);
        let prepared = runtime
            .with_application_advancement(&scope, |_phase| {
                let selected = runtime
                    .on_branch(instance.branch())
                    .select(&_phase)
                    .unwrap();
                let binding = runtime
                    .installed_schema()
                    .principal_binding(DocumentPrincipalBinding::reference())
                    .unwrap();
                let principal = selected
                    .resolve_authenticated_principal(
                        &_phase,
                        &binding,
                        &external,
                        &scope,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let operation = runtime
                    .installed_schema()
                    .installed_operation(WorkflowAdvanceOperation::reference())
                    .unwrap();
                let capability = runtime
                    .installed_schema()
                    .capability(
                        WorkflowAdvanceCapability::reference(),
                        WorkflowAdvanceOperation::reference(),
                    )
                    .unwrap();
                let access = selected
                    .admit_capability_access(
                        &_phase,
                        &principal,
                        &capability,
                        WorkflowAdvanceInput {
                            document_identity: DOCUMENT_IDENTITY.to_owned(),
                        },
                        &scope,
                    )
                    .unwrap();
                let admission = runtime
                    .authorize_capability_operation(&_phase, access, &operation, Default::default())
                    .unwrap();
                Adapter::prepare::<_, WorkflowAdvanceCapability, _, _, _, _>(
                    &_phase,
                    &selected,
                    application.vocabulary().workflow_spec_for(&selected),
                    instance,
                    admission,
                )
                .unwrap()
            })
            .unwrap();
        let idempotency = WorthQueryApplicationIdempotencyBinding::for_host_commit::<
            DocumentRetentionSchema,
            WorkflowAdvanceOperation,
            _,
            _,
        >(&key, &"cross-runtime-phase")
        .unwrap();
        (application, prepared, idempotency)
    }
}
