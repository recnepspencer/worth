//! Named bound sets for generic helpers that submit and drive source-only custody.
//!
//! Each trait restates exactly the requirements of the native owners below and
//! is implemented for every qualifying type. None grants admission, execution
//! or cleanup authority; every request still passes its native owner.
use super::WorthQuerySourceOperationBinding;
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
        NoApplicationMutationSource,
    },
    application_program::{
        ApplicationAction, ApplicationActionDelivery, ApplicationSourceCompletion,
    },
};
use worth_query_installation::facade::ApplicationSchema;

/// A mutation binding whose description can be captured, durably retained and
/// advanced as a source-only operation.
pub trait WorthQuerySourceMutationBinding<S: ApplicationSchema>:
    WorthQuerySourceOperationBinding<S>
    + ApplicationMutationBinding<
        S,
        Input: serde::Serialize + Clone + Send + Sync,
        IdempotencyKey: serde::Serialize,
        ScopeBinding: ApplicationMutationScopeResolution<
            S,
            <Self as ApplicationMutationBinding<S>>::PrincipalIdentity,
        >,
    > + 'static
{
}
impl<S, B> WorthQuerySourceMutationBinding<S> for B
where
    S: ApplicationSchema,
    B: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<
            S,
            Input: serde::Serialize + Clone + Send + Sync,
            IdempotencyKey: serde::Serialize,
            ScopeBinding: ApplicationMutationScopeResolution<
                S,
                <B as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
        > + 'static,
{
}

/// A mutation intent whose binding drives source-only custody. Generic helpers
/// write `I: WorthQuerySourceMutation<S>` instead of restating the binding's
/// input, key, scope-resolution and native driver bounds.
pub trait WorthQuerySourceMutation<S: ApplicationSchema>:
    ApplicationMutationIntent<S, Binding: WorthQuerySourceMutationBinding<S>> + 'static
{
}
impl<S, I> WorthQuerySourceMutation<S> for I
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S, Binding: WorthQuerySourceMutationBinding<S>> + 'static,
{
}

/// A source mutation submitted without an observed source, as
/// `request.mutate(intent).without_source()` requires.
pub trait WorthQuerySourcelessMutation<S: ApplicationSchema>:
    WorthQuerySourceMutation<S>
    + ApplicationMutationIntent<
        S,
        Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    >
{
}
impl<S, I> WorthQuerySourcelessMutation<S> for I
where
    S: ApplicationSchema,
    I: WorthQuerySourceMutation<S>
        + ApplicationMutationIntent<
            S,
            Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
        >,
{
}

/// An installed action for binding `B` whose declared delivery completes the
/// source operation itself, so retention yields a
/// [`super::WorthQuerySourceOnlyOperation`] in source-completion mode.
pub trait WorthQuerySourceCompletionAction<S: ApplicationSchema, B: ApplicationMutationBinding<S>>:
    ApplicationAction<
    S,
    B,
    Delivery: ApplicationActionDelivery<S, B, Mode = ApplicationSourceCompletion>,
>
{
}
impl<S, B, A> WorthQuerySourceCompletionAction<S, B> for A
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    A: ApplicationAction<
        S,
        B,
        Delivery: ApplicationActionDelivery<S, B, Mode = ApplicationSourceCompletion>,
    >,
{
}
