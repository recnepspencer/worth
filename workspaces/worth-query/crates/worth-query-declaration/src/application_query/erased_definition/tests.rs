use super::{ErasedApplicationQueryDefinition, WorthQueryPortableApplicationQueryParts};
use crate::application_query::{
    validate_portable_application_query_freshly, ApplicationQueryBasisSupport,
    ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDefinitionDenial, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryDisclosurePosture,
    ApplicationQueryDisclosureRule, ApplicationQueryDisclosureSelector,
    ApplicationQueryInfluenceContract, ApplicationQueryLaneEligibility,
    ApplicationQueryParameterDefinition, ApplicationQueryResultShape,
    ApplicationQueryResultShapeBuilder, WorthQueryPortableApplicationQueryDisclosureParts,
    WorthQueryPortableApplicationQueryResultShapeParts,
};
use crate::application_schema::{
    validate_portable_application_schema_freshly, ApplicationEntityRef, ApplicationSchemaMember,
    WorthQueryPortableApplicationSchemaParts, WorthQueryPortableApplicationSchemaRecord,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::{
    AspectMask, AspectValue, CanonicalFieldPath, FieldKey, ScalarAspectType,
};

struct Schema;
struct Entity;
struct Parameters;
struct Result;

crate::worth_query_structured_value_binding!(QueryParametersBinding for Parameters { identity: "worth.query.test.portable-query-parameters.v1" });
crate::worth_query_structured_value_binding!(QueryResultBinding for Result { identity: "worth.query.test.portable-query-result.v1" });
crate::worth_query_application_query!(
    Query for Schema,
    identity "worth.query.test.portable-query.v1",
    parameters QueryParametersBinding,
    result QueryResultBinding,
    scope Entity => "worth.query.test.portable-query-scope.v1",
    name "portable_query"
);
crate::worth_query_portable_type!(Result => "worth.query.test.portable-query-result.v1");

#[test]
fn typed_definition_round_trips_through_owned_untrusted_parts() {
    let original = typed_definition();
    let canonical = original.canonical_basis().clone();
    let parts = original.clone().into_parts();

    let reconstructed = ErasedApplicationQueryDefinition::from_untrusted_parts(parts.clone());

    assert_eq!(reconstructed, original);
    assert_eq!(reconstructed.parts(), &parts);
    assert_eq!(reconstructed.canonical_basis(), &canonical);
}

#[test]
fn reconstructed_definition_owns_runtime_text_without_static_promotion() {
    let mut parts = typed_definition().into_parts();
    let dynamic_entity = ["Owned", "Entity"].join("");
    parts.name = format!("{}_query", dynamic_entity.to_lowercase());
    parts.root_entity = dynamic_entity.clone();
    parts.scope_entity = dynamic_entity.clone();
    parts.result_shape = ApplicationQueryResultShape::from_untrusted_parts(
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: parts.query_type.clone(),
            root_entity: dynamic_entity.clone(),
            result_type: parts.result_type.clone(),
            fields: Vec::new(),
            relations: Vec::new(),
        },
    );

    let reconstructed = ErasedApplicationQueryDefinition::from_untrusted_parts(parts);

    assert_eq!(reconstructed.name(), "ownedentity_query");
    assert_eq!(reconstructed.root_entity(), dynamic_entity);
    assert_eq!(
        validate_portable_application_query_freshly(reconstructed.parts()),
        Ok(())
    );
}

#[test]
fn noncanonical_untrusted_sequences_are_preserved_and_fail_fresh_schema_readmission() {
    let mut parts = dynamically_owned_parts();
    parts.parameters = vec![parameter("zeta"), parameter("alpha")];
    let reconstructed = ErasedApplicationQueryDefinition::from_untrusted_parts(parts);
    assert_eq!(reconstructed.parameters()[0].name(), "zeta");
    assert_eq!(reconstructed.parameters()[1].name(), "alpha");
    assert_eq!(
        validate_portable_application_query_freshly(reconstructed.parts()),
        Err(ApplicationQueryDefinitionDenial::InvalidCanonicalOrdering)
    );

    let mut members = vec![
        ApplicationSchemaMember::Entity {
            entity: "OwnedEntity".to_owned(),
        },
        ApplicationSchemaMember::ApplicationQuery {
            definition: reconstructed,
        },
    ];
    members.sort();
    let record = WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(
        WorthQueryPortableApplicationSchemaParts {
            owner: "WORTH.tests".to_owned(),
            name: "portable-query-readmission".to_owned(),
            major: 1,
            minor: 0,
            members,
            contributions: Vec::new(),
        },
    );

    assert_eq!(
        validate_portable_application_schema_freshly(record),
        Err(crate::application_schema::ApplicationSchemaDeclarationDenial::InvalidApplicationQuery)
    );
}

#[test]
fn incomplete_disclosure_capability_is_total_to_construct_but_fails_fresh_validation() {
    let mut parts = dynamically_owned_parts();
    parts.disclosure = ApplicationQueryDisclosureContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryDisclosureParts {
            posture: ApplicationQueryDisclosurePosture::Governed,
            classification: "restricted".to_owned(),
            capability_name: Some("ReadRestricted".to_owned()),
            capability_type: None,
            rules: Vec::new(),
        },
    );

    let reconstructed = ErasedApplicationQueryDefinition::from_untrusted_parts(parts);

    assert_eq!(
        validate_portable_application_query_freshly(reconstructed.parts()),
        Err(ApplicationQueryDefinitionDenial::InvalidDisclosureContract)
    );
}

#[test]
fn disclosure_field_masks_must_match_the_typed_selector_contract() {
    let mut parts = dynamically_owned_parts();
    let field = FieldKey::new("HiddenField").expect("fixture field key is valid");
    let selector = ApplicationQueryDisclosureSelector::InternalField {
        entity: "OwnedEntity".to_owned(),
        aspect: "HiddenFacts".to_owned(),
        field: "HiddenField".to_owned(),
        projection_mask: AspectMask::whole_aspect(),
        diagnostic_mask: AspectMask::new([CanonicalFieldPath::single(field)]),
    };
    parts.disclosure = ApplicationQueryDisclosureContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryDisclosureParts {
            posture: ApplicationQueryDisclosurePosture::Public,
            classification: "public".to_owned(),
            capability_name: None,
            capability_type: None,
            rules: vec![ApplicationQueryDisclosureRule::from_untrusted_fields(
                selector,
                AspectValue::Bool(true),
                ApplicationQueryInfluenceContract::forbid_all(),
            )],
        },
    );

    let reconstructed = ErasedApplicationQueryDefinition::from_untrusted_parts(parts);

    assert_eq!(
        validate_portable_application_query_freshly(reconstructed.parts()),
        Err(ApplicationQueryDefinitionDenial::DisclosureSelectorMismatch)
    );
}

fn typed_definition() -> ErasedApplicationQueryDefinition {
    let entity = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier("Entity");
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Entity,
        Result,
        QueryResultBinding,
    >::new(entity)
    .build();
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(entity)
        .scope(entity)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 0))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("the typed fixture is valid")
        .into_erased()
}

fn dynamically_owned_parts() -> WorthQueryPortableApplicationQueryParts {
    let mut parts = typed_definition().into_parts();
    parts.name = "owned_query".to_owned();
    parts.root_entity = "OwnedEntity".to_owned();
    parts.scope_entity = "OwnedEntity".to_owned();
    parts.result_shape = ApplicationQueryResultShape::from_untrusted_parts(
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: parts.query_type.clone(),
            root_entity: "OwnedEntity".to_owned(),
            result_type: parts.result_type.clone(),
            fields: Vec::new(),
            relations: Vec::new(),
        },
    );
    parts
}

fn parameter(name: &str) -> ApplicationQueryParameterDefinition {
    ApplicationQueryParameterDefinition::from_untrusted_fields(
        name.to_owned(),
        ScalarAspectType::String,
        WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".to_owned()),
    )
}
