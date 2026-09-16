use std::marker::PhantomData;

use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationConnectionDeclaration, ApplicationOutputGraph, ApplicationOutputGraphShape,
};

/// Complete set of independently startable output trees in one program.
pub struct ApplicationProgramOutputs<Roots> {
    marker: PhantomData<fn() -> Roots>,
}

mod sealed {
    pub trait ProgramOutputRootsShape {}
    pub trait ProgramOutputsShape {}
}

/// One or more independently startable output trees.
pub trait ApplicationProgramOutputRootsShape<Schema>:
    sealed::ProgramOutputRootsShape + Sized + 'static
where
    Schema: ApplicationSchema,
{
    fn append_connections(connections: &mut Vec<ApplicationConnectionDeclaration>);
    fn append_connection_types(connections: &mut Vec<std::any::TypeId>);
    fn append_root_graph_types(roots: &mut Vec<std::any::TypeId>);
}

/// The complete output topology authored by one program.
pub trait ApplicationProgramOutputsShape<Schema>:
    sealed::ProgramOutputsShape + Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Roots: ApplicationProgramOutputRootsShape<Schema>;

    fn connections() -> Vec<ApplicationConnectionDeclaration> {
        let mut connections = Vec::new();
        Self::Roots::append_connections(&mut connections);
        connections
    }

    fn connection_types() -> Vec<std::any::TypeId> {
        let mut connections = Vec::new();
        Self::Roots::append_connection_types(&mut connections);
        connections
    }

    fn root_graph_types() -> Vec<std::any::TypeId> {
        let mut roots = Vec::new();
        Self::Roots::append_root_graph_types(&mut roots);
        roots
    }
}

impl<Roots> sealed::ProgramOutputsShape for ApplicationProgramOutputs<Roots> {}

impl<Schema, Roots> ApplicationProgramOutputsShape<Schema> for ApplicationProgramOutputs<Roots>
where
    Schema: ApplicationSchema,
    Roots: ApplicationProgramOutputRootsShape<Schema>,
{
    type Roots = Roots;
}

impl<RootConnection, Dependents> sealed::ProgramOutputRootsShape
    for ApplicationOutputGraph<RootConnection, Dependents>
{
}

impl<Schema, RootConnection, Dependents> ApplicationProgramOutputRootsShape<Schema>
    for ApplicationOutputGraph<RootConnection, Dependents>
where
    Schema: ApplicationSchema,
    Self: ApplicationOutputGraphShape<Schema>,
{
    fn append_connections(connections: &mut Vec<ApplicationConnectionDeclaration>) {
        connections.extend(<Self as ApplicationOutputGraphShape<Schema>>::connections());
    }

    fn append_connection_types(connections: &mut Vec<std::any::TypeId>) {
        connections.extend(<Self as ApplicationOutputGraphShape<Schema>>::connection_types());
    }

    fn append_root_graph_types(roots: &mut Vec<std::any::TypeId>) {
        roots.push(std::any::TypeId::of::<Self>());
    }
}

impl<Left, Right> sealed::ProgramOutputRootsShape for (Left, Right) {}

impl<Schema, Left, Right> ApplicationProgramOutputRootsShape<Schema> for (Left, Right)
where
    Schema: ApplicationSchema,
    Left: ApplicationProgramOutputRootsShape<Schema>,
    Right: ApplicationProgramOutputRootsShape<Schema>,
{
    fn append_connections(connections: &mut Vec<ApplicationConnectionDeclaration>) {
        Left::append_connections(connections);
        Right::append_connections(connections);
    }

    fn append_connection_types(connections: &mut Vec<std::any::TypeId>) {
        Left::append_connection_types(connections);
        Right::append_connection_types(connections);
    }

    fn append_root_graph_types(roots: &mut Vec<std::any::TypeId>) {
        Left::append_root_graph_types(roots);
        Right::append_root_graph_types(roots);
    }
}
