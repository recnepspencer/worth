pub use worth_query_decl::facade::{
    application_operation::ApplicationMutationBinding, application_program::*,
    application_schema::ApplicationSchema,
};
pub use worth_query_decl::facade::{authoring, canonicalization, schema_view, typed, validation};

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
