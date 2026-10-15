//! Separate workflow program over the existing capability contexts.
use super::{
    super::{program::Feature, schema::*},
    bindings::*,
};
use worth_query_host::facade::{
    declaration::application_operation::*, declaration::application_program::*,
};
pub struct Program;
pub struct Completion;
impl ApplicationActionDelivery<CapabilitySchema, GuardedBinding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "capability-workflow.completion";
}
pub type Action = ApplicationActionReference<
    CapabilitySchema,
    ApplicationRootComposition,
    Feature,
    GuardedBinding,
    Completion,
>;
impl ApplicationProgramDefinition<CapabilitySchema> for Program {
    type Contributions = (CapabilityContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("capability-workflow.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<CapabilitySchema, Feature>()
            .action::<GuardedBinding, Completion>()
            .finish_action()
            .native_action::<ControlBinding>()
            .finish_action()
            .native_action::<StartBinding>()
            .finish_action()
            .native_action::<AdvanceBinding>()
            .finish_action()
            .finish()]
    }
}
pub struct Effect(pub SetRetentionInput);
#[derive(Clone)]
pub struct Control(pub SetRetentionInput);
#[derive(Clone)]
pub struct Start(pub SetRetentionInput);
#[derive(Clone)]
pub struct Advance(pub SetRetentionInput);
impl ApplicationMutationIntent<CapabilitySchema> for Effect {
    type Binding = GuardedBinding;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <GuardedBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding {
        <GuardedBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding::new(
            DocumentIdentityField::reference(),
            self.0.identity.clone(),
        )
    }
}
impl ApplicationMutationIntent<CapabilitySchema> for Control {
    type Binding = ControlBinding;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <ControlBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding {
        <ControlBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding::new(
            DocumentIdentityField::reference(),
            self.0.identity.clone(),
        )
    }
}
impl ApplicationMutationIntent<CapabilitySchema> for Advance {
    type Binding = AdvanceBinding;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <AdvanceBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding {
        <AdvanceBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding::new(
            DocumentIdentityField::reference(),
            self.0.identity.clone(),
        )
    }
}
impl ApplicationMutationIntent<CapabilitySchema> for Start {
    type Binding = StartBinding;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <StartBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding {
        <StartBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding::new(
            DocumentIdentityField::reference(),
            self.0.identity.clone(),
        )
    }
}
