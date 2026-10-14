use super::*;

fn consumer_specs_with_source(source: ApplicationFeatureSpec) -> Vec<ApplicationFeatureSpec> {
    vec![
        source,
        ApplicationFeatureSpec::root::<ConsumerSchema, PlanarOutputFeature>()
            .provides::<PlanarDerivedBodyOutput>()
            .conditional_operation::<MutatePlanar>()
            .finish(),
        ApplicationFeatureSpec::root::<ConsumerSchema, PlanarFinalOutputFeature>()
            .provides::<PlanarFinalBodyOutput>()
            .conditional_operation::<PublishFinalPlanarOutput>()
            .conditional_operation::<PreserveFinalPlanarOutput>()
            .finish(),
        ApplicationFeatureSpec::root::<ConsumerSchema, PlanarAlternateFinalOutputFeature>()
            .provides::<PlanarAlternateFinalBodyOutput>()
            .conditional_operation::<PublishAlternatePlanarOutput>()
            .finish(),
        ApplicationFeatureSpec::root::<ConsumerSchema, PlanarSummaryFeature>().finish(),
        ApplicationFeatureSpec::root::<ConsumerSchema, PlanarAlternateSummaryFeature>().finish(),
        ApplicationFeatureSpec::root::<ConsumerSchema, ParameterFeature>().finish(),
        ApplicationFeatureSpec::at::<ConsumerSchema, SecondaryPlanarRoot, PlanarSourceFeature>()
            .provides::<PlanarBodyOutput>()
            .finish(),
        ApplicationFeatureSpec::at::<ConsumerSchema, SecondaryPlanarRoot, PlanarOutputFeature>()
            .provides::<PlanarDerivedBodyOutput>()
            .finish(),
        ApplicationFeatureSpec::at::<ConsumerSchema, DiscoveredPlanarRoot, PlanarSourceFeature>()
            .provides::<PlanarBodyOutput>()
            .finish(),
        ApplicationFeatureSpec::at::<ConsumerSchema, DiscoveredPlanarRoot, PlanarOutputFeature>()
            .provides::<PlanarDerivedBodyOutput>()
            .finish(),
        ApplicationFeatureSpec::at::<
            ConsumerSchema,
            DiscoveredPlanarRoot,
            PlanarFinalOutputFeature,
        >()
        .provides::<PlanarFinalBodyOutput>()
        .finish(),
        ApplicationFeatureSpec::at::<
            ConsumerSchema,
            RequiredSharedPlanarRoot,
            PlanarSourceFeature,
        >()
        .provides::<PlanarBodyOutput>()
        .finish(),
        ApplicationFeatureSpec::at::<
            ConsumerSchema,
            RequiredSharedPlanarRoot,
            PlanarOutputFeature,
        >()
        .provides::<PlanarDerivedBodyOutput>()
        .finish(),
        ApplicationFeatureSpec::at::<
            ConsumerSchema,
            RequiredSharedPlanarRoot,
            PlanarFinalOutputFeature,
        >()
        .provides::<PlanarFinalBodyOutput>()
        .finish(),
        ApplicationFeatureSpec::at::<ConsumerSchema, InitialDiscoveredPlanarRoot, PlanarSourceFeature>()
            .provides::<PlanarBodyOutput>()
            .finish(),
        ApplicationFeatureSpec::at::<ConsumerSchema, InitialDiscoveredPlanarRoot, PlanarOutputFeature>()
            .provides::<PlanarDerivedBodyOutput>()
            .finish(),
        ApplicationFeatureSpec::at::<ConsumerSchema, InitialDiscoveredPlanarRoot, PlanarFinalOutputFeature>()
            .provides::<PlanarFinalBodyOutput>()
            .finish(),
    ]
}

fn source_builder() -> worth_query_decl::facade::application_program::ApplicationFeatureSpecBuilder<
    ConsumerSchema,
    ApplicationRootComposition,
    PlanarSourceFeature,
> {
    ApplicationFeatureSpec::root::<ConsumerSchema, PlanarSourceFeature>()
            .provides::<PlanarBodyOutput>()
            .repeated_optional_member_with_external_input::<
                PlanarEditBinding<ConsumerSchema>,
                correspondence::OptionalAdjustmentCorrespondence,
                external_input::NeutralExternalProvider,
            >()
            .mutation::<PriorCycleAdjustmentBinding<ConsumerSchema>>()
            .mutation_with_external_input::<
                VertexReplacementBinding<ConsumerSchema>,
                external_input::NeutralExternalProvider,
            >()
}

pub(super) fn consumer_feature_specs() -> Vec<ApplicationFeatureSpec> {
    consumer_specs_with_source(source_builder().finish())
}

pub(super) fn consumer_fixed_feature_specs() -> Vec<ApplicationFeatureSpec> {
    consumer_specs_with_source(source_builder().fixed_output_action::<PlanarSourceAdjustmentBinding<ConsumerSchema>, ConsumerFixedDelivery>().finish())
}
