//! Borrow the genuine preparation owners together, without replacing them by copied scope data.
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationScopeBinding},
    application_schema::ApplicationSchema,
};
use {
    crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding,
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation,
    crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation,
};

pub(super) type Scope<S, M> = <<M as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope;

/// One request-local native preparation. Construction carries no permission of
/// its own: the document validates the original admission and exact selection.
pub struct WorthQueryDocumentHistoryOperation<
    'borrow,
    'runtime,
    S: ApplicationSchema,
    M: ApplicationMutationBinding<S>,
> {
    pub(super) selected: &'borrow WorthQuerySelectedProductOperation<'runtime, S>,
    pub(super) admitted:
        &'borrow WorthQueryAdmittedApplicationOperation<S, M::Operation, M::Input, Scope<S, M>>,
    pub(super) binding: WorthQueryApplicationIdempotencyBinding,
}

impl<'borrow, 'runtime, S: ApplicationSchema, M: ApplicationMutationBinding<S>>
    WorthQueryDocumentHistoryOperation<'borrow, 'runtime, S, M>
{
    pub fn new(
        selected: &'borrow WorthQuerySelectedProductOperation<'runtime, S>,
        admitted: &'borrow WorthQueryAdmittedApplicationOperation<
            S,
            M::Operation,
            M::Input,
            Scope<S, M>,
        >,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Self {
        Self {
            selected,
            admitted,
            binding,
        }
    }
}
