use super::{grants::*, schema::*};
use worth_query_host::facade::{
    declaration::application_operation::*, declaration::application_program::*,
};
pub struct Program;
pub struct Feature;
pub struct Completion;
impl ApplicationFeature<CapabilitySchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "capability-retention.feature";
}
impl ApplicationActionDelivery<CapabilitySchema, SetRetentionBinding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "capability-retention.completion";
}
pub type Action = ApplicationActionReference<
    CapabilitySchema,
    ApplicationRootComposition,
    Feature,
    SetRetentionBinding,
    Completion,
>;
impl ApplicationProgramDefinition<CapabilitySchema> for Program {
    type Contributions = (CapabilityContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("capability-retention.program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<CapabilitySchema, Feature>()
            .action::<SetRetentionBinding, Completion>()
            .finish_action()
            .operation::<RevokeGrant>()
            .finish()]
    }
}
pub struct Original {
    pub input: SetRetentionInput,
}
impl ApplicationMutationIntent<CapabilitySchema> for Original {
    type Binding = SetRetentionBinding;
    fn input(&self) -> &SetRetentionInput {
        &self.input
    }
    fn scope_binding(
        &self,
    ) -> <SetRetentionBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding {
        <SetRetentionBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding::new(
            DocumentIdentityField::reference(),
            self.input.identity.clone(),
        )
    }
}
