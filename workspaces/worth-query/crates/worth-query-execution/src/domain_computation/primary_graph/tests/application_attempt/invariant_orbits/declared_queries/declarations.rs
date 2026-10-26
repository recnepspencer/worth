//! Authored whole-orbit queries over the same real static schema profiles.
use super::super::schema::*;
use crate::domain_computation::primary_graph::*;
use worth_query_declaration::{
    facade::{application_query::*, application_schema::*},
    *,
};

#[derive(Clone, Debug)]
pub struct Parameters;
worth_query_portable_type!(Parameters => "worth.query.declared-orbit.parameters.v1");
worth_query_structured_value_binding!(pub ParametersBinding for Parameters {identity: "worth.query.declared-orbit.parameters.v1"});
#[derive(Debug)]
pub struct NodeRow {
    pub key: u64,
}
worth_query_portable_type!(NodeRow => "worth.query.declared-orbit.row.v1");
worth_query_structured_value_binding!(pub RowsBinding for NodeRow {identity: "worth.query.declared-orbit.row.v1"});
pub struct KeySlot;
pub struct OrbitSlot;
pub struct OtherSlot;
worth_query_portable_type!(KeySlot => "worth.query.declared-orbit.key.v1");
worth_query_portable_type!(OrbitSlot => "worth.query.declared-orbit.orbit.v1");
worth_query_portable_type!(OtherSlot => "worth.query.declared-orbit.other.v1");
worth_query_application_query!(pub Orbit for Schema: OrbitModel, identity "DeclaredOrbit", parameters ParametersBinding, result RowsBinding, scope Node => "Node", name "declared_orbit");
worth_query_application_query!(pub Allowed for Schema: OrbitModel, identity "DeclaredOrbitAllowed", parameters ParametersBinding, result RowsBinding, scope Node => "Node", name "declared_orbit_allowed");
worth_query_application_query!(pub Omitted for Schema: OrbitModel, identity "DeclaredOrbitOmitted", parameters ParametersBinding, result RowsBinding, scope Node => "Node", name "declared_orbit_omitted");
worth_query_application_query!(pub Missing for Schema: OrbitModel, identity "DeclaredOrbitMissing", parameters ParametersBinding, result RowsBinding, scope Node => "Node", name "declared_orbit_missing");

pub(super) fn key<S: OrbitModel, Q: ApplicationQueryMarkerIdentity<S>>(
) -> ApplicationQueryResultFieldRef<
    Q,
    KeySlot,
    S,
    Node,
    NodeData,
    NodeKey,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("key", NodeKey::reference())
}
pub(super) fn orbit<S: OrbitModel>() -> ApplicationQueryOrbitRef<S, Orbit, OrbitSlot, Next, Node> {
    ApplicationQueryOrbitRef::new("orbit", Next::reference())
}
impl<S: OrbitModel, Q: ApplicationQueryMarkerIdentity<S>> WorthQueryApplicationProjection<S, Q>
    for NodeRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, S, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key::<S, Q>())?,
        })
    }
}
pub(super) fn whole<S: OrbitModel, Q: ApplicationQueryMarkerIdentity<S, Scope = Node>>(
) -> ApplicationQueryOrbitRef<S, Q, OrbitSlot, Next, Node> {
    ApplicationQueryOrbitRef::new("orbit", Next::reference())
}
pub(super) fn builder<
    S: OrbitModel,
    Q: ApplicationQueryMarkerIdentity<
            S,
            Scope = Node,
            ResultBinding = RowsBinding,
            ParameterBinding = ParametersBinding,
        > + 'static,
>(
    query: ApplicationQueryReference<S, Q, Parameters, NodeRow, Node>,
    disclosure: ApplicationQueryDisclosureContract,
) -> ApplicationQueryDefinitionBuilder<S, Q, Parameters, NodeRow, Node> {
    let shape = ApplicationQueryResultShapeBuilder::<S, Q, Node, NodeRow, RowsBinding>::new(
        Node::reference(),
    )
    .field(key::<S, Q>())
    .build();
    ApplicationQueryDefinitionBuilder::declare(query)
        .root(Node::reference())
        .scope(Node::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .orbit(whole::<S, Q>())
}
pub(super) fn definition<S: OrbitModel>(
) -> ApplicationQueryDefinition<S, Orbit, Parameters, NodeRow, Node> {
    builder(
        Orbit::reference(),
        ApplicationQueryDisclosureContract::public(),
    )
    .build()
    .unwrap()
}
fn disclosure<S: OrbitModel, Q: ApplicationQueryMarkerIdentity<S, Scope = Node> + 'static>(
    value: Option<u64>,
) -> ApplicationQueryDisclosureContract {
    let contract = ApplicationQueryDisclosureContract::governed_by(
        "orbit-query",
        super::capability::declarations::ReadOrbit::<S>::reference(),
    )
    .disclose_field_by(
        key::<S, Q>(),
        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(0).unwrap(),
        ApplicationQueryInfluenceContract::forbid_all(),
    );
    match value {
        None => contract,
        Some(value) => contract.orbit_by(
            whole::<S, Q>(),
            ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(value).unwrap(),
            ApplicationQueryInfluenceContract::permit_all(),
        ),
    }
}
pub(in crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits) fn declare<
    const O: bool,
>(
    schema: ApplicationSchemaDeclarationBuilder<OrbitSchema<O>>,
) -> ApplicationSchemaDeclarationBuilder<OrbitSchema<O>> {
    // The negative profile really lacks the orbit law and therefore these queries.
    if O {
        super::capability::declare(schema)
            .application_query(definition::<OrbitSchema<O>>())
            .application_query(super::metadata_only::definition::<OrbitSchema<O>>())
            .application_query(
                builder(
                    Allowed::reference(),
                    disclosure::<OrbitSchema<O>, Allowed>(Some(0)),
                )
                .build()
                .unwrap(),
            )
            .application_query(
                builder(
                    Omitted::reference(),
                    disclosure::<OrbitSchema<O>, Omitted>(Some(1)),
                )
                .build()
                .unwrap(),
            )
            .application_query(
                builder(
                    Missing::reference(),
                    disclosure::<OrbitSchema<O>, Missing>(None),
                )
                .build()
                .unwrap(),
            )
    } else {
        schema
    }
}
