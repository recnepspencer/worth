//! Capture complete original meaning once through the public ordinary entrance.
use worth_query_decl::facade::{application_operation::*, application_schema::*};
use worth_query_host::facade::{application_entry::*, primary_graph::WorthQueryObservedResultSet};

// compiled-example: begin
pub fn capture_original<'a, 'p, 's, S, I>(
    request: &WorthQueryApplicationRequest<'a, 'p, 's, S>,
    intent: I,
    preconditions: TypedMutationPreconditions<S, <I::Binding as ApplicationMutationBinding<S>>::Operation,
        <<I::Binding as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope>,
    source: WorthQueryObservedResultSet<<<I::Binding as ApplicationMutationBinding<S>>::SourceExpectation as ApplicationMutationSourceExpectation<S>>::Query>,
    key: <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
) -> Result<
    WorthQueryOperationDescription<S, I>,
    (
        WorthQueryOperationBuilder<'a, 'p, 's, S, I, WorthQueryMutationSourcePrepared>,
        <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
        WorthQueryOperationInputCaptureDenial,
    ),
>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
{
    request
        .mutate(intent)
        .preconditions(preconditions)
        .expect_result_set(source)
        .idempotency_owned(key)
}
// compiled-example: end
