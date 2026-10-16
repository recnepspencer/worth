//! A real governed root whose ordinary source deliberately omits required locality.
use super::*;
use worth_query_decl::facade::application_program::{
    ApplicationArtifactDependency, ApplicationArtifactResourceCeiling,
    ApplicationArtifactRetention, ApplicationArtifactSuccession, ApplicationDerivedArtifact,
};
use worth_query_host::facade::application_contribution::WorthQueryProducerOutputFamily;
use worth_query_topology_entry::PlanarOutputFamily;

pub(crate) struct MissingLocalityProgram;
struct GovernedPlanarArtifact;
impl ApplicationDerivedArtifact<ConsumerSchema, PlanarOutputFeature> for GovernedPlanarArtifact {
    type Output = PlanarDerivedBodyOutput;
    type Locality = composed_action::SourceLocality;
    const IDENTITY: &'static str = "consumer.governed-root-artifact";
    const RETENTION: ApplicationArtifactRetention = ApplicationArtifactRetention::Retained;
    const SUCCESSION: ApplicationArtifactSuccession = ApplicationArtifactSuccession::Replace;
    const REQUIRED: bool = true;
    const PRODUCER_FAMILY: &'static str =
        <PlanarOutputFamily as WorthQueryProducerOutputFamily<ConsumerSchema>>::IDENTITY;
    const DEPENDENCIES: &'static [ApplicationArtifactDependency] =
        &[ApplicationArtifactDependency::new(
            <PlanarSourceFeature as ApplicationFeature<ConsumerSchema>>::IDENTITY,
        )];
    const REUSE_RULE: &'static str = "exact-source";
    const RESOURCE_CEILING: ApplicationArtifactResourceCeiling =
        ApplicationArtifactResourceCeiling::new(4_096, 8_192);
    const STOPPED_OUTCOME: &'static str = "governed-planar-resource-denied";
}
impl ApplicationProgramDefinition<ConsumerSchema> for MissingLocalityProgram {
    type Contributions =
        <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Outputs = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
    type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("consumer.fixed-missing-artifact-locality");
    fn adoption_authorization<V: worth_query_decl::facade::application_program::ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(visitor: V) -> V::Output{
        visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        // The existing source delivery specs genuinely declare both metadata absent.
        // Keep the real root's native producer/conditional operation and graph.
        let mut specs = features::source_delivery_specs::<ConsumerFixedDelivery>();
        specs[1] = ApplicationFeatureSpec::root::<ConsumerSchema, PlanarOutputFeature>()
            .derived_artifact::<GovernedPlanarArtifact>()
            .conditional_operation::<MutatePlanar>()
            .finish();
        specs
    }
}
