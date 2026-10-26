use worth_relational::facade::runtime::{
    CustomInvariantDescriptor, CustomInvariantExecutionContext, CustomInvariantExecutionError,
    CustomInvariantLeaseBudget, CustomInvariantPreparationError, CustomInvariantRegistration,
    CustomInvariantRule, CustomInvariantScopePlanner, CustomInvariantVerdict,
};

use super::program_selection::{WorthQueryProgramRuleSelection, WorthQueryProgramRuleStanding};
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
        program_selection: Option<WorthQueryProgramRuleSelection>,
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
        program_selection: Option<WorthQueryProgramRuleSelection>,
    ) -> Result<CustomInvariantRegistration, String> {
        super::refusal::maximum::<Rule::RefusalBinding>()?;
        CustomInvariantRegistration::new(InstalledApplicationInvariantRule {
            rule: *self,
            descriptor,
            binding_identity,
            program_selection,
            _schema: std::marker::PhantomData,
        })
        .map_err(|denial| format!("{denial:?}"))
    }
}

struct InstalledApplicationInvariantRule<Schema, Rule> {
    rule: Rule,
    descriptor: CustomInvariantDescriptor,
    binding_identity: ApplicationSchemaBindingIdentity,
    program_selection: Option<WorthQueryProgramRuleSelection>,
    _schema: std::marker::PhantomData<fn() -> Schema>,
}

struct InstalledApplicationInvariantScope<Scope> {
    rule_scope: Scope,
    preparation_reads: super::super::invariant_projection::WorthQueryInvariantProjectionWork,
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
        )
        .map_err(preparation_denial)?;
        let rule_scope = self.rule.prepare_scope(&mut planner)?;
        Ok(InstalledApplicationInvariantScope {
            rule_scope,
            admission: planner.freeze_admission().map_err(preparation_denial)?,
            preparation_reads: planner.reads(),
        })
    }

    fn supports_checked_preparation(&self) -> bool {
        self.rule.supports_checked_preparation()
    }

    fn prepare_scope_checked(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, CustomInvariantPreparationError>> {
        let mut planner = match super::WorthQueryApplicationInvariantScopePlanner::new(
            planner,
            self.binding_identity.clone(),
        ) {
            Ok(planner) => planner,
            Err(denial) => return Some(Err(preparation_denial(denial))),
        };
        self.rule
            .prepare_scope_checked(&mut planner, budget)
            .map(|result| {
                result.and_then(|rule_scope| {
                    Ok(InstalledApplicationInvariantScope {
                        rule_scope,
                        admission: planner.freeze_admission().map_err(preparation_denial)?,
                        preparation_reads: planner.reads(),
                    })
                })
            })
    }

    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        if let Some(selection) = &self.program_selection {
            match selection.standing(context) {
                Ok(WorthQueryProgramRuleStanding::Silent) => {
                    return Some(Ok(CustomInvariantVerdict::Pass))
                }
                Ok(WorthQueryProgramRuleStanding::Governs) => {}
                Err(error) => return Some(Err(error)),
            }
        }
        let context = super::WorthQueryApplicationInvariantContext::new(
            context,
            self.binding_identity.clone(),
            std::sync::Arc::clone(&scope.admission),
            scope.preparation_reads,
        );
        self.rule
            .evaluate_checked(&context, &scope.rule_scope, budget)
            .map(|result| {
                result.and_then(|verdict| {
                    super::refusal::encode::<Rule::RefusalBinding>(verdict, budget)
                })
            })
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        if let Some(selection) = &self.program_selection
            && selection.standing(context)? == WorthQueryProgramRuleStanding::Silent
        {
            return Ok(CustomInvariantVerdict::Pass);
        }
        let refusal_budget = context.refusal_budget();
        let context = super::WorthQueryApplicationInvariantContext::new(
            context,
            self.binding_identity.clone(),
            std::sync::Arc::clone(&scope.admission),
            scope.preparation_reads,
        );
        self.rule
            .evaluate(&context, &scope.rule_scope)
            .and_then(|verdict| {
                super::refusal::encode::<Rule::RefusalBinding>(verdict, refusal_budget)
            })
    }
}

fn preparation_denial(
    denial: super::WorthQueryInvariantAccessDenial,
) -> CustomInvariantPreparationError {
    CustomInvariantPreparationError::new(denial.to_string())
}
