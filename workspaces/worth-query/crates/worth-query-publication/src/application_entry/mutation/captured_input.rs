//! Fresh admission of an exact encoded input through its binding's authored scope.
use std::marker::PhantomData;
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationEncodedInput, ApplicationMutationBinding, ApplicationMutationInputScope,
        ApplicationMutationIntent,
    },
    application_schema::ApplicationSchema,
};

/// An immutable captured input associated with its exact installed mutation binding.
///
/// Correction delivery owners name this carrier when several bindings share one
/// input codec. It borrows its input and authored scope from the original encoded
/// custody; it carries no principal, admission, or replacement-input constructor.
pub struct WorthQueryCapturedMutationInput<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    input: ApplicationEncodedInput<Binding::InputBinding>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema: ApplicationSchema, Binding: ApplicationMutationBinding<Schema>>
    WorthQueryCapturedMutationInput<Schema, Binding>
{
    pub(in crate::application_entry) fn new(
        input: ApplicationEncodedInput<Binding::InputBinding>,
    ) -> Self {
        Self {
            input,
            marker: PhantomData,
        }
    }
}

impl<Schema, Binding> ApplicationMutationIntent<Schema>
    for WorthQueryCapturedMutationInput<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Binding::Input: ApplicationMutationInputScope<Schema, Binding>,
{
    type Binding = Binding;
    fn input(&self) -> &Binding::Input {
        self.input.input()
    }
    fn scope_binding(&self) -> Binding::ScopeBinding {
        self.input.input().scope_for_binding()
    }
}
