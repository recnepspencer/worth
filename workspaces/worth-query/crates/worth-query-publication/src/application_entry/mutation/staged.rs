//! The one-shot parts of a request, moved out before its identities are encoded.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
    ApplicationMutationSourceExpectation,
};
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_query_installation::facade::ApplicationSchema;

use super::request::{
    WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryMutationExpectedSource,
};

/// A request's preconditions and source expectation, each usable once.
///
/// The request's identities borrow its key and input, so preparation cannot also
/// borrow the request mutably to take these. An entry point stages them first,
/// encodes the identities once, then prepares with shared borrows only.
pub(in crate::application_entry) struct WorthQueryStagedMutation<Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(super) preconditions: TypedMutationPreconditions<
        Schema,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
        <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
    >,
    pub(super) source: Option<
        WorthQueryMutationExpectedSource<
            <<Intent::Binding as ApplicationMutationBinding<Schema>>::SourceExpectation as ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
    >,
}

impl<Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        '_,
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    /// Moves the one-shot preconditions and source expectation out of the request.
    pub(in crate::application_entry) fn stage(
        &mut self,
    ) -> WorthQueryStagedMutation<Schema, Intent> {
        WorthQueryStagedMutation {
            preconditions: std::mem::take(&mut self.request.preconditions),
            source: self.request.source.take(),
        }
    }
}
