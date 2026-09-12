use worth_relational::facade::runtime::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantPreparationError, CustomInvariantRegistration, CustomInvariantRule,
    CustomInvariantScopePlanner, CustomInvariantVerdict,
};

use super::WorthQueryApplicationInvariantRule;

pub(in crate::domain_computation::primary_graph) trait ErasedApplicationInvariantRule:
    Send
{
    fn into_registration(
        self: Box<Self>,
        descriptor: CustomInvariantDescriptor,
    ) -> Result<CustomInvariantRegistration, String>;
}

impl<Rule: WorthQueryApplicationInvariantRule> ErasedApplicationInvariantRule for Rule {
    fn into_registration(
        self: Box<Self>,
        descriptor: CustomInvariantDescriptor,
    ) -> Result<CustomInvariantRegistration, String> {
        CustomInvariantRegistration::new(InstalledApplicationInvariantRule {
            rule: *self,
            descriptor,
        })
        .map_err(|denial| format!("{denial:?}"))
    }
}

struct InstalledApplicationInvariantRule<Rule> {
    rule: Rule,
    descriptor: CustomInvariantDescriptor,
}

impl<Rule: WorthQueryApplicationInvariantRule> CustomInvariantRule
    for InstalledApplicationInvariantRule<Rule>
{
    type Scope = Rule::Scope;

    fn descriptor(&self) -> CustomInvariantDescriptor {
        self.descriptor.clone()
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        self.rule.prepare_scope(planner)
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.rule.evaluate(context, scope)
    }
}
