use std::marker::PhantomData;

use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationConnectionDeclaration, ApplicationConnectionIdentity,
    ApplicationConnectionInstanceRef, ApplicationConnectionRef, ApplicationFeature,
    ApplicationInputPort, ApplicationOccurrenceConnectionBinding, ApplicationOutputPort,
};

/// Typed shape of the required-output graph rooted at one performed action.
pub struct ApplicationOutputGraph<RootConnection, Dependents> {
    marker: PhantomData<fn() -> (RootConnection, Dependents)>,
}

/// A performed action whose required root outputs are discovered at its retained commit.
pub struct ApplicationDiscoveredOutputGraph<RootConnection, Dependents> {
    marker: PhantomData<fn() -> (RootConnection, Dependents)>,
}

/// One transitive output edge and the edges that consume each of its outputs.
pub struct ApplicationOutputEdge<Connection, Dependents> {
    marker: PhantomData<fn() -> (Connection, Dependents)>,
}

/// Explicit terminal posture for an output with no further consumers.
pub struct ApplicationOutputLeaf;

mod sealed {
    pub trait ConnectionShape {}
    pub trait OutputEdgesShape {}
    pub trait OutputChildrenShape<ParentFeature> {}
    pub trait RequiredRootKind {}
    pub trait DiscoveredRootKind {}
}

mod root_kinds;
pub use root_kinds::{ApplicationDiscoveredOutputRoot, ApplicationRequiredOutputRoot};

/// A fully typed authored connection that lowers without a repeated runtime
/// connection inventory.
pub trait ApplicationConnectionShape<Schema>: sealed::ConnectionShape + Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Binding: ApplicationConnectionIdentity;
    type SourceFeature: ApplicationFeature<Schema>;
    type SourcePort: ApplicationOutputPort<Schema, Self::SourceFeature>;
    type TargetFeature: ApplicationFeature<Schema>;
    type TargetPort: ApplicationInputPort<
        Schema,
        Self::TargetFeature,
        Value = <Self::SourcePort as ApplicationOutputPort<Schema, Self::SourceFeature>>::Value,
    >;

    fn declaration() -> ApplicationConnectionDeclaration;
}

impl<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding>
    ApplicationConnectionShape<Schema>
    for ApplicationConnectionRef<
        Schema,
        SourceFeature,
        SourcePort,
        TargetFeature,
        TargetPort,
        Binding,
    >
where
    Schema: ApplicationSchema + 'static,
    SourceFeature: ApplicationFeature<Schema>,
    TargetFeature: ApplicationFeature<Schema>,
    SourcePort: ApplicationOutputPort<Schema, SourceFeature>,
    TargetPort: ApplicationInputPort<
        Schema,
        TargetFeature,
        Value = <SourcePort as ApplicationOutputPort<Schema, SourceFeature>>::Value,
    >,
    Binding: ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>,
{
    type Binding = Binding;
    type SourceFeature = SourceFeature;
    type SourcePort = SourcePort;
    type TargetFeature = TargetFeature;
    type TargetPort = TargetPort;

    fn declaration() -> ApplicationConnectionDeclaration {
        Self::declaration()
    }
}

impl<
        Schema,
        SourceInstance,
        SourceFeature,
        SourcePort,
        TargetInstance,
        TargetFeature,
        TargetPort,
        Binding,
    > ApplicationConnectionShape<Schema>
    for ApplicationConnectionInstanceRef<
        Schema,
        SourceInstance,
        SourceFeature,
        SourcePort,
        TargetInstance,
        TargetFeature,
        TargetPort,
        Binding,
    >
where
    Schema: ApplicationSchema + 'static,
    SourceInstance: super::ApplicationCompositionInstance,
    TargetInstance: super::ApplicationCompositionInstance,
    SourceFeature: ApplicationFeature<Schema>,
    TargetFeature: ApplicationFeature<Schema>,
    SourcePort: ApplicationOutputPort<Schema, SourceFeature>,
    TargetPort: ApplicationInputPort<
        Schema,
        TargetFeature,
        Value = <SourcePort as ApplicationOutputPort<Schema, SourceFeature>>::Value,
    >,
    Binding: ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>,
{
    type Binding = Binding;
    type SourceFeature = SourceFeature;
    type SourcePort = SourcePort;
    type TargetFeature = TargetFeature;
    type TargetPort = TargetPort;

    fn declaration() -> ApplicationConnectionDeclaration {
        Self::declaration()
    }
}

impl<Schema, SourceFeature, SourcePort, TargetFeature, TargetPort, Binding> sealed::ConnectionShape
    for ApplicationConnectionRef<
        Schema,
        SourceFeature,
        SourcePort,
        TargetFeature,
        TargetPort,
        Binding,
    >
{
}

impl<
        Schema,
        SourceInstance,
        SourceFeature,
        SourcePort,
        TargetInstance,
        TargetFeature,
        TargetPort,
        Binding,
    > sealed::ConnectionShape
    for ApplicationConnectionInstanceRef<
        Schema,
        SourceInstance,
        SourceFeature,
        SourcePort,
        TargetInstance,
        TargetFeature,
        TargetPort,
        Binding,
    >
{
}

pub trait ApplicationOutputGraphShape<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type RootConnection: ApplicationConnectionShape<Schema>;
    type Dependents: ApplicationOutputEdgesShape<Schema>;

    fn connections() -> Vec<ApplicationConnectionDeclaration> {
        let mut connections = vec![Self::RootConnection::declaration()];
        Self::Dependents::append_connections(&mut connections);
        connections
    }

    fn connection_types() -> Vec<std::any::TypeId> {
        let mut connections = vec![std::any::TypeId::of::<Self::RootConnection>()];
        Self::Dependents::append_connection_types(&mut connections);
        connections
    }
}

impl<Schema, RootConnection, Dependents> ApplicationOutputGraphShape<Schema>
    for ApplicationOutputGraph<RootConnection, Dependents>
where
    Schema: ApplicationSchema,
    RootConnection: ApplicationConnectionShape<Schema>,
    Dependents: ApplicationOutputChildrenShape<Schema, RootConnection::TargetFeature>,
{
    type RootConnection = RootConnection;
    type Dependents = Dependents;
}

impl<Schema, RootConnection, Dependents> ApplicationOutputGraphShape<Schema>
    for ApplicationDiscoveredOutputGraph<RootConnection, Dependents>
where
    Schema: ApplicationSchema,
    RootConnection: ApplicationConnectionShape<Schema>,
    Dependents: ApplicationOutputChildrenShape<Schema, RootConnection::TargetFeature>,
{
    type RootConnection = RootConnection;
    type Dependents = Dependents;
}

pub trait ApplicationOutputEdgesShape<Schema>: sealed::OutputEdgesShape + Sized + 'static
where
    Schema: ApplicationSchema,
{
    fn append_connections(connections: &mut Vec<ApplicationConnectionDeclaration>);
    fn append_connection_types(connections: &mut Vec<std::any::TypeId>);
}

/// Sealed adjacency proof for every edge leaving one feature.
///
/// A child whose source feature is not the parent's target feature cannot be
/// admitted as an output graph edge:
///
/// ```compile_fail
/// use worth_query_declaration::facade::{
///     application_program::{
///         ApplicationConnectionShape, ApplicationFeature, ApplicationOutputChildrenShape,
///         ApplicationOutputEdge, ApplicationOutputLeaf,
///     },
///     application_schema::ApplicationSchema,
/// };
/// fn mismatched_child<Schema, Parent, Child>()
/// where
///     Schema: ApplicationSchema,
///     Parent: ApplicationFeature<Schema>,
///     Child: ApplicationConnectionShape<Schema>,
/// {
///     fn require<S, P, C>()
///     where
///         S: ApplicationSchema,
///         P: ApplicationFeature<S>,
///         C: ApplicationOutputChildrenShape<S, P>,
///     {}
///     require::<Schema, Parent, ApplicationOutputEdge<Child, ApplicationOutputLeaf>>();
/// }
/// ```
///
/// Supplying the source-feature equality is the valid counterpart:
///
/// ```
/// use worth_query_declaration::facade::{
///     application_program::{
///         ApplicationConnectionShape, ApplicationFeature, ApplicationOutputChildrenShape,
///         ApplicationOutputEdge, ApplicationOutputLeaf,
///     },
///     application_schema::ApplicationSchema,
/// };
/// fn adjacent_child<Schema, Parent, Child>()
/// where
///     Schema: ApplicationSchema,
///     Parent: ApplicationFeature<Schema>,
///     Child: ApplicationConnectionShape<Schema, SourceFeature = Parent>,
/// {
///     fn require<S, P, C>()
///     where
///         S: ApplicationSchema,
///         P: ApplicationFeature<S>,
///         C: ApplicationOutputChildrenShape<S, P>,
///     {}
///     require::<Schema, Parent, ApplicationOutputEdge<Child, ApplicationOutputLeaf>>();
/// }
/// ```
pub trait ApplicationOutputChildrenShape<Schema, ParentFeature>:
    ApplicationOutputEdgesShape<Schema> + sealed::OutputChildrenShape<ParentFeature>
where
    Schema: ApplicationSchema,
    ParentFeature: ApplicationFeature<Schema>,
{
}

pub trait ApplicationOutputEdgeShape<Schema>: ApplicationOutputEdgesShape<Schema>
where
    Schema: ApplicationSchema,
{
    type Connection: ApplicationConnectionShape<Schema>;
    type Dependents: ApplicationOutputEdgesShape<Schema>;
}

impl<Schema> ApplicationOutputEdgesShape<Schema> for ApplicationOutputLeaf
where
    Schema: ApplicationSchema,
{
    fn append_connections(_: &mut Vec<ApplicationConnectionDeclaration>) {}
    fn append_connection_types(_: &mut Vec<std::any::TypeId>) {}
}

impl sealed::OutputEdgesShape for ApplicationOutputLeaf {}

impl<Schema, ParentFeature> ApplicationOutputChildrenShape<Schema, ParentFeature>
    for ApplicationOutputLeaf
where
    Schema: ApplicationSchema,
    ParentFeature: ApplicationFeature<Schema>,
{
}

impl<ParentFeature> sealed::OutputChildrenShape<ParentFeature> for ApplicationOutputLeaf {}

impl<Schema, Connection, Dependents> ApplicationOutputEdgesShape<Schema>
    for ApplicationOutputEdge<Connection, Dependents>
where
    Schema: ApplicationSchema,
    Connection: ApplicationConnectionShape<Schema>,
    Dependents: ApplicationOutputEdgesShape<Schema>,
{
    fn append_connections(connections: &mut Vec<ApplicationConnectionDeclaration>) {
        connections.push(Connection::declaration());
        Dependents::append_connections(connections);
    }

    fn append_connection_types(connections: &mut Vec<std::any::TypeId>) {
        connections.push(std::any::TypeId::of::<Connection>());
        Dependents::append_connection_types(connections);
    }
}

impl<Connection, Dependents> sealed::OutputEdgesShape
    for ApplicationOutputEdge<Connection, Dependents>
{
}

impl<Schema, ParentFeature, Connection, Dependents>
    ApplicationOutputChildrenShape<Schema, ParentFeature>
    for ApplicationOutputEdge<Connection, Dependents>
where
    Schema: ApplicationSchema,
    ParentFeature: ApplicationFeature<Schema>,
    Connection: ApplicationConnectionShape<Schema, SourceFeature = ParentFeature>,
    Dependents: ApplicationOutputChildrenShape<Schema, Connection::TargetFeature>,
{
}

impl<ParentFeature, Connection, Dependents> sealed::OutputChildrenShape<ParentFeature>
    for ApplicationOutputEdge<Connection, Dependents>
{
}

impl<Schema, Connection, Dependents> ApplicationOutputEdgeShape<Schema>
    for ApplicationOutputEdge<Connection, Dependents>
where
    Schema: ApplicationSchema,
    Connection: ApplicationConnectionShape<Schema>,
    Dependents: ApplicationOutputEdgesShape<Schema>,
{
    type Connection = Connection;
    type Dependents = Dependents;
}

impl<Schema, Left, Right> ApplicationOutputEdgesShape<Schema> for (Left, Right)
where
    Schema: ApplicationSchema,
    Left: ApplicationOutputEdgesShape<Schema>,
    Right: ApplicationOutputEdgesShape<Schema>,
{
    fn append_connections(connections: &mut Vec<ApplicationConnectionDeclaration>) {
        Left::append_connections(connections);
        Right::append_connections(connections);
    }

    fn append_connection_types(connections: &mut Vec<std::any::TypeId>) {
        Left::append_connection_types(connections);
        Right::append_connection_types(connections);
    }
}

impl<Left, Right> sealed::OutputEdgesShape for (Left, Right) {}

impl<Schema, ParentFeature, Left, Right> ApplicationOutputChildrenShape<Schema, ParentFeature>
    for (Left, Right)
where
    Schema: ApplicationSchema,
    ParentFeature: ApplicationFeature<Schema>,
    Left: ApplicationOutputChildrenShape<Schema, ParentFeature>,
    Right: ApplicationOutputChildrenShape<Schema, ParentFeature>,
{
}

impl<ParentFeature, Left, Right> sealed::OutputChildrenShape<ParentFeature> for (Left, Right) {}
