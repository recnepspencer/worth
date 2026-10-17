//! Actual rostered successors preserve output topology while changing action meaning.
use super::*;
pub(crate) struct UnchangedFixed;
pub(crate) struct RemovedFixed;
#[cfg(test)]
pub(crate) struct ChangedFixed;
#[cfg(test)]
pub(crate) struct UnchangedNoProjection;
#[cfg(test)]
pub(crate) struct ChangedNoProjection;
macro_rules! successor {
    ($ty:ty, $id:literal, $specs:expr) => {
        impl ApplicationProgramDefinition<ConsumerSchema> for $ty {
            type Contributions =
                <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Contributions;
            type Outputs =
                <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Outputs;
            type Rules = <ConsumerProgram as ApplicationProgramDefinition<ConsumerSchema>>::Rules;
            const IDENTITY: ApplicationProgramIdentity = ApplicationProgramIdentity::new($id);
            fn adoption_authorization<Visitor: worth_query_decl::facade::application_program::ApplicationProgramAdoptionAuthorizationVisitor<ConsumerSchema>>(visitor: Visitor) -> Visitor::Output { visitor.capability::<crate::program_deployment::ConsumerProgramDeploymentBinding>() }
            fn feature_specs() -> Vec<ApplicationFeatureSpec> {
                $specs
            }
        }
    };
}
successor!(
    UnchangedFixed,
    "fixed-action-unchanged-successor",
    features::consumer_fixed_feature_specs()
);
successor!(
    RemovedFixed,
    "fixed-action-removed-successor",
    features::consumer_feature_specs()
);

#[cfg(test)]
successor!(
    ChangedFixed,
    "fixed-action-changed-contract-successor",
    features::consumer_changed_fixed_feature_specs()
);

// The no-projection owner's successors: identical meaning, and the same
// delivery under a changed declared contract. `RemovedFixed` removes either.
#[cfg(test)]
successor!(
    UnchangedNoProjection,
    "no-projection-action-unchanged-successor",
    features::source_delivery_specs::<super::no_projection::NoProjectionDelivery>()
);
#[cfg(test)]
successor!(
    ChangedNoProjection,
    "no-projection-action-changed-contract-successor",
    features::changed_source_delivery_specs::<super::no_projection::NoProjectionDelivery>()
);
