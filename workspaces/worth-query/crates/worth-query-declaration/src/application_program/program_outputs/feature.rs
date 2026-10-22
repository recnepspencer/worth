//! Feature-selected roots are deduplicated before their graphs are flattened.
use super::*;
use crate::application_operation::ApplicationMutationBinding;
use crate::application_program::{ApplicationActionDelivery, ApplicationActionOutputRoots};
use std::any::TypeId;

pub struct ApplicationActionOutput<Binding, Delivery>(PhantomData<fn() -> (Binding, Delivery)>);
pub struct ApplicationFeatureOutputs<Members>(PhantomData<fn() -> Members>);

mod members {
    pub trait Sealed {}
}

/// A closed typed action roster, visited in authoring order without TypeId ordering.
pub trait ApplicationFeatureOutputMembers<S: ApplicationSchema>: members::Sealed + 'static {
    fn append_distinct(
        seen: &mut Vec<TypeId>,
        connections: &mut Vec<ApplicationConnectionDeclaration>,
        connection_types: &mut Vec<TypeId>,
        roots: &mut Vec<TypeId>,
    );
}
impl members::Sealed for ApplicationNoOutputGraph {}
impl<S: ApplicationSchema> ApplicationFeatureOutputMembers<S> for ApplicationNoOutputGraph {
    fn append_distinct(
        _: &mut Vec<TypeId>,
        _: &mut Vec<ApplicationConnectionDeclaration>,
        _: &mut Vec<TypeId>,
        _: &mut Vec<TypeId>,
    ) {
    }
}
impl<B, D> members::Sealed for ApplicationActionOutput<B, D> {}
impl<S, B, D> ApplicationFeatureOutputMembers<S> for ApplicationActionOutput<B, D>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    D: ApplicationActionDelivery<S, B>,
    D::Mode: ApplicationActionOutputRoots<S>,
{
    fn append_distinct(
        seen: &mut Vec<TypeId>,
        connections: &mut Vec<ApplicationConnectionDeclaration>,
        connection_types: &mut Vec<TypeId>,
        roots: &mut Vec<TypeId>,
    ) {
        type Roots<S, B, D> = <<D as ApplicationActionDelivery<S, B>>::Mode as ApplicationActionOutputRoots<S>>::Roots;
        let identity = TypeId::of::<Roots<S, B, D>>();
        if seen.contains(&identity) {
            return;
        }
        seen.push(identity);
        Roots::<S, B, D>::append_connections(connections);
        Roots::<S, B, D>::append_connection_types(connection_types);
        Roots::<S, B, D>::append_root_graph_types(roots);
    }
}
impl<A, B> members::Sealed for (A, B) {}
impl<S, A, B> ApplicationFeatureOutputMembers<S> for (A, B)
where
    S: ApplicationSchema,
    A: ApplicationFeatureOutputMembers<S>,
    B: ApplicationFeatureOutputMembers<S>,
{
    fn append_distinct(
        seen: &mut Vec<TypeId>,
        connections: &mut Vec<ApplicationConnectionDeclaration>,
        connection_types: &mut Vec<TypeId>,
        roots: &mut Vec<TypeId>,
    ) {
        A::append_distinct(seen, connections, connection_types, roots);
        B::append_distinct(seen, connections, connection_types, roots);
    }
}
impl<M> super::sealed::ProgramOutputRootsShape for ApplicationFeatureOutputs<M> {}
impl<S, M> ApplicationProgramOutputRootsShape<S> for ApplicationFeatureOutputs<M>
where
    S: ApplicationSchema,
    M: ApplicationFeatureOutputMembers<S>,
{
    fn append_connections(connections: &mut Vec<ApplicationConnectionDeclaration>) {
        M::append_distinct(
            &mut Vec::new(),
            connections,
            &mut Vec::new(),
            &mut Vec::new(),
        );
    }
    fn append_connection_types(connection_types: &mut Vec<TypeId>) {
        M::append_distinct(
            &mut Vec::new(),
            &mut Vec::new(),
            connection_types,
            &mut Vec::new(),
        );
    }
    fn append_root_graph_types(roots: &mut Vec<TypeId>) {
        M::append_distinct(&mut Vec::new(), &mut Vec::new(), &mut Vec::new(), roots);
    }
}
