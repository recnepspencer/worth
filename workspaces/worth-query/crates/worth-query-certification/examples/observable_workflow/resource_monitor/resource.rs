use crate::document_retention_model::{
    retention_entry::{DocumentRetentionQueryBinding, DocumentRetentionRead, DOCUMENT_IDENTITY},
    schema::*,
};
use worth_query_decl::facade::{
    application_query::*, application_resource::*, application_schema::*, worth_query_portable_type,
};
pub struct Monitor;
worth_query_portable_type!(Monitor => "neutral.recoverable-archive.monitor.v1");
impl ApplicationResourceBinding<DocumentRetentionSchema> for Monitor {
    type Access = DocumentRetentionQueryBinding;
    type InputBinding = U64ApplicationValueBinding;
    type ResultBinding = U64ApplicationValueBinding;
    type FailureBinding = StringApplicationValueBinding;
    type Provider = super::producer::ProviderRole;
    const IDENTITY: &'static str = "neutral.recoverable-archive.monitor.v1";
    fn scope(
        _: &u64,
    ) -> <Self::Access as ApplicationQueryBinding<DocumentRetentionSchema>>::ScopeBinding {
        DocumentRetentionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        }
        .into_scope()
    }
    fn parameters(
        _: &u64,
    ) -> ApplicationQueryParameterSet<
        <Self::Access as ApplicationQueryBinding<DocumentRetentionSchema>>::Query,
    > {
        ApplicationQueryParameterSet::new()
    }
}
pub fn definition() -> ApplicationResourceDefinition<DocumentRetentionSchema, Monitor> {
    ApplicationResourceDefinition::request_response(
        ApplicationResourceRequestResponseProfile {
            retry_max_attempts: 2,
            retry_delay_milliseconds: 2,
            timeout_milliseconds: 5,
        },
        ApplicationResourceLimits {
            maximum_live: 1,
            maximum_retained_results: 1,
            ingress_slots_per_request: 2,
            maximum_input_bytes: 32,
            maximum_frame_bytes: 2048,
        },
        ApplicationResourceAuthorization::public(),
        ApplicationQueryDisclosureContract::public(),
    )
    .unwrap()
}
