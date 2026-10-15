//! Complete fixed source meaning, including independently admitted external input.
use super::*;
use worth_query_decl::facade::application_program::*;
use worth_query_topology_entry::AdjustPlanarSource;

pub(crate) struct SourceLocality;
pub(crate) struct SourceChange;
#[cfg(test)]
pub(crate) struct ChangedSourceChange;
pub(crate) struct CatalogInputRequirement;

impl ApplicationLocalityScope for SourceLocality {
    const IDENTITY: &'static str = "consumer.source-root.v1";
    const GRANULE: ApplicationLocalityGranule = ApplicationLocalityGranule::Root;
}
impl ApplicationChangeShape for SourceChange {
    const IDENTITY: &'static str = "consumer.source-replacement.v1";
    const POSTURE: ApplicationChangePosture = ApplicationChangePosture::Replace;
}
#[cfg(test)]
impl ApplicationChangeShape for ChangedSourceChange {
    const IDENTITY: &'static str = "consumer.source-reconstruction.v1";
    const POSTURE: ApplicationChangePosture = ApplicationChangePosture::Reconstruct;
}
impl ApplicationEvaluatedRequirementRule<ConsumerSchema, AdjustPlanarSource>
    for CatalogInputRequirement
{
    const IDENTITY: &'static str = "consumer.catalog-input-requirement.v1";
    type Context = u64;
    type Requirement = &'static str;
    type Finding = &'static str;
    fn evaluate(
        revision: &u64,
    ) -> ApplicationEvaluatedRequirement<Self::Requirement, Self::Finding> {
        if *revision > 0 {
            ApplicationEvaluatedRequirement::required("catalog-value", "revision-selected")
        } else {
            ApplicationEvaluatedRequirement::not_required("no-catalog-selection")
        }
    }
}

// compiled-example: begin
pub(crate) fn source_feature_spec() -> ApplicationFeatureSpec {
    features::source_builder()
        .action::<PlanarSourceAdjustmentBinding<ConsumerSchema>, ConsumerFixedDelivery>()
        .locality_and_change::<SourceLocality, SourceChange>()
        .evaluated_requirement::<CatalogInputRequirement>()
        .external_input::<external_input::NeutralExternalProvider>()
        .finish_action()
        .finish()
}
// compiled-example: end

#[cfg(test)]
#[test]
fn composed_action_macro_matches_canonical_installed_authoring() {
    let compact = worth_query_decl::facade::worth_query_feature_spec!(
        root(ConsumerSchema, PlanarSourceFeature);
        provides(PlanarBodyOutput);
        action(PlanarSourceAdjustmentBinding<ConsumerSchema>, ConsumerFixedDelivery) {
            external_input(external_input::NeutralExternalProvider);
            evaluated_requirement(CatalogInputRequirement);
            locality_and_change(SourceLocality, SourceChange);
        };
    );
    assert_eq!(
        compact.actions().first(),
        source_feature_spec().actions().last()
    );
}
