use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility, ApplicationQueryReference,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
    ApplicationQueryResultShapeBuilder, ExactlyOneResult, ForwardResultTraversal, ManyResults,
    OptionalOneResult,
};
use worth_query_declaration::{
    worth_query_application_query, worth_query_application_schema, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_portable_type, worth_query_relation,
    worth_query_structured_value_binding,
};

use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

worth_query_application_schema! {
    pub schema CardinalitySchema {
        owner: application_query_cardinality_test,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(Parent::reference())
                .entity(Child::reference())
                .aspect(Parent::reference(), ParentFacts::reference())
                .aspect(Child::reference(), ChildFacts::reference())
                .field(Parent::reference(), ParentId::reference())
                .field(Child::reference(), ChildId::reference())
                .relation(
                    ParentChild::reference(),
                    Parent::reference(),
                    Child::reference(),
                )
                .application_query(query_definition(
                    OptionalChildQuery::reference(),
                    ApplicationQueryCardinality::OptionalOne,
                ))
                .application_query(query_definition(
                    ManyChildrenQuery::reference(),
                    ApplicationQueryCardinality::Many,
                ))
        }
    }
}

worth_query_entity!(pub Parent for CardinalitySchema);
worth_query_entity!(pub Child for CardinalitySchema);
worth_query_aspect!(pub ParentFacts for CardinalitySchema, Parent; identity = AspectIdentity(0x91611048), revision = AspectContractRevision(1),);
worth_query_aspect!(pub ChildFacts for CardinalitySchema, Child; identity = AspectIdentity(0x91611049), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ParentId for CardinalitySchema, Parent, ParentFacts:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
    read_only, equality
);
worth_query_field!(
    pub ChildId for CardinalitySchema, Child, ChildFacts:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
    read_only, equality
);
worth_query_relation!(
    pub ParentChild in CardinalitySchema, Parent => Child; integrity = same_context_unbounded_retain_dangling);

struct OptionalChildQueryParameters;
struct ManyChildrenQueryParameters;
struct ParentResult;
struct ChildResult;
struct ParentIdSlot;
struct ChildIdSlot;
struct ChildrenSlot;

worth_query_portable_type!(ParentResult => "worth.query.test.cardinality.parent-result.v1");
worth_query_portable_type!(ChildResult => "worth.query.test.cardinality.child-result.v1");
worth_query_portable_type!(ParentIdSlot => "worth.query.test.cardinality.parent-id-slot.v1");
worth_query_portable_type!(ChildIdSlot => "worth.query.test.cardinality.child-id-slot.v1");
worth_query_portable_type!(ChildrenSlot => "worth.query.test.cardinality.children-slot.v1");
worth_query_structured_value_binding!(
    OptionalChildQueryParametersBinding for OptionalChildQueryParameters {
        identity: "OptionalChildQueryParameters"
    }
);
worth_query_structured_value_binding!(
    ManyChildrenQueryParametersBinding for ManyChildrenQueryParameters {
        identity: "ManyChildrenQueryParameters"
    }
);
worth_query_structured_value_binding!(
    ParentResultBinding for ParentResult {
        identity: "worth.query.test.cardinality.parent-result.v1"
    }
);
worth_query_structured_value_binding!(
    ChildResultBinding for ChildResult {
        identity: "worth.query.test.cardinality.child-result.v1"
    }
);

worth_query_application_query!(
    OptionalChildQuery for CardinalitySchema,
    identity "OptionalChildQuery",
    parameters OptionalChildQueryParametersBinding,
    result ParentResultBinding,
    scope Parent => "Parent",
    name "optional_child"
);
worth_query_application_query!(
    ManyChildrenQuery for CardinalitySchema,
    identity "ManyChildrenQuery",
    parameters ManyChildrenQueryParametersBinding,
    result ParentResultBinding,
    scope Parent => "Parent",
    name "many_children"
);

#[test]
fn nested_cardinality_changes_installed_graph_binding_and_planning_identity() {
    let schema = installed_schema();
    let optional = schema
        .certification_query(OptionalChildQuery::reference())
        .unwrap();
    let many = schema
        .certification_query(ManyChildrenQuery::reference())
        .unwrap();

    assert_eq!(
        optional.read_graph().relations()[0].cardinality(),
        ApplicationQueryCardinality::OptionalOne
    );
    assert_eq!(
        many.read_graph().relations()[0].cardinality(),
        ApplicationQueryCardinality::Many
    );
    assert_ne!(optional.read_graph().digest(), many.read_graph().digest());
    assert_ne!(
        optional.read_family_binding().identity(),
        many.read_family_binding().identity()
    );
    assert_ne!(
        optional.read_graph().canonical_planning_basis().digest(),
        many.read_graph().canonical_planning_basis().digest()
    );
    assert_ne!(
        optional.read_family_binding().canonical_planning_identity(),
        many.read_family_binding().canonical_planning_identity()
    );
}

fn query_definition<Query, Parameters>(
    reference: ApplicationQueryReference<
        CardinalitySchema,
        Query,
        Parameters,
        ParentResult,
        Parent,
    >,
    cardinality: ApplicationQueryCardinality,
) -> ApplicationQueryDefinition<CardinalitySchema, Query, Parameters, ParentResult, Parent>
where
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
        CardinalitySchema,
        ResultBinding = ParentResultBinding,
    >,
{
    let child = ApplicationQueryResultShapeBuilder::<
        CardinalitySchema,
        Query,
        Child,
        ChildResult,
        ChildResultBinding,
    >::new(Child::reference())
    .field(ApplicationQueryResultFieldRef::<
        Query,
        ChildIdSlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("child_id", ChildId::reference()));
    let shape = ApplicationQueryResultShapeBuilder::<
        CardinalitySchema,
        Query,
        Parent,
        ParentResult,
        ParentResultBinding,
    >::new(Parent::reference())
    .field(ApplicationQueryResultFieldRef::<
        Query,
        ParentIdSlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("parent_id", ParentId::reference()));
    let shape = match cardinality {
        ApplicationQueryCardinality::OptionalOne => shape.relation(
            ApplicationQueryResultRelationRef::<
                Query,
                ChildrenSlot,
                _,
                _,
                _,
                _,
                ForwardResultTraversal,
                OptionalOneResult,
            >::forward_optional("children", ParentChild::reference()),
            child,
        ),
        ApplicationQueryCardinality::ExactlyOne => shape.relation(
            ApplicationQueryResultRelationRef::<
                Query,
                ChildrenSlot,
                _,
                _,
                _,
                _,
                ForwardResultTraversal,
                ExactlyOneResult,
            >::forward_one("children", ParentChild::reference()),
            child,
        ),
        ApplicationQueryCardinality::Many => shape.relation(
            ApplicationQueryResultRelationRef::<
                Query,
                ChildrenSlot,
                _,
                _,
                _,
                _,
                ForwardResultTraversal,
                ManyResults,
            >::forward_many("children", ParentChild::reference()),
            child,
        ),
    }
    .build();
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Parent::reference())
        .scope(Parent::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 2))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}

fn installed_schema() -> crate::facade::WorthQueryInstalledApplicationSchema<CardinalitySchema> {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "application_query_cardinality_test",
        1,
        0,
    ))
    .application_schema(CardinalitySchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(CardinalitySchema::declaration().unwrap())
    .unwrap()
}
