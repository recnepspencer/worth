use super::source_completion::ConsumerSourceCompletion;
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

pub(super) fn source_builder(
) -> worth_query_decl::facade::application_program::ApplicationFeatureSpecBuilder<
    ConsumerSchema,
    ApplicationRootComposition,
    PlanarSourceFeature,
> {
    ApplicationFeatureSpec::root::<ConsumerSchema, PlanarSourceFeature>()
        .provides::<PlanarBodyOutput>()
        .action::<PlanarEditBinding<ConsumerSchema>, ConsumerSourceCompletion>()
        .correspondence::<correspondence::OptionalAdjustmentCorrespondence>()
        .external_input::<external_input::NeutralExternalProvider>()
        .finish_action()
        .action::<PriorCycleAdjustmentBinding<ConsumerSchema>, ConsumerSourceCompletion>()
        .finish_action()
        .action::<VertexReplacementBinding<ConsumerSchema>, ConsumerSourceCompletion>()
        .external_input::<external_input::NeutralExternalProvider>()
        .finish_action()
}

pub(super) fn consumer_feature_specs() -> Vec<ApplicationFeatureSpec> {
    consumer_specs_with_source(source_builder().finish())
}

pub(super) fn consumer_fixed_feature_specs() -> Vec<ApplicationFeatureSpec> {
    consumer_specs_with_source(composed_action::source_feature_spec())
}

#[cfg(test)]
pub(super) fn source_delivery_specs<D>() -> Vec<ApplicationFeatureSpec>
where
    D: worth_query_decl::facade::application_program::ApplicationActionDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    >,
{
    consumer_specs_with_source(
        source_builder()
            .action::<PlanarSourceAdjustmentBinding<ConsumerSchema>, D>()
            .finish_action()
            .finish(),
    )
}

/// The same delivery under a changed declared locality/change contract.
#[cfg(test)]
pub(super) fn changed_source_delivery_specs<D>() -> Vec<ApplicationFeatureSpec>
where
    D: worth_query_decl::facade::application_program::ApplicationActionDelivery<
        ConsumerSchema,
        PlanarSourceAdjustmentBinding<ConsumerSchema>,
    >,
{
    use composed_action::{ChangedSourceChange, SourceLocality};
    consumer_specs_with_source(
        source_builder()
            .action::<PlanarSourceAdjustmentBinding<ConsumerSchema>, D>()
            .locality_and_change::<SourceLocality, ChangedSourceChange>()
            .finish_action()
            .finish(),
    )
}

#[cfg(test)]
pub(super) fn consumer_changed_fixed_feature_specs() -> Vec<ApplicationFeatureSpec> {
    consumer_specs_with_source(source_builder()
        .action::<PlanarSourceAdjustmentBinding<ConsumerSchema>, ConsumerFixedDelivery>()
        .locality_and_change::<composed_action::SourceLocality, composed_action::ChangedSourceChange>()
        .evaluated_requirement::<composed_action::CatalogInputRequirement>()
        .external_input::<external_input::NeutralExternalProvider>()
        .finish_action().finish())
}
