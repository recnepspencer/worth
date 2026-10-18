//! Isolated fixed/discovered actions keep the existing no-source intent durable across restore.
use super::*;
use roots::ConsumerInitialFixedProgramRoot;
use worth_query_decl::facade::application_program::*;
use worth_query_topology_entry::{PlanarInitialAdjustment, PlanarInitialAdjustmentBinding};

type Binding = PlanarInitialAdjustmentBinding<ConsumerSchema>;
pub(crate) type FixedAction = ApplicationActionReference<
    ConsumerSchema,
    InitialDiscoveredPlanarRoot,
    PlanarSourceFeature,
    Binding,
    FixedDelivery,
>;
pub(crate) type DiscoveredAction = ApplicationActionReference<
    ConsumerSchema,
    InitialDiscoveredPlanarRoot,
    PlanarSourceFeature,
    Binding,
    DiscoveredDelivery,
>;
pub(crate) struct FixedDelivery;
pub(crate) struct DiscoveredDelivery;
impl ApplicationActionDelivery<ConsumerSchema, Binding> for FixedDelivery {
    type Mode =
        ApplicationFixedDeliveryMode<ConsumerInitialFixedProgramRoot, ApplicationNoProjection>;
    const IDENTITY: &'static str = "consumer.historical-fixed.v1";
}
impl ApplicationFixedActionDelivery<ConsumerSchema, Binding> for FixedDelivery {
    type Root = ConsumerInitialFixedProgramRoot;
    type Projection = ApplicationNoProjection;
}
impl ApplicationActionDelivery<ConsumerSchema, Binding> for DiscoveredDelivery {
    type Mode = ApplicationDiscoveredDeliveryMode<
        ConsumerInitialDiscoveredProgramRoot,
        ApplicationNoProjection,
    >;
    const IDENTITY: &'static str = "consumer.historical-discovered.v1";
}
impl ApplicationDiscoveredActionDelivery<ConsumerSchema, Binding> for DiscoveredDelivery {
    type Root = ConsumerInitialDiscoveredProgramRoot;
    type Projection = ApplicationNoProjection;
}
pub(crate) struct FixedProgram;
pub(crate) struct DiscoveredProgram;
macro_rules! historical_program {
    ($program:ty,$delivery:ty,$root:ty,$identity:literal) => {
        impl ApplicationProgramDefinition<ConsumerSchema> for $program {
            type Contributions = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
            type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
            type Outputs = ApplicationProgramOutputs<(ConsumerProgramRoot,
                (ConsumerSecondaryProgramRoot,(ConsumerDiscoveredProgramRoot,(ConsumerRequiredSharedRoot,$root))))>;
            const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new($identity);
            fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(visitor:V)->V::Output {
                visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>()
            }
            fn feature_specs()->Vec<ApplicationFeatureSpec> {
                let mut specs=ConsumerProgram::feature_specs();
                specs.retain(|spec| !(spec.feature().composition_instance()==InitialDiscoveredPlanarRoot::PATH
                    && spec.feature().identity()==<PlanarSourceFeature as ApplicationFeature<ConsumerSchema>>::IDENTITY));
                specs.push(ApplicationFeatureSpec::at::<ConsumerSchema,InitialDiscoveredPlanarRoot,PlanarSourceFeature>()
                    .provides::<PlanarBodyOutput>().action::<Binding, $delivery>().finish_action().finish());
                specs
            }
        }
    };
}
historical_program!(
    FixedProgram,
    FixedDelivery,
    ConsumerInitialFixedProgramRoot,
    "consumer.historical-fixed-program.v1"
);
historical_program!(
    DiscoveredProgram,
    DiscoveredDelivery,
    ConsumerInitialDiscoveredProgramRoot,
    "consumer.historical-discovered-program.v1"
);

// Initiation/replay can move this intent; no runtime source observation enters its durable identity.
pub(crate) struct Original(pub PlanarInitialAdjustment);
impl worth_query_decl::facade::application_operation::ApplicationMutationIntent<ConsumerSchema>
    for Original
{
    type Binding = Binding;
    fn input(&self) -> &PlanarInitialAdjustment {
        &self.0
    }
    fn scope_binding(&self) -> worth_query_topology_entry::PlanarMutationScope<ConsumerSchema> {
        <PlanarInitialAdjustment as worth_query_decl::facade::application_operation::ApplicationMutationIntent<ConsumerSchema>>::scope_binding(&self.0)
    }
}
