//! Authored input selection for a mutation binding already chosen by its owner.
use super::{ApplicationMutationBinding, ApplicationMutationIntent};
use crate::application_schema::ApplicationSchema;

/// Selects the declared scope of `Binding` from its exact input type.
///
/// This returns authored field/value selection, never a resolved identity,
/// permission, currentness witness or execution authority. The runtime must
/// still admit the installed binding, principal, scope and source.
/// Ordinary mutation dispatch continues to use [`ApplicationMutationIntent`].
///
/// Author explicit associations for the concrete installed schema. Generic
/// downstream schema impls can overlap the blanket under Rust's coherence
/// rules, including when the input currently has no ordinary intent.
#[doc = include_str!("input_scope_contract.md")]
pub trait ApplicationMutationInputScope<Schema, Binding>: Sized
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema, Input = Self>,
{
    fn scope_for_binding(&self) -> Binding::ScopeBinding;
}

/// An ordinary intent input keeps its existing single scope expression.
impl<Schema, Binding, Input> ApplicationMutationInputScope<Schema, Binding> for Input
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema, Input = Input>,
    Input: ApplicationMutationIntent<Schema, Binding = Binding>,
{
    fn scope_for_binding(&self) -> Binding::ScopeBinding {
        <Self as ApplicationMutationIntent<Schema>>::scope_binding(self)
    }
}
