//! The installed commit-boundary law evaluates proposed retention, not caller claims.
use super::super::schema::{Retention, Schema, Target};
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::application_invariants::*;

pub struct MaximumRetention;
impl ApplicationInvariantMarkerIdentity<Schema> for MaximumRetention {
    const IDENTIFIER: &'static str = "proposal-retention-maximum-ten";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}
pub fn definition() -> ApplicationInvariantDefinition<Schema, MaximumRetention> {
    ApplicationInvariantDefinition::new(
        MaximumRetention::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            [ApplicationInvariantScopeTarget::Entity("Target".to_owned())],
            [ApplicationInvariantScopeTarget::Entity("Target".to_owned())],
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}
type Field = WorthQueryApplicationInvariantFieldBinding<Schema, Target, u64>;
type Targets = Vec<WorthQueryApplicationInvariantEntity<Schema, Target>>;
pub struct Rule {
    field: Field,
}
pub fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Schema>,
) -> Result<Rule, String> {
    resolver
        .typed_field(Retention::at("Retention"))
        .map(|field| Rule { field })
        .ok_or_else(|| "the declared target retention field is not installed".to_owned())
}
impl WorthQueryApplicationInvariantRule<Schema> for Rule {
    type Refusal = ();
    type RefusalBinding = WorthQueryEmptyInvariantRefusal;
    type Scope = Targets;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Targets, WorthQueryApplicationInvariantError>> {
        Some(self.prepare(planner, Some(budget)))
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Targets,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>,
    > {
        Some(self.evaluate_values(context, scope, Some(budget)))
    }
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
    ) -> Result<Targets, WorthQueryApplicationInvariantError> {
        self.prepare(planner, None)
    }
    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Targets,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        self.evaluate_values(context, scope, None)
    }
}
impl Rule {
    fn prepare(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<Targets, WorthQueryApplicationInvariantError> {
        let stop = || WorthQueryApplicationInvariantError::Stopped;
        if budget.is_some_and(|budget| !budget.checkpoint(0)) {
            return Err(stop());
        }
        let proposed = planner.proposed();
        let targets = proposed
            .touched_entities(&self.field)
            .map_err(WorthQueryApplicationInvariantError::from)?;
        let bytes = targets
            .capacity()
            .checked_mul(std::mem::size_of::<
                WorthQueryApplicationInvariantEntity<Schema, Target>,
            >())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(stop)?;
        if budget.is_some_and(|budget| !budget.claim_result(bytes)) {
            return Err(stop());
        }
        for target in &targets {
            if budget.is_some_and(|budget| !budget.checkpoint(1)) {
                return Err(stop());
            }
            proposed
                .field(&self.field, target)
                .map_err(WorthQueryApplicationInvariantError::from)?;
        }
        Ok(targets)
    }
    fn evaluate_values(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Targets,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<WorthQueryApplicationInvariantVerdict<()>, WorthQueryApplicationInvariantError>
    {
        let stop = || WorthQueryApplicationInvariantError::Stopped;
        if budget.is_some_and(|budget| !budget.checkpoint(0)) {
            return Err(stop());
        }
        for target in scope {
            if budget.is_some_and(|budget| !budget.checkpoint(1)) {
                return Err(stop());
            }
            let value = context
                .proposed()
                .field(&self.field, target)
                .map_err(WorthQueryApplicationInvariantError::from)?;
            if !value.is_some_and(|days| days <= 10) {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation(()));
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
