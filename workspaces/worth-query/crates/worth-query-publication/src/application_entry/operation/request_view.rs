//! A fresh native request borrows the complete original operation meaning.
use super::descriptor::WorthQueryOperationDescription;
use crate::application_entry::mutation::request::WorthQueryMutationExpectedSource;
use crate::application_entry::{
    WorthQueryApplicationMutationRequest, WorthQueryApplicationMutationRequestWithIdempotency,
    WorthQueryApplicationRequest, WorthQueryMutationSourcePrepared,
};
use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema, Intent: ApplicationMutationIntent<Schema>>
    WorthQueryOperationDescription<Schema, Intent>
{
    pub(super) fn request_view<'borrow>(
        &'borrow self,
        request: &'borrow WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        installed_action: &'borrow worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
    ) -> WorthQueryApplicationMutationRequestWithIdempotency<
        'borrow,
        'borrow,
        'borrow,
        'borrow,
        Schema,
        &'borrow Intent,
        WorthQueryMutationSourcePrepared,
    > {
        let source = self.source.as_ref().map(|source| match source {
            WorthQueryMutationExpectedSource::Row(source) => {
                WorthQueryMutationExpectedSource::Row(source.clone())
            }
            WorthQueryMutationExpectedSource::ResultSet(source) => {
                WorthQueryMutationExpectedSource::ResultSet(source.clone())
            }
        });
        let native = WorthQueryApplicationMutationRequest {
            application: request.application,
            principal: request.principal,
            scope: request.scope,
            branch: self.branch,
            intent: &self.intent,
            preconditions: Default::default(),
            source,
            source_preparation: std::marker::PhantomData,
        };
        let mut native = native.idempotency(&self.key);
        native.compiled_action = Some(installed_action);
        native.borrowed_preconditions = Some(&self.preconditions);
        native.operation_input = Some(&self.encoded_input);
        native
    }
}
