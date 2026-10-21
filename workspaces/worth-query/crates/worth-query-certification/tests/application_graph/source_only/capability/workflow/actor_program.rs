//! Dedicated actor-policy program leaves the original completion court unchanged.
use super::super::{program::Feature, schema::*};
use super::{actor_binding::ActorBinding, bindings::*};
use worth_query_host::facade::{
    declaration::application_capability::ApplicationWorkflowRecordedActorSource,
    declaration::application_operation::*, declaration::application_program::*,
};
pub struct Program;
pub struct Spec;
impl ApplicationWorkflowSpec for Spec {
    type Schema = CapabilitySchema;
    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("capability-workflow.captured-actor.v1");
    fn observation_policy() -> ApplicationWorkflowObservationPolicy {
        ApplicationWorkflowObservationPolicy::disabled()
    }
    fn actor_policy() -> ApplicationWorkflowActorPolicy {
        ApplicationWorkflowActorPolicy::for_operation::<Self, ActorBinding>(
            ApplicationWorkflowRecordedActorSource::BoundProposal,
        )
    }
}
pub struct Completion;
impl ApplicationActionDelivery<CapabilitySchema, ActorBinding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "capability-workflow.actor-completion";
}
pub type Action = ApplicationActionReference<
    CapabilitySchema,
    ApplicationRootComposition,
    Feature,
    ActorBinding,
    Completion,
>;
impl ApplicationProgramDefinition<CapabilitySchema> for Program {
    type Contributions = (CapabilityContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("capability-workflow.actor-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<CapabilitySchema, Feature>()
            .action::<ActorBinding, Completion>()
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
impl ApplicationMutationIntent<CapabilitySchema> for Effect {
    type Binding = ActorBinding;
    fn input(&self) -> &SetRetentionInput {
        &self.0
    }
    fn scope_binding(
        &self,
    ) -> <ActorBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding {
        <ActorBinding as ApplicationMutationBinding<CapabilitySchema>>::ScopeBinding::new(
            DocumentIdentityField::reference(),
            self.0.identity.clone(),
        )
    }
}
