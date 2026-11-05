//! Both negative placements use the same genuinely installed external producer family.
use super::*;
use worth_query_topology_entry::*;
pub struct RootInstance;
pub struct ChildInstance;
impl ApplicationCompositionInstance for RootInstance {
    const PATH: &'static str = "eligibility.root";
}
impl ApplicationCompositionInstance for ChildInstance {
    const PATH: &'static str = "eligibility.child";
}
type RootConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    RootInstance,
    PlanarSourceFeature,
    PlanarBodyOutput,
    RootInstance,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarInitialToEscapingOutput,
>;
pub type Root = ApplicationOutputGraph<RootConnection, ApplicationOutputLeaf>;
type ChildRootConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    ChildInstance,
    PlanarSourceFeature,
    PlanarBodyOutput,
    ChildInstance,
    PlanarOutputFeature,
    PlanarBodyInput,
    PlanarInitialToOutputConnection,
>;
type ChildConnection = ApplicationConnectionInstanceRef<
    ConsumerSchema,
    ChildInstance,
    PlanarOutputFeature,
    PlanarDerivedBodyOutput,
    ChildInstance,
    PlanarFinalOutputFeature,
    PlanarDerivedBodyInput,
    PlanarOutputToEscapingOutput,
>;
pub type ChildRoot = ApplicationOutputGraph<
    ChildRootConnection,
    ApplicationOutputEdge<ChildConnection, ApplicationOutputLeaf>,
>;
pub struct RootDelivery;
pub struct ChildDelivery;
macro_rules! delivery {
    ($delivery:ty, $root:ty, $identity:literal) => {
        impl ApplicationActionDelivery<ConsumerSchema, Binding> for $delivery {
            type Mode = ApplicationFixedDeliveryMode<$root, ApplicationNoProjection>;
            const IDENTITY: &'static str = $identity;
        }
        impl ApplicationFixedActionDelivery<ConsumerSchema, Binding> for $delivery {
            type Root = $root;
            type Projection = ApplicationNoProjection;
        }
    };
}
delivery!(RootDelivery, Root, "eligibility.root-delivery.v1");
delivery!(ChildDelivery, ChildRoot, "eligibility.child-delivery.v1");
pub type RootAction = ApplicationActionReference<
    ConsumerSchema,
    RootInstance,
    PlanarSourceFeature,
    Binding,
    RootDelivery,
>;
pub type ChildAction = ApplicationActionReference<
    ConsumerSchema,
    ChildInstance,
    PlanarSourceFeature,
    Binding,
    ChildDelivery,
>;
pub(crate) struct Program;
impl ApplicationProgramDefinition<ConsumerSchema> for Program {
    type Contributions =
        <FixedProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
    type Rules = <FixedProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
    type Outputs = ApplicationProgramOutputs<(
        <<FixedProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs as ApplicationProgramOutputsShape<ConsumerSchema>>::Roots,
        (Root, ChildRoot),
    )>;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("eligibility.fixed-program.v1");
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(
        visitor: V,
    ) -> V::Output {
        FixedProgram::adoption_authorization(visitor)
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let mut specs = FixedProgram::feature_specs();
        specs.extend([
            ApplicationFeatureSpec::at::<ConsumerSchema, RootInstance, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .action::<Binding, RootDelivery>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::at::<ConsumerSchema, RootInstance, PlanarOutputFeature>()
                .finish(),
            ApplicationFeatureSpec::at::<ConsumerSchema, ChildInstance, PlanarSourceFeature>()
                .provides::<PlanarBodyOutput>()
                .action::<Binding, ChildDelivery>()
                .finish_action()
                .finish(),
            ApplicationFeatureSpec::at::<ConsumerSchema, ChildInstance, PlanarOutputFeature>()
                .provides::<PlanarDerivedBodyOutput>()
                .finish(),
            ApplicationFeatureSpec::at::<ConsumerSchema, ChildInstance, PlanarFinalOutputFeature>()
                .finish(),
        ]);
        specs
    }
}
