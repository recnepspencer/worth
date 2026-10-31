//! An isolated leaf profile reuses the installed contribution and source recipe.
use super::roots::ConsumerInitialLeafProgramRoot;
use super::*;
use worth_query_decl::facade::application_program::*;
use worth_query_topology_entry::PlanarInitialAdjustmentBinding;

type Binding = PlanarInitialAdjustmentBinding<ConsumerSchema>;
pub(crate) type HistoryAction = ApplicationActionReference<
    ConsumerSchema,
    InitialDiscoveredPlanarRoot,
    PlanarSourceFeature,
    Binding,
    HistoryDelivery,
>;
pub(crate) type HistoryRoot = ConsumerInitialLeafProgramRoot;
pub(crate) struct HistoryDelivery;
pub(crate) struct HistoryProgram;
impl ApplicationActionDelivery<ConsumerSchema, Binding> for HistoryDelivery {
    type Mode = ApplicationFixedDeliveryMode<HistoryRoot, ApplicationNoProjection>;
    const IDENTITY: &'static str = "consumer.document-history-leaf.v1";
}
impl ApplicationFixedActionDelivery<ConsumerSchema, Binding> for HistoryDelivery {
    type Root = HistoryRoot;
    type Projection = ApplicationNoProjection;
}
impl ApplicationProgramDefinition<ConsumerSchema> for HistoryProgram {
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    type Outputs = ApplicationProgramOutputs<(
        ConsumerProgramRoot,
        (
            ConsumerSecondaryProgramRoot,
            (
                ConsumerDiscoveredProgramRoot,
                (ConsumerRequiredSharedRoot, HistoryRoot),
            ),
        ),
    )>;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("consumer.document-history-program.v1");
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(
        visitor: V,
    ) -> V::Output {
        visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = ConsumerProgram::feature_specs();
        specs.retain(|spec| {
            !(spec.feature().composition_instance() == InitialDiscoveredPlanarRoot::PATH
                && (spec.feature().identity()
                    == <PlanarSourceFeature as ApplicationFeature<ConsumerSchema>>::IDENTITY
                    || spec.feature().identity()
                    == <PlanarFinalOutputFeature as ApplicationFeature<ConsumerSchema>>::IDENTITY))
        });
        specs.push(
            ApplicationFeatureSpec::at::<
                ConsumerSchema,
                InitialDiscoveredPlanarRoot,
                PlanarSourceFeature,
            >()
            .provides::<PlanarBodyOutput>()
            .action::<Binding, HistoryDelivery>()
            .finish_action()
            .finish(),
        );
        specs
    }
}
