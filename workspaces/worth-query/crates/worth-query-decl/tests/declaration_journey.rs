use worth_query_decl::facade::{authoring, canonicalization, schema_view, typed, validation};
use worth_query_decl::facade::{
    application_operation::ApplicationMutationBinding,
    application_program::*,
    application_schema::ApplicationSchema,
};

/// Compile-pass contract through the declaration audience's actual export route.
pub fn complete_action<S, I, F, B, D, L, H, C, R, P>(
    parent: ApplicationFeatureSpecBuilder<S, I, F>,
) -> ApplicationFeatureSpec
where
    S: ApplicationSchema + 'static,
    I: ApplicationCompositionInstance,
    F: ApplicationFeature<S>,
    B: ApplicationMutationBinding<S>,
    D: ApplicationActionDelivery<S, B>,
    L: ApplicationLocalityScope,
    H: ApplicationChangeShape,
    C: ApplicationRepeatedOptionalMemberCorrespondence<S, B>,
    R: ApplicationEvaluatedRequirementRule<S, B::Operation>,
    P: ApplicationExternalInputProvider<S, B::Operation>,
{
    parent
        .action::<B, D>()
        .locality_and_change::<L, H>()
        .correspondence::<C>()
        .evaluated_requirement::<R>()
        .external_input::<P>()
        .finish_action()
        .finish()
}

/// Native declaration uses the same contract axis without a delivery override.
pub fn native_action<S, I, F, B, P>(
    parent: ApplicationFeatureSpecBuilder<S, I, F>,
) -> ApplicationFeatureSpec
where
    S: ApplicationSchema + 'static,
    I: ApplicationCompositionInstance,
    F: ApplicationFeature<S>,
    B: ApplicationMutationBinding<S>,
    P: ApplicationExternalInputProvider<S, B::Operation>,
{
    parent
        .native_action::<B>()
        .external_input::<P>()
        .finish_action()
        .finish()
}

/// Lifecycle supplies selector meaning; the audience exports that exact type.
pub fn select_action<S, B, A>() -> ApplicationActionSelector
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    A: ApplicationAction<S, B>,
{
    A::selector()
}

#[test]
fn declaration_facade_carries_authored_intent_into_schema_validation() {
    let query =
        authoring::RawAuthoredQuery::detail_builder(authoring::RootEntityKey::new("task").unwrap())
            .project(authoring::AspectFieldSelector::new("title", "text").unwrap())
            .build()
            .unwrap();
    let shape = authoring::RawAuthoredResultShape::detail_builder()
        .field(authoring::AuthoredResultShapeField::new("title", "text", "title").unwrap())
        .build()
        .unwrap();
    let request = authoring::GuidedAuthoringPath::pair_detail(query, shape).unwrap();

    let canonical = canonicalization::canonicalize_request(request).unwrap();
    let canonical_authority = canonical.query().authority();
    let schema = schema_view::QuerySchemaView::new(
        "task-schema-v1",
        [schema_view::SchemaFieldView::new(
            authoring::AspectName::new("title").unwrap(),
            authoring::FieldName::new("text").unwrap(),
            typed::ScalarAspectType::String,
        )],
        [],
    );
    let schema_authority = schema.basis_authority();

    let validated = validation::validate_canonical_bundle(canonical, schema).unwrap();

    assert_eq!(validated.query().canonical_authority(), canonical_authority);
    assert_eq!(validated.query().schema_basis_authority(), schema_authority);
    assert_eq!(validated.result_shape().bindings().len(), 1);
}
