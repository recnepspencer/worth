use worth_relational::facade::runtime::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantPreparationError, CustomInvariantRegistration, CustomInvariantRule,
    CustomInvariantScopePlanner, CustomInvariantVerdict,
};

use super::WorthQueryApplicationInvariantRule;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
};

pub(in crate::domain_computation::primary_graph) trait ErasedApplicationInvariantRule<Schema>:
    Send
where
    Schema: ApplicationSchema,
{
    fn into_registration(
        self: Box<Self>,
        descriptor: CustomInvariantDescriptor,
        binding_identity: ApplicationSchemaBindingIdentity,
    ) -> Result<CustomInvariantRegistration, String>;
}

impl<Schema, Rule> ErasedApplicationInvariantRule<Schema> for Rule
where
    Schema: ApplicationSchema,
    Rule: WorthQueryApplicationInvariantRule<Schema>,
{
    fn into_registration(
        self: Box<Self>,
        descriptor: CustomInvariantDescriptor,
        binding_identity: ApplicationSchemaBindingIdentity,
    ) -> Result<CustomInvariantRegistration, String> {
        CustomInvariantRegistration::new(InstalledApplicationInvariantRule {
            rule: *self,
            descriptor,
            binding_identity,
            _schema: std::marker::PhantomData,
        })
        .map_err(|denial| format!("{denial:?}"))
    }
}

struct InstalledApplicationInvariantRule<Schema, Rule> {
    rule: Rule,
    descriptor: CustomInvariantDescriptor,
    binding_identity: ApplicationSchemaBindingIdentity,
    _schema: std::marker::PhantomData<fn() -> Schema>,
}

struct InstalledApplicationInvariantScope<Scope> {
    rule_scope: Scope,
    admission: std::sync::Arc<super::prepared_scope::ApplicationInvariantAdmission>,
}

impl<Schema, Rule> CustomInvariantRule for InstalledApplicationInvariantRule<Schema, Rule>
where
    Schema: ApplicationSchema,
    Rule: WorthQueryApplicationInvariantRule<Schema>,
{
    type Scope = InstalledApplicationInvariantScope<Rule::Scope>;

    fn descriptor(&self) -> CustomInvariantDescriptor {
        self.descriptor.clone()
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        let mut planner = super::WorthQueryApplicationInvariantScopePlanner::new(
            planner,
            self.binding_identity.clone(),
        );
        let rule_scope = self.rule.prepare_scope(&mut planner)?;
        Ok(InstalledApplicationInvariantScope {
            rule_scope,
            admission: planner.freeze_admission(),
        })
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let context = super::WorthQueryApplicationInvariantContext::new(
            context,
            self.binding_identity.clone(),
            std::sync::Arc::clone(&scope.admission),
        );
        self.rule.evaluate(&context, &scope.rule_scope)
    }
}
