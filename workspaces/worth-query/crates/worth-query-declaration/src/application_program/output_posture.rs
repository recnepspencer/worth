use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationConnectionDeclaration, ApplicationConnectionShape, ApplicationDiscoveredOutputGraph,
    ApplicationOutputChildrenShape, ApplicationOutputGraph,
};

/// Explicit posture for an application program with no managed required-output
/// graph. Ordinary actions and queries still belong to the program; callers
/// simply cannot request output-demand progression from this shape.
pub struct ApplicationNoOutputGraph;

mod sealed {
    pub trait ProgramOutputShape {}
}

/// Sealed program output posture. Only declaration-owned output shapes can
/// determine installation behavior.
pub trait ApplicationProgramOutputShape<Schema>:
    sealed::ProgramOutputShape + Sized + 'static
where
    Schema: ApplicationSchema,
{
    fn connections() -> Vec<ApplicationConnectionDeclaration>;
    fn connection_types() -> Vec<std::any::TypeId>;
}

impl sealed::ProgramOutputShape for ApplicationNoOutputGraph {}

impl<Schema> ApplicationProgramOutputShape<Schema> for ApplicationNoOutputGraph
where
    Schema: ApplicationSchema,
{
    fn connections() -> Vec<ApplicationConnectionDeclaration> {
        Vec::new()
    }

    fn connection_types() -> Vec<std::any::TypeId> {
        Vec::new()
    }
}

impl<RootConnection, Dependents> sealed::ProgramOutputShape
    for ApplicationOutputGraph<RootConnection, Dependents>
{
}

impl<RootConnection, Dependents> sealed::ProgramOutputShape
    for ApplicationDiscoveredOutputGraph<RootConnection, Dependents>
{
}

impl<Schema, RootConnection, Dependents> ApplicationProgramOutputShape<Schema>
    for ApplicationOutputGraph<RootConnection, Dependents>
where
    Schema: ApplicationSchema,
    RootConnection: ApplicationConnectionShape<Schema>,
    Dependents: ApplicationOutputChildrenShape<Schema, RootConnection::TargetFeature>,
{
    fn connections() -> Vec<ApplicationConnectionDeclaration> {
        let mut connections = vec![RootConnection::declaration()];
        Dependents::append_connections(&mut connections);
        connections
    }

    fn connection_types() -> Vec<std::any::TypeId> {
        let mut connections = vec![std::any::TypeId::of::<RootConnection>()];
        Dependents::append_connection_types(&mut connections);
        connections
    }
}

impl<Schema, RootConnection, Dependents> ApplicationProgramOutputShape<Schema>
    for ApplicationDiscoveredOutputGraph<RootConnection, Dependents>
where
    Schema: ApplicationSchema,
    RootConnection: ApplicationConnectionShape<Schema>,
    Dependents: ApplicationOutputChildrenShape<Schema, RootConnection::TargetFeature>,
{
    fn connections() -> Vec<ApplicationConnectionDeclaration> {
        let mut connections = vec![RootConnection::declaration()];
        Dependents::append_connections(&mut connections);
        connections
    }

    fn connection_types() -> Vec<std::any::TypeId> {
        let mut connections = vec![std::any::TypeId::of::<RootConnection>()];
        Dependents::append_connection_types(&mut connections);
        connections
    }
}
