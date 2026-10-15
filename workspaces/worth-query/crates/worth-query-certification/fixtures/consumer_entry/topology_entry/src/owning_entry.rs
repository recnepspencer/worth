//! Capture complete original meaning once through the public ordinary entrance.
use worth_query_decl::facade::{application_operation::*, application_schema::*};
use worth_query_host::facade::{application_entry::*, primary_graph::WorthQueryObservedResultSet};

// compiled-example: begin
pub fn capture_original<S, I>(
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    intent: I,
    preconditions: TypedMutationPreconditions<S, <I::Binding as ApplicationMutationBinding<S>>::Operation,
        <<I::Binding as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope>,
    source: WorthQueryObservedResultSet<<<I::Binding as ApplicationMutationBinding<S>>::SourceExpectation as ApplicationMutationSourceExpectation<S>>::Query>,
    key: <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
) -> WorthQueryOperationDescription<S, I>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone,
{
    request
        .mutate(intent)
        .preconditions(preconditions)
        .expect_result_set(source)
        .idempotency_owned(key)
}
// compiled-example: end
