use std::marker::PhantomData;

use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationConnectionDeclaration, ApplicationConnectionIdentity,
    ApplicationConnectionInstanceRef, ApplicationConnectionRef, ApplicationFeature,
    ApplicationInputPort, ApplicationOccurrenceConnectionBinding, ApplicationOutputPort,
    ApplicationProgramOutputShape,
};

/// Typed shape of the required-output graph rooted at one performed action.
pub struct ApplicationOutputGraph<RootConnection, Dependents> {
    marker: PhantomData<fn() -> (RootConnection, Dependents)>,
}

/// An installed output tree evaluated only when explicitly demanded.
/// Saving its source does not require producing this tree.
pub struct ApplicationDemandOutputGraph<RootConnection, Dependents> {
    marker: PhantomData<fn() -> (RootConnection, Dependents)>,
}

/// A performed action whose roots are discovered from its authored result.
/// Runtime envelopes discover at the actual appended candidate; standalone
/// delivery retains the committed source until its discovered outputs settle.
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
    pub trait OutputEdgesShape<Schema> {
        fn append_typed(
            connections: &mut Vec<(std::any::TypeId, super::ApplicationConnectionDeclaration)>,
        );
    }
    pub trait OutputChildrenShape<ParentFeature> {}
    pub trait FixedRootKind {}
    pub trait RequiredRootKind {}
    pub trait DiscoveredRootKind {}
}

mod input_capture;
mod root_kinds;
pub use input_capture::{
    ApplicationCapturedInputConnection, ApplicationCapturedPrerequisites,
    ApplicationPrerequisiteSource, ApplicationStoredPrerequisites,
    WorthQueryProducerInputCaptureDefinition,
};
pub use root_kinds::{
    ApplicationDiscoveredOutputRoot, ApplicationFixedOutputRoot, ApplicationRequiredOutputRoot,
};

/// A fully typed authored connection that lowers without a repeated runtime
/// connection inventory.
pub trait ApplicationConnectionShape<Schema>: sealed::ConnectionShape + Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Binding: ApplicationConnectionIdentity;
    type Prerequisites: ApplicationPrerequisiteSource;
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

mod connections;

pub trait ApplicationOutputGraphShape<Schema>: ApplicationProgramOutputShape<Schema>
where
    Schema: ApplicationSchema,
{
    type RootConnection: ApplicationConnectionShape<Schema>;
    type Dependents: ApplicationOutputEdgesShape<Schema>;
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
    for ApplicationDemandOutputGraph<RootConnection, Dependents>
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

pub trait ApplicationOutputEdgesShape<Schema>:
    sealed::OutputEdgesShape<Schema> + Sized + 'static
where
    Schema: ApplicationSchema,
{
    fn append_connections(connections: &mut Vec<ApplicationConnectionDeclaration>) {
        let mut typed = Vec::new();
        Self::append_typed(&mut typed);
        connections.extend(typed.into_iter().map(|(_, declaration)| declaration));
    }
    fn append_connection_types(connections: &mut Vec<std::any::TypeId>) {
        let mut typed = Vec::new();
        Self::append_typed(&mut typed);
        connections.extend(typed.into_iter().map(|(identity, _)| identity));
    }
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

impl<Schema> ApplicationOutputEdgesShape<Schema> for ApplicationOutputLeaf where
    Schema: ApplicationSchema
{
}

impl<Schema> sealed::OutputEdgesShape<Schema> for ApplicationOutputLeaf {
    fn append_typed(_: &mut Vec<(std::any::TypeId, ApplicationConnectionDeclaration)>) {}
}

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
}

impl<Schema, Connection, Dependents> sealed::OutputEdgesShape<Schema>
    for ApplicationOutputEdge<Connection, Dependents>
where
    Schema: ApplicationSchema,
    Connection: ApplicationConnectionShape<Schema>,
    Dependents: ApplicationOutputEdgesShape<Schema>,
{
    fn append_typed(connections: &mut Vec<(std::any::TypeId, ApplicationConnectionDeclaration)>) {
        connections.push((
            std::any::TypeId::of::<Connection>(),
            Connection::declaration(),
        ));
        Dependents::append_typed(connections);
    }
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
}

impl<Schema, Left, Right> sealed::OutputEdgesShape<Schema> for (Left, Right)
where
    Schema: ApplicationSchema,
    Left: ApplicationOutputEdgesShape<Schema>,
    Right: ApplicationOutputEdgesShape<Schema>,
{
    fn append_typed(connections: &mut Vec<(std::any::TypeId, ApplicationConnectionDeclaration)>) {
        Left::append_typed(connections);
        Right::append_typed(connections);
    }
}

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

mod occurrences;
pub use occurrences::{
    ApplicationOccurrenceComponent, ApplicationOccurrenceConnections, ApplicationOccurrenceGroup,
    ApplicationOutputOccurrences,
};
