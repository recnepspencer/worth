//! Actual rostered successors preserve output topology while changing action meaning.
use super::*;
pub(crate) struct UnchangedFixed;
pub(crate) struct RemovedFixed;
macro_rules! successor {
    ($ty:ty, $id:literal, $specs:ident) => {
        impl ApplicationProgramDefinition<ConsumerSchema> for $ty {
            type Contributions =
                <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
            type Outputs =
                <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
            type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
            const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new($id);
            fn adoption_authorization<Visitor: worth_query_decl::facade::application_program::ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(visitor: Visitor) -> Visitor::Output { visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>() }
            fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                features::$specs()
            }
        }
    };
}
successor!(
    UnchangedFixed,
    "fixed-action-unchanged-successor",
    consumer_fixed_feature_specs
);
successor!(
    RemovedFixed,
    "fixed-action-removed-successor",
    consumer_feature_specs
);
