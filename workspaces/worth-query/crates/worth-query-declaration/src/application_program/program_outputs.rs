use std::marker::PhantomData;
mod feature;
pub use feature::{
    ApplicationActionOutput, ApplicationFeatureOutputMembers, ApplicationFeatureOutputs,
};

use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationConnectionDeclaration, ApplicationDemandOutputGraph,
    ApplicationDiscoveredOutputGraph, ApplicationNoOutputGraph, ApplicationOutputGraph,
    ApplicationOutputGraphShape, ApplicationProgramOutputShape,
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
    fn append_connections(
        connections: &mut Vec<(std::any::TypeId, ApplicationConnectionDeclaration)>,
    );
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
            .into_iter()
            .map(|(_, declaration)| declaration)
            .collect()
    }

    fn connection_types() -> Vec<std::any::TypeId> {
        let mut connections = Vec::new();
        Self::Roots::append_connections(&mut connections);
        connections
            .into_iter()
            .map(|(identity, _)| identity)
            .collect()
    }

    fn root_graph_types() -> Vec<std::any::TypeId> {
        let mut roots = Vec::new();
        Self::Roots::append_root_graph_types(&mut roots);
        roots
    }
}

impl<Roots> sealed::ProgramOutputsShape for ApplicationProgramOutputs<Roots> {}

impl sealed::ProgramOutputRootsShape for ApplicationNoOutputGraph {}

impl<Schema> ApplicationProgramOutputRootsShape<Schema> for ApplicationNoOutputGraph
where
    Schema: ApplicationSchema,
{
    fn append_connections(_: &mut Vec<(std::any::TypeId, ApplicationConnectionDeclaration)>) {}
    fn append_root_graph_types(_: &mut Vec<std::any::TypeId>) {}
}

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

impl<RootConnection, Dependents> sealed::ProgramOutputRootsShape
    for ApplicationDiscoveredOutputGraph<RootConnection, Dependents>
{
}

impl<Schema, RootConnection, Dependents> ApplicationProgramOutputRootsShape<Schema>
    for ApplicationOutputGraph<RootConnection, Dependents>
where
    Schema: ApplicationSchema,
    Self: ApplicationOutputGraphShape<Schema>,
{
    fn append_connections(
        connections: &mut Vec<(std::any::TypeId, ApplicationConnectionDeclaration)>,
    ) {
        append_graph::<Schema, Self>(connections);
    }

    fn append_root_graph_types(roots: &mut Vec<std::any::TypeId>) {
        roots.push(std::any::TypeId::of::<Self>());
    }
}

impl<Schema, RootConnection, Dependents> ApplicationProgramOutputRootsShape<Schema>
    for ApplicationDiscoveredOutputGraph<RootConnection, Dependents>
where
    Schema: ApplicationSchema,
    Self: ApplicationOutputGraphShape<Schema>,
{
    fn append_connections(
        connections: &mut Vec<(std::any::TypeId, ApplicationConnectionDeclaration)>,
    ) {
        append_graph::<Schema, Self>(connections);
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
    fn append_connections(
        connections: &mut Vec<(std::any::TypeId, ApplicationConnectionDeclaration)>,
    ) {
        Left::append_connections(connections);
        Right::append_connections(connections);
    }

    fn append_root_graph_types(roots: &mut Vec<std::any::TypeId>) {
        Left::append_root_graph_types(roots);
        Right::append_root_graph_types(roots);
    }
}

impl<RootConnection, Dependents> sealed::ProgramOutputRootsShape
    for ApplicationDemandOutputGraph<RootConnection, Dependents>
{
}

impl<Schema, RootConnection, Dependents> ApplicationProgramOutputRootsShape<Schema>
    for ApplicationDemandOutputGraph<RootConnection, Dependents>
where
    Schema: ApplicationSchema,
    Self: ApplicationOutputGraphShape<Schema>,
{
    fn append_connections(
        connections: &mut Vec<(std::any::TypeId, ApplicationConnectionDeclaration)>,
    ) {
        append_graph::<Schema, Self>(connections);
    }

    fn append_root_graph_types(roots: &mut Vec<std::any::TypeId>) {
        roots.push(std::any::TypeId::of::<Self>());
    }
}

/// Repeated roots share exact typed declarations, never merely textual IDs.
fn append_graph<S, G>(out: &mut Vec<(std::any::TypeId, ApplicationConnectionDeclaration)>)
where
    S: ApplicationSchema,
    G: ApplicationProgramOutputShape<S>,
{
    let previous_roots = out.len();
    let mut local = std::collections::BTreeSet::new();
    for entry in G::connection_types().into_iter().zip(G::connections()) {
        // A duplicate INSIDE one graph remains visible to ordinary validation.
        if !local.insert(entry.0) || !out[..previous_roots].contains(&entry) {
            out.push(entry);
        }
    }
}
