//! A finite installed connection component, ordered by authentic input occurrences.
use super::*;

pub struct ApplicationOutputOccurrences<Connections>(PhantomData<fn() -> Connections>);
mod tuple_seal {
    pub trait Tuple {}
}
/// Closed tuple of typed ports. Arbitrary runtime connection lists cannot implement it.
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
                $(out.push($c::declaration().in_occurrence_component());)+
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
impl<C> sealed::OutputEdgesShape for ApplicationOutputOccurrences<C> {}
impl<S: ApplicationSchema, C: ApplicationOccurrenceConnections<S>> ApplicationOutputEdgesShape<S>
    for ApplicationOutputOccurrences<C>
{
    fn append_connections(out: &mut Vec<ApplicationConnectionDeclaration>) {
        C::append(out);
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
