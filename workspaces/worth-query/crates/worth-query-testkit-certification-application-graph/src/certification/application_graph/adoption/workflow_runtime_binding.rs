//! Workflow configuration refusal preserves runtime, vocabulary and signing custody.
pub use worth_query_decl::facade::application_program::{
    ApplicationActionReference, ApplicationRootComposition,
};
pub use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    application_installation::WorthQueryWorkflowRuntimeBindingDenial,
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::publish_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    programs::{DocumentRetentionFeature, RetentionCompletion, RetentionProgramP0},
    retention_entry::{SetRetentionBinding, SetRetentionIntent, DOCUMENT_IDENTITY},
    schema::{DocumentRetentionSchema, SetRetentionInput},
    workflow::{
        install_certification_authentication_with_clock, install_workflow_spec, workflow_resources,
    },
};

pub type Action = ApplicationActionReference<
    DocumentRetentionSchema,
    ApplicationRootComposition,
    DocumentRetentionFeature,
    SetRetentionBinding,
    RetentionCompletion,
>;
