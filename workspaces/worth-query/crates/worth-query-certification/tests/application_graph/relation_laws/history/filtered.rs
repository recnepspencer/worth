//! A typed child predicate captures only its genuinely disclosed physical occurrence.
use super::super::schema::*;
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_application_query,
    worth_query_portable_type, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

type Schema = GraphSchema<Payload>;
#[derive(Clone, Debug)]
pub struct ReadInput {
    pub key: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub key: u64,
    pub children: Vec<u64>,
}
worth_query_portable_type!(Row => "worth.query.record-history.filtered.node-row.v1");
worth_query_structured_value_binding!(pub InputBinding for ReadInput { identity: "worth.query.record-history.filtered.node-input.v1" });
worth_query_structured_value_binding!(pub Parameters for () { identity: "worth.query.record-history.filtered.node-parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "worth.query.record-history.filtered.node-row.v1" });
worth_query_structured_value_binding!(pub ChildResultBinding for u64 { identity: "worth.query.record-history.filtered.node-child.v1" });
worth_query_application_query!(pub Query for Schema, identity "FilteredHistorySubjects", parameters Parameters,
    result ResultBinding, scope Node => "Node", name "filtered_history_subjects");
pub struct KeySlot;
pub struct ChildKeySlot;
pub struct Children;
worth_query_portable_type!(KeySlot => "worth.query.record-history.filtered.node-key.v1");
worth_query_portable_type!(ChildKeySlot => "worth.query.record-history.filtered.child-key.v1");
worth_query_portable_type!(Children => "worth.query.record-history.filtered.children.v1");

type Field<Slot> = ApplicationQueryResultFieldRef<
    Query,
    Slot,
    Schema,
    Node,
    NodeFacts,
    NodeKey,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
>;
fn key() -> Field<KeySlot> {
    ApplicationQueryResultFieldRef::new("key", NodeKey::reference())
}
fn child_key() -> Field<ChildKeySlot> {
    ApplicationQueryResultFieldRef::new("child_key", NodeKey::reference())
}
fn children() -> ApplicationQueryResultRelationRef<
    Query,
    Children,
    Schema,
    Precedes,
    Node,
    Node,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("children", Precedes::reference())
}

pub struct Binding;
type Scope =
    ApplicationQueryFieldScope<Schema, Node, NodeFacts, NodeKey, u64, ReadOnly, NoApplicationUnit>;
impl ApplicationQueryBinding<Schema> for Binding {
    type Input = ReadInput;
    type InputBinding = InputBinding;
    type Query = Query;
    type ParameterBinding = Parameters;
    type ResultBinding = ResultBinding;
    type ScopeBinding = Scope;
    type PrincipalBinding = GraphPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = "worth.query.record-history.filtered.node-read.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Node,
        NodeFacts,
        NodeKey,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        NodeKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        GraphPrincipal,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        GraphPrincipal::reference()
    }
}

impl WorthQueryApplicationProjection<Schema, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            key: row.field(key())?,
            children: row
                .many(children())?
                .iter()
                .map(|child| child.field(child_key()))
                .collect::<Result<_, _>>()?,
        })
    }
}

pub struct Relations;
worth_query_portable_type!(Relations => "worth.query.record-history.filtered-relations.v1");
impl ApplicationHistoryQueryBinding<Schema> for Relations {
    type SubjectRead = Binding;
    const IDENTITY: &'static str = "FilteredPrecedesChanges";
}
pub struct ChildParameter;
fn parameter() -> ApplicationQueryParameterRef<Query, ChildParameter, U64ApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("child_key")
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let nested =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Node, u64, ChildResultBinding>::new(
            Node::reference(),
        )
        .field(child_key());
    let shape = ApplicationQueryResultShapeBuilder::<Schema, Query, Node, Row, ResultBinding>::new(
        Node::reference(),
    )
    .field(key())
    .relation_where_equal(children(), nested, NodeKey::reference(), parameter())
    .build();
    let query = ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Node::reference())
        .scope(Node::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(parameter())
        .build()
        .unwrap();
    schema
        .application_query(query)
        .application_query_binding::<Binding>()
        .application_history_query(
            ApplicationHistoryQueryDefinition::<Schema, Relations>::relation(
                children(),
                ApplicationQueryDisclosureContract::public(),
            )
            .aspect(EdgeFacts::reference())
            .finish()
            .unwrap(),
        )
}

use super::super::installation::World;
use super::access::limits;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::num::NonZeroUsize;
use worth_query_host::facade::primary_graph::*;
pub(super) fn capture(
    world: &World<Payload>,
    key: u64,
) -> WorthQueryCapturedApplicationHistorySubjects<Schema, Relations> {
    let runtime = world.application.runtime();
    let request = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let history = runtime
                .installed_schema()
                .installed_history_query::<Relations>()
                .unwrap();
            let selected = runtime
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    history.subject_read().principal_binding(),
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let scope = selected
                .resolve_entity(
                    &phase,
                    NodeKey::reference(),
                    key,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
            let plan = selected
                .admit_application_query(
                    &phase,
                    history.subject_read().query(),
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(parameter(), 2)
                        .unwrap(),
                    WorthQueryProductQueryControls::new(NonZeroUsize::MIN, &request),
                )
                .unwrap();
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let capture = runtime
                .admit_application_history_capture(
                    &execution,
                    &history,
                    plan,
                    WorthQueryApplicationHistoryCaptureLimits {
                        maximum_subjects: NonZeroUsize::new(8).unwrap(),
                        maximum_aspects: NonZeroUsize::MIN,
                        maximum_source_visits: NonZeroUsize::new(128).unwrap(),
                        maximum_owned_bytes: limits().maximum_owned_bytes,
                    },
                )
                .unwrap();
            let (read, subjects) = runtime
                .execute_application_history_capture(&phase, capture)
                .unwrap()
                .into_parts();
            assert_eq!(read.rows()[0].key, key);
            if key == 0 {
                assert_eq!(read.rows()[0].children, vec![2]);
            }
            subjects
        })
        .unwrap()
}
