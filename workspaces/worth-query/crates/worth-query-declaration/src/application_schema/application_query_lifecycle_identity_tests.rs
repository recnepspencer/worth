use super::{
    ApplicationEffectMarkerIdentity, ApplicationEffectRef, ApplicationEntityRef,
    ApplicationFieldRef, ApplicationRelationRef, ApplicationRetainedEffectBinding,
    EqualityPredicate, NoApplicationUnit, ReadOnly,
};
use crate::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDependencyCeiling, ApplicationQueryDisclosureContract,
    ApplicationQueryLaneEligibility, ApplicationQueryLiveCauseBinding,
    ApplicationQueryLiveResourceContract, ApplicationQueryOrderingDirection,
    ApplicationQueryReference, ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
    ApplicationQueryResultShapeBuilder, ErasedApplicationQueryDefinition, ForwardResultTraversal,
    ManyResults,
};
pub(super) struct Schema;
struct Root;
struct Child;
struct RootAspect;
struct ChildAspect;
struct RootIdentity;
struct ChildIdentity;
struct Relation;
struct Parameters;
struct OtherParameters;
struct QueryResult;
struct OtherQueryResult;
struct OtherScope;
struct RootIdentitySlot;
struct ChildIdentitySlot;
struct ChildRelationSlot;
struct Effect;
mod identity_axes;

worth_query_portable_type!(Cause => "worth.query.test.query-live-cause.v1");
crate::worth_query_structured_value_binding!(
    pub(super) CauseBinding for Cause {
        identity: "worth.query.test.query-live-cause.v1"
    }
);
crate::worth_query_structured_value_binding!(
    ChildResultBinding for () {
        identity: "worth.rust.unit"
    }
);
worth_query_portable_type!(QueryResult => "worth.query.test.lifecycle.query-result.v1");
worth_query_portable_type!(OtherQueryResult => "worth.query.test.lifecycle.other-result.v1");
worth_query_portable_type!(RootIdentitySlot => "worth.query.test.lifecycle.root-slot.v1");
worth_query_portable_type!(ChildIdentitySlot => "worth.query.test.lifecycle.child-slot.v1");
worth_query_portable_type!(ChildRelationSlot => "worth.query.test.lifecycle.relation-slot.v1");
worth_query_portable_type!(LiveBinding => "worth.query.test.lifecycle.live-binding.v1");
worth_query_portable_type!(OtherLiveBinding => "worth.query.test.lifecycle.other-binding.v1");
crate::worth_query_structured_value_binding!(QueryParametersBinding for Parameters { identity: "Parameters" });
crate::worth_query_structured_value_binding!(QueryResultBinding for QueryResult { identity: "worth.query.test.lifecycle.query-result.v1" });
crate::worth_query_application_query!(
    Query for Schema,
    identity "Query",
    parameters QueryParametersBinding,
    result QueryResultBinding,
    scope Root => "Root",
    name "query"
);
crate::worth_query_structured_value_binding!(OtherQueryParametersBinding for Parameters { identity: "Parameters" });
crate::worth_query_structured_value_binding!(OtherQueryResultBinding for QueryResult { identity: "worth.query.test.lifecycle.query-result.v1" });
crate::worth_query_application_query!(
    OtherQuery for Schema,
    identity "worth.query.test.lifecycle.other-query.v1",
    parameters OtherQueryParametersBinding,
    result OtherQueryResultBinding,
    scope Root => "Root",
    name "query"
);
crate::worth_query_structured_value_binding!(OtherParametersQueryParametersBinding for OtherParameters { identity: "OtherParameters" });
crate::worth_query_structured_value_binding!(OtherParametersQueryResultBinding for QueryResult { identity: "worth.query.test.lifecycle.query-result.v1" });
crate::worth_query_application_query!(
    OtherParametersQuery for Schema,
    identity "Query",
    parameters OtherParametersQueryParametersBinding,
    result OtherParametersQueryResultBinding,
    scope Root => "Root",
    name "query"
);
crate::worth_query_structured_value_binding!(OtherResultQueryParametersBinding for Parameters { identity: "Parameters" });
crate::worth_query_structured_value_binding!(OtherResultQueryResultBinding for OtherQueryResult { identity: "worth.query.test.lifecycle.other-result.v1" });
crate::worth_query_application_query!(
    OtherResultQuery for Schema,
    identity "Query",
    parameters OtherResultQueryParametersBinding,
    result OtherResultQueryResultBinding,
    scope Root => "Root",
    name "query"
);
crate::worth_query_structured_value_binding!(OtherScopeQueryParametersBinding for Parameters { identity: "Parameters" });
crate::worth_query_structured_value_binding!(OtherScopeQueryResultBinding for QueryResult { identity: "worth.query.test.lifecycle.query-result.v1" });
crate::worth_query_application_query!(
    OtherScopeQuery for Schema,
    identity "Query",
    parameters OtherScopeQueryParametersBinding,
    result OtherScopeQueryResultBinding,
    scope OtherScope => "OtherScope",
    name "query"
);

impl ApplicationEffectMarkerIdentity<Schema> for Effect {
    type PayloadBinding = CauseBinding;
    const IDENTIFIER: &'static str = "Cause";
}

impl crate::application_schema::DeclaredApplicationFieldValue for RootIdentity {
    type Value = u64;
    type Binding = crate::application_schema::U64ApplicationValueBinding;
    const PRESENCE: crate::application_schema::ApplicationFieldPresence =
        crate::application_schema::ApplicationFieldPresence::Required;
}

impl crate::application_schema::RequiredApplicationFieldValue for RootIdentity {}

impl crate::application_schema::DeclaredApplicationFieldValue for ChildIdentity {
    type Value = u64;
    type Binding = crate::application_schema::U64ApplicationValueBinding;
    const PRESENCE: crate::application_schema::ApplicationFieldPresence =
        crate::application_schema::ApplicationFieldPresence::Required;
}

impl crate::application_schema::RequiredApplicationFieldValue for ChildIdentity {}

#[derive(Clone)]
pub(super) struct Cause {
    root: u64,
    child: u64,
}

impl ApplicationRetainedEffectBinding for CauseBinding {
    fn retained_bytes(value: &Self::Value) -> u64 {
        std::mem::size_of_val(value) as u64
    }
}

struct LiveBinding;
struct OtherLiveBinding;

impl ApplicationQueryLiveCauseBinding<Schema, Query, Root, Child> for LiveBinding {
    type Effect = Effect;
    type PayloadBinding = CauseBinding;
    type ScopeIdentityBinding = crate::application_schema::U64ApplicationValueBinding;
    type TargetIdentityBinding = crate::application_schema::U64ApplicationValueBinding;

    fn effect() -> ApplicationEffectRef<Schema, Self::Effect, Cause> {
        ApplicationEffectRef::from_declaration()
    }

    fn scope_identity(payload: &Cause) -> u64 {
        payload.root
    }

    fn target_identity(payload: &Cause) -> u64 {
        payload.child
    }
}

impl ApplicationQueryLiveCauseBinding<Schema, Query, Root, Child> for OtherLiveBinding {
    type Effect = Effect;
    type PayloadBinding = CauseBinding;
    type ScopeIdentityBinding = crate::application_schema::U64ApplicationValueBinding;
    type TargetIdentityBinding = crate::application_schema::U64ApplicationValueBinding;

    fn effect() -> ApplicationEffectRef<Schema, Self::Effect, Cause> {
        ApplicationEffectRef::from_declaration()
    }

    fn scope_identity(payload: &Cause) -> u64 {
        payload.root
    }

    fn target_identity(payload: &Cause) -> u64 {
        payload.child
    }
}

fn typed_definition<QueryMarker>() -> ErasedApplicationQueryDefinition
where
    QueryMarker: crate::application_query::ApplicationQueryMarkerIdentity<Schema>,
{
    let root = ApplicationEntityRef::<Schema, Root>::from_schema_identifier("Root");
    let scope = ApplicationEntityRef::<Schema, QueryMarker::Scope>::from_schema_identifier("Root");
    let identity = ApplicationQueryResultFieldRef::<
        QueryMarker,
        RootIdentitySlot,
        Schema,
        Root,
        RootAspect,
        RootIdentity,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    >::new("root", root_identity_field());
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        QueryMarker,
        Root,
        <QueryMarker::ResultBinding as crate::application_schema::ApplicationStructuredValueBinding>::Value,
        QueryMarker::ResultBinding,
    >::new(root)
    .field(identity)
    .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::<
        Schema,
        QueryMarker,
        <QueryMarker::ParameterBinding as crate::application_schema::ApplicationStructuredValueBinding>::Value,
        <QueryMarker::ResultBinding as crate::application_schema::ApplicationStructuredValueBinding>::Value,
        QueryMarker::Scope,
    >::from_declaration())
    .root(root)
    .scope(scope)
    .result_shape(shape)
    .cardinality(ApplicationQueryCardinality::ExactlyOne)
    .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 1))
    .disclosure(ApplicationQueryDisclosureContract::public())
    .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
    .lanes(ApplicationQueryLaneEligibility::one_shot())
    .public()
    .build()
    .expect("typed identity fixture must be lawful")
    .into_erased()
}

fn collection_definition(continuation: bool) -> ErasedApplicationQueryDefinition {
    let root = root_entity();
    let child = child_entity();
    let child_shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Child, (), ChildResultBinding>::new(
            child,
        )
        .field(child_identity());
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Root,
        QueryResult,
        QueryResultBinding,
    >::new(root)
    .field(root_identity())
    .relation(children(), child_shape)
    .build();
    let builder = ApplicationQueryDefinitionBuilder::declare(query_reference())
        .root(root)
        .scope(root)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 2))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .order_by(
            child_identity(),
            ApplicationQueryOrderingDirection::Ascending,
        );
    let builder = if continuation {
        builder.continue_by(children())
    } else {
        builder
    };
    builder
        .build()
        .expect("continuation identity fixture must be lawful")
        .into_erased()
}

fn live_definition<Binding>(
    resources: ApplicationQueryLiveResourceContract,
) -> ErasedApplicationQueryDefinition
where
    Binding: ApplicationQueryLiveCauseBinding<
        Schema,
        Query,
        Root,
        Child,
        ScopeIdentityBinding = crate::application_schema::U64ApplicationValueBinding,
        TargetIdentityBinding = crate::application_schema::U64ApplicationValueBinding,
    >,
{
    let root = root_entity();
    let child = child_entity();
    let child_shape =
        ApplicationQueryResultShapeBuilder::<Schema, Query, Child, (), ChildResultBinding>::new(
            child,
        )
        .field(child_identity());
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Root,
        QueryResult,
        QueryResultBinding,
    >::new(root)
    .field(root_identity())
    .relation(children(), child_shape)
    .build();
    ApplicationQueryDefinitionBuilder::declare(query_reference())
        .root(root)
        .scope(root)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 2))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .public()
        .order_by(
            child_identity(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(children())
        .live_by::<Child, Binding, _, _, _, _, _, _, _, _>(
            root_identity(),
            child_identity(),
            resources,
        )
        .build()
        .expect("live identity fixture must be lawful")
        .into_erased()
}

fn query_reference() -> ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Root> {
    Query::reference()
}

fn root_entity() -> ApplicationEntityRef<Schema, Root> {
    ApplicationEntityRef::from_schema_identifier("Root")
}

fn child_entity() -> ApplicationEntityRef<Schema, Child> {
    ApplicationEntityRef::from_schema_identifier("Child")
}

fn root_identity_field() -> ApplicationFieldRef<
    Schema,
    Root,
    RootAspect,
    RootIdentity,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationFieldRef::from_schema_identifiers("Root", "Identity", "Id")
}

fn root_identity() -> ApplicationQueryResultFieldRef<
    Query,
    RootIdentitySlot,
    Schema,
    Root,
    RootAspect,
    RootIdentity,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("root", root_identity_field())
}

fn child_identity() -> ApplicationQueryResultFieldRef<
    Query,
    ChildIdentitySlot,
    Schema,
    Child,
    ChildAspect,
    ChildIdentity,
    u64,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new(
        "child",
        ApplicationFieldRef::from_schema_identifiers("Child", "Identity", "Id"),
    )
}

fn children() -> ApplicationQueryResultRelationRef<
    Query,
    ChildRelationSlot,
    Schema,
    Relation,
    Root,
    Child,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many(
        "children",
        ApplicationRelationRef::from_schema_identifiers(
            "Children", "Root", "Child",
            crate::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        ),
    )
}
