//! A finite installed connection component, ordered by authentic input occurrences.
use super::*;

pub struct ApplicationOutputOccurrences<Connections>(PhantomData<fn() -> Connections>);
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
impl<C> sealed::OutputEdgesShape for ApplicationOutputOccurrences<C> {}
impl<S: ApplicationSchema, C: ApplicationOccurrenceConnections<S>> ApplicationOutputEdgesShape<S>
    for ApplicationOutputOccurrences<C>
{
    fn append_connections(out: &mut Vec<ApplicationConnectionDeclaration>) {
        let start = out.len();
        C::append(out);
        for connection in &mut out[start..] {
            connection.set_occurrence_component(std::any::TypeId::of::<C>());
        }
    }
    fn append_connection_types(out: &mut Vec<std::any::TypeId>) {
        C::append_types(out);
    }
}
impl<F, C> sealed::OutputChildrenShape<F> for ApplicationOutputOccurrences<C> {}
impl<S: ApplicationSchema, F: ApplicationFeature<S>, C: ApplicationOccurrenceConnections<S>>
    ApplicationOutputChildrenShape<S, F> for ApplicationOutputOccurrences<C>
{
}
