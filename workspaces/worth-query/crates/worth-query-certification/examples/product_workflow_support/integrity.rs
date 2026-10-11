use worth_query_host::facade::application_invariants::{
    CustomInvariantLeaseBudget, WorthQueryApplicationInvariantContext,
    WorthQueryApplicationInvariantEntity, WorthQueryApplicationInvariantExecutionError,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantPreparationError,
    WorthQueryApplicationInvariantRule, WorthQueryApplicationInvariantSchemaResolver,
    WorthQueryApplicationInvariantScopePlanner, WorthQueryApplicationInvariantVerdict,
};

use super::schema::{
    IntentLifecycleField, IntentRevisionField, TemporalHostSchema, TemporalIntent,
};

pub struct TemporalIntegrityRule {
    lifecycle:
        WorthQueryApplicationInvariantFieldBinding<TemporalHostSchema, TemporalIntent, String>,
    revision: WorthQueryApplicationInvariantFieldBinding<TemporalHostSchema, TemporalIntent, u64>,
}

pub fn resolve_rule(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, TemporalHostSchema>,
) -> Result<TemporalIntegrityRule, String> {
    Ok(TemporalIntegrityRule {
        lifecycle: resolver
            .typed_field(IntentLifecycleField::reference())
            .ok_or("temporal lifecycle field is not installed")?,
        revision: resolver
            .typed_field(IntentRevisionField::reference())
            .ok_or("temporal revision field is not installed")?,
    })
}

impl WorthQueryApplicationInvariantRule<TemporalHostSchema> for TemporalIntegrityRule {
    type Scope = Vec<WorthQueryApplicationInvariantEntity<TemporalHostSchema, TemporalIntent>>;

    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, TemporalHostSchema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        Some(self.prepare_under(planner, Some(budget)))
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, TemporalHostSchema>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>,
    > {
        Some(self.evaluate_under(context, scope, Some(budget)))
    }

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, TemporalHostSchema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        self.prepare_under(planner, None)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, TemporalHostSchema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        self.evaluate_under(context, scope, None)
    }
}

impl TemporalIntegrityRule {
    fn prepare_under(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, TemporalHostSchema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        <Self as WorthQueryApplicationInvariantRule<TemporalHostSchema>>::Scope,
        WorthQueryApplicationInvariantPreparationError,
    > {
        if budget.is_some_and(|budget| !budget.checkpoint(0)) {
            return Err(WorthQueryApplicationInvariantPreparationError::new(
                "caller stopped",
            ));
        }
        let proposed = planner.proposed();
        let records = proposed
            .touched_entities(&self.lifecycle)
            .map_err(|error| {
                WorthQueryApplicationInvariantPreparationError::new(error.to_string())
            })?;
        for record in &records {
            if budget.is_some_and(|budget| !budget.checkpoint(1)) {
                return Err(WorthQueryApplicationInvariantPreparationError::new(
                    "caller stopped",
                ));
            }
            proposed.field(&self.lifecycle, record).map_err(|error| {
                WorthQueryApplicationInvariantPreparationError::new(error.to_string())
            })?;
            proposed.field(&self.revision, record).map_err(|error| {
                WorthQueryApplicationInvariantPreparationError::new(error.to_string())
            })?;
        }
        Ok(records)
    }
    fn evaluate_under(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, TemporalHostSchema>,
        scope: &<Self as WorthQueryApplicationInvariantRule<TemporalHostSchema>>::Scope,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        if budget.is_some_and(|budget| !budget.checkpoint(0)) {
            return Err(WorthQueryApplicationInvariantExecutionError::new(
                "caller stopped",
            ));
        }
        let proposed = context.proposed();
        for record in scope {
            if budget.is_some_and(|budget| !budget.checkpoint(1)) {
                return Err(WorthQueryApplicationInvariantExecutionError::new(
                    "caller stopped",
                ));
            }
            let lifecycle = proposed.field(&self.lifecycle, record).map_err(|error| {
                WorthQueryApplicationInvariantExecutionError::new(error.to_string())
            })?;
            let revision = proposed.field(&self.revision, record).map_err(|error| {
                WorthQueryApplicationInvariantExecutionError::new(error.to_string())
            })?;
            let valid = matches!(
                (revision, lifecycle.as_deref()),
                (Some(revision), Some("active" | "completed")) if revision > 0
            );
            if !valid {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation);
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
