//! A finite installed connection component, ordered by authentic input occurrences.
use super::*;

/// The installed dependents of one closed, named occurrence component.
///
/// `Component` is the component's identity: every placement of the same
/// component names the same type, and its complete structural connection
/// vocabulary is expanded only from [`ApplicationOccurrenceComponent::Connections`].
pub struct ApplicationOutputOccurrences<Component>(PhantomData<fn() -> Component>);
/// A named, closed occurrence component. Its type is its identity.
///
/// Structural tuples and [`ApplicationOccurrenceGroup`]s compose the
/// component's connections but never name a component themselves, so a
/// component's full connection vocabulary is spelled once, at its declaration.
///
/// ```
/// use std::marker::PhantomData;
/// use worth_query_declaration::facade::{application_program::*, application_schema::ApplicationSchema};
/// fn edges<S: ApplicationSchema, E: ApplicationOutputEdgesShape<S>>() {}
/// struct Named<C>(PhantomData<fn() -> C>);
/// impl<S: ApplicationSchema, C: ApplicationConnectionShape<S> + 'static> ApplicationOccurrenceComponent<S>
///     for Named<C>
/// {
///     type Connections = (C,);
/// }
/// fn named<S: ApplicationSchema, C: ApplicationConnectionShape<S> + 'static>() {
///     edges::<S, ApplicationOutputOccurrences<Named<C>>>();
/// }
/// ```
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::{application_program::*, application_schema::ApplicationSchema};
/// fn edges<S: ApplicationSchema, E: ApplicationOutputEdgesShape<S>>() {}
/// fn structural<S: ApplicationSchema, C: ApplicationConnectionShape<S> + 'static>() {
///     edges::<S, ApplicationOutputOccurrences<(C,)>>();
/// }
/// ```
pub trait ApplicationOccurrenceComponent<S: ApplicationSchema>: 'static {
    type Connections: ApplicationOccurrenceConnections<S>;
}
/// Recursive composition inside one closed occurrence component.
pub struct ApplicationOccurrenceGroup<Left, Right>(PhantomData<fn() -> (Left, Right)>);
mod tuple_seal {
    pub trait Tuple {}
}
/// Closed typed ports. Arbitrary runtime connection lists cannot implement it.
pub trait ApplicationOccurrenceConnections<S: ApplicationSchema>:
    tuple_seal::Tuple + 'static
{
    fn append(connections: &mut Vec<ApplicationConnectionDeclaration>);
    fn append_types(connections: &mut Vec<std::any::TypeId>);
}
macro_rules! tuple {
    ($($c:ident),+) => {
        impl<$($c),+> tuple_seal::Tuple for ($($c,)+) {}
        impl<S: ApplicationSchema, $($c: ApplicationConnectionShape<S>),+> ApplicationOccurrenceConnections<S> for ($($c,)+) {
            fn append(out: &mut Vec<ApplicationConnectionDeclaration>) {
                $(out.push($c::declaration());)+
            }
            fn append_types(out: &mut Vec<std::any::TypeId>) { $(out.push(std::any::TypeId::of::<$c>());)+ }
        }
    }
}
tuple!(A);
tuple!(A, B);
tuple!(A, B, C);
tuple!(A, B, C, D);
tuple!(A, B, C, D, E);
tuple!(A, B, C, D, E, F);
tuple!(A, B, C, D, E, F, G);
tuple!(A, B, C, D, E, F, G, H);
impl<L, R> tuple_seal::Tuple for ApplicationOccurrenceGroup<L, R> {}
impl<S, L, R> ApplicationOccurrenceConnections<S> for ApplicationOccurrenceGroup<L, R>
where
    S: ApplicationSchema,
    L: ApplicationOccurrenceConnections<S>,
    R: ApplicationOccurrenceConnections<S>,
{
    fn append(out: &mut Vec<ApplicationConnectionDeclaration>) {
        L::append(out);
        R::append(out);
    }
    fn append_types(out: &mut Vec<std::any::TypeId>) {
        L::append_types(out);
        R::append_types(out);
    }
}
impl<S: ApplicationSchema, C: ApplicationOccurrenceComponent<S>> ApplicationOutputEdgesShape<S>
    for ApplicationOutputOccurrences<C>
{
}
impl<S: ApplicationSchema, C: ApplicationOccurrenceComponent<S>> sealed::OutputEdgesShape<S>
    for ApplicationOutputOccurrences<C>
{
    fn append_typed(out: &mut Vec<(std::any::TypeId, ApplicationConnectionDeclaration)>) {
        let component = std::any::TypeId::of::<C>();
        if out
            .iter()
            .any(|(_, declaration)| declaration.occurrence_component() == Some(component))
        {
            return;
        }
        // Preserve every first-expansion member, including invalid duplicates.
        // Later placements reuse C's declaration, not its runtime continuation.
        let mut declarations = Vec::new();
        let mut identities = Vec::new();
        C::Connections::append(&mut declarations);
        C::Connections::append_types(&mut identities);
        for declaration in &mut declarations {
            declaration.set_occurrence_component(component);
        }
        out.extend(identities.into_iter().zip(declarations));
    }
}
impl<F, C> sealed::OutputChildrenShape<F> for ApplicationOutputOccurrences<C> {}
impl<S: ApplicationSchema, F: ApplicationFeature<S>, C: ApplicationOccurrenceComponent<S>>
    ApplicationOutputChildrenShape<S, F> for ApplicationOutputOccurrences<C>
{
}
