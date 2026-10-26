//! A fixed schema roster: complete forward, reverse and guarded composed paths.
use super::super::schema::*;
use crate::domain_computation::primary_graph::*;
use worth_query_declaration::{
    facade::{application_query::*, application_schema::*},
    *,
};

#[derive(Clone, Debug)]
pub struct Parameters;
worth_query_portable_type!(Parameters => "worth.query.walk-closure.parameters.v1");
worth_query_structured_value_binding!(pub ParametersBinding for Parameters {identity: "worth.query.walk-closure.parameters.v1"});
#[derive(Debug)]
pub struct NodeRow {
    pub key: u64,
}
worth_query_portable_type!(NodeRow => "worth.query.walk-closure.row.v1");
worth_query_structured_value_binding!(pub RowsBinding for NodeRow {identity: "worth.query.walk-closure.row.v1"});
pub struct KeySlot;
worth_query_portable_type!(KeySlot => "worth.query.walk-closure.key.v1");
worth_query_application_query!(pub Forward for WalkSchema, identity "WalkClosureForward", parameters ParametersBinding, result RowsBinding, scope Node => "Node", name "walk_closure_forward");
worth_query_application_query!(pub Reverse for WalkSchema, identity "WalkClosureReverse", parameters ParametersBinding, result RowsBinding, scope Node => "Node", name "walk_closure_reverse");
worth_query_application_query!(pub Composed for WalkSchema, identity "WalkClosureComposed", parameters ParametersBinding, result RowsBinding, scope Node => "Node", name "walk_closure_composed");

fn key<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    KeySlot,
    WalkSchema,
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
impl<Q: ApplicationQueryMarkerIdentity<WalkSchema>> WorthQueryApplicationProjection<WalkSchema, Q>
    for NodeRow
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, WalkSchema, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key::<Q>())?,
        })
    }
}
fn definition<Q: ApplicationQueryMarkerIdentity<WalkSchema> + 'static>(
    query: ApplicationQueryReference<WalkSchema, Q, Parameters, NodeRow, Node>,
    path: ApplicationQueryRootPath<WalkSchema, Node, Node>,
) -> ApplicationQueryDefinition<WalkSchema, Q, Parameters, NodeRow, Node>
where
    Q::ResultBinding: ApplicationStructuredValueBinding<Value = NodeRow>,
{
    let shape =
        ApplicationQueryResultShapeBuilder::<WalkSchema, Q, Node, NodeRow, Q::ResultBinding>::new(
            Node::reference(),
        )
        .field(key::<Q>())
        .build();
    ApplicationQueryDefinitionBuilder::declare(query)
        .root(Node::reference())
        .scope(Node::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(EditNode::reference())
        .root_path(path)
        .build()
        .unwrap()
}
pub(in crate::domain_computation::primary_graph::tests::application_attempt::invariant_walks) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<WalkSchema>,
) -> ApplicationSchemaDeclarationBuilder<WalkSchema> {
    schema
        .application_query(definition(
            Forward::reference(),
            ApplicationQueryRootPath::from(Node::reference()).forward_closure(Next::reference()),
        ))
        .application_query(definition(
            Reverse::reference(),
            ApplicationQueryRootPath::from(Node::reference()).reverse_closure(Next::reference()),
        ))
        .application_query(definition(
            Composed::reference(),
            ApplicationQueryRootPath::from(Node::reference())
                .forward(Next::reference())
                .where_equal(
                    Blocked::reference(),
                    ApplicationEncodedScalarValue::try_new(false).unwrap(),
                )
                .forward_closure(Next::reference())
                .where_equal(
                    Blocked::reference(),
                    ApplicationEncodedScalarValue::try_new(false).unwrap(),
                ),
        ))
}
