//! Once-only typed attachments to the action being authored.
use super::ApplicationActionSpecBuilder;
use crate::{
    application_operation::ApplicationMutationBinding,
    application_program::{
        ApplicationChangeShape, ApplicationCompositionInstance,
        ApplicationEvaluatedRequirementRule, ApplicationExternalInputProvider, ApplicationFeature,
        ApplicationLocalityScope, ApplicationRepeatedOptionalMemberCorrespondence,
    },
    application_schema::ApplicationSchema,
};

// Public for inferred return types, but this private module exposes no caller
// constructors or state vocabulary through the audience facade.
pub struct Unconfigured;
pub struct Configured;

impl<S, I, F, B, C, R, P> ApplicationActionSpecBuilder<S, I, F, B, Unconfigured, C, R, P>
where
    S: ApplicationSchema + 'static,
    I: ApplicationCompositionInstance,
    F: ApplicationFeature<S>,
    B: ApplicationMutationBinding<S>,
{
    /// Attach the paired locality/change contract exactly once.
    pub fn locality_and_change<Scope, Shape>(
        mut self,
    ) -> ApplicationActionSpecBuilder<S, I, F, B, Configured, C, R, P>
    where
        Scope: ApplicationLocalityScope,
        Shape: ApplicationChangeShape,
    {
        self.action.attach_locality_and_change::<Scope, Shape>();
        self.transition()
    }
}

impl<S, I, F, B, L, R, P> ApplicationActionSpecBuilder<S, I, F, B, L, Unconfigured, R, P>
where
    S: ApplicationSchema + 'static,
    I: ApplicationCompositionInstance,
    F: ApplicationFeature<S>,
    B: ApplicationMutationBinding<S>,
{
    /// Attach the binding-scoped correspondence contract exactly once.
    pub fn correspondence<Correspondence>(
        mut self,
    ) -> ApplicationActionSpecBuilder<S, I, F, B, L, Configured, R, P>
    where
        Correspondence: ApplicationRepeatedOptionalMemberCorrespondence<S, B>,
    {
        self.action
            .attach_correspondence::<Correspondence>(Correspondence::IDENTITY);
        self.transition()
    }
}

impl<S, I, F, B, L, C, P> ApplicationActionSpecBuilder<S, I, F, B, L, C, Unconfigured, P>
where
    S: ApplicationSchema + 'static,
    I: ApplicationCompositionInstance,
    F: ApplicationFeature<S>,
    B: ApplicationMutationBinding<S>,
{
    /// Attach an operation-scoped evaluated rule, independently of a provider.
    pub fn evaluated_requirement<Rule>(
        mut self,
    ) -> ApplicationActionSpecBuilder<S, I, F, B, L, C, Configured, P>
    where
        Rule: ApplicationEvaluatedRequirementRule<S, B::Operation>,
    {
        self.action
            .attach_evaluated_requirement::<Rule>(Rule::IDENTITY);
        self.transition()
    }
}

impl<S, I, F, B, L, C, R> ApplicationActionSpecBuilder<S, I, F, B, L, C, R, Unconfigured>
where
    S: ApplicationSchema + 'static,
    I: ApplicationCompositionInstance,
    F: ApplicationFeature<S>,
    B: ApplicationMutationBinding<S>,
{
    /// Attach an operation-scoped provider, independently of an evaluated rule.
    pub fn external_input<Provider>(
        mut self,
    ) -> ApplicationActionSpecBuilder<S, I, F, B, L, C, R, Configured>
    where
        Provider: ApplicationExternalInputProvider<S, B::Operation>,
    {
        self.action
            .attach_external_input::<Provider>(Provider::IDENTITY);
        self.transition()
    }
}
