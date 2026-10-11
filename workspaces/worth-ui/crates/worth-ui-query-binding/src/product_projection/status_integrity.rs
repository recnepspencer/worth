use worth_query_host::facade::application_invariants::{
    CustomInvariantLeaseBudget, WorthQueryApplicationInvariantContext,
    WorthQueryApplicationInvariantEntity, WorthQueryApplicationInvariantExecutionError,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantPreparationError,
    WorthQueryApplicationInvariantRule, WorthQueryApplicationInvariantSchemaResolver,
    WorthQueryApplicationInvariantScopePlanner, WorthQueryApplicationInvariantVerdict,
};

use crate::declaration::{
    QueryRevisionValueField, QueryTextStatusField, WorthUiApplicationSchema, WorthUiRecord,
};

pub(super) struct WorthUiStatusIntegrityRule {
    status:
        WorthQueryApplicationInvariantFieldBinding<WorthUiApplicationSchema, WorthUiRecord, String>,
    revision:
        WorthQueryApplicationInvariantFieldBinding<WorthUiApplicationSchema, WorthUiRecord, u64>,
}

pub(super) fn resolve_rule(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, WorthUiApplicationSchema>,
) -> Result<WorthUiStatusIntegrityRule, String> {
    Ok(WorthUiStatusIntegrityRule {
        status: resolver
            .typed_field(QueryTextStatusField::reference())
            .ok_or("UI status field is not installed")?,
        revision: resolver
            .typed_field(QueryRevisionValueField::reference())
            .ok_or("UI revision field is not installed")?,
    })
}

impl WorthQueryApplicationInvariantRule<WorthUiApplicationSchema> for WorthUiStatusIntegrityRule {
    type Scope = Vec<WorthQueryApplicationInvariantEntity<WorthUiApplicationSchema, WorthUiRecord>>;

    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WorthUiApplicationSchema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        Some(self.prepare_under(planner, Some(budget)))
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, WorthUiApplicationSchema>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>,
    > {
        Some(self.evaluate_under(context, scope, Some(budget)))
    }

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WorthUiApplicationSchema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        self.prepare_under(planner, None)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, WorthUiApplicationSchema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        self.evaluate_under(context, scope, None)
    }
}

impl WorthUiStatusIntegrityRule {
    fn prepare_under(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WorthUiApplicationSchema>,
        budget: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Result<
        <Self as WorthQueryApplicationInvariantRule<WorthUiApplicationSchema>>::Scope,
        WorthQueryApplicationInvariantPreparationError,
    > {
        if budget.is_some_and(|budget| !budget.checkpoint(0)) {
            return Err(WorthQueryApplicationInvariantPreparationError::new(
                "caller stopped",
            ));
        }
        let proposed = planner.proposed();
        let records = proposed.touched_entities(&self.status).map_err(|error| {
            WorthQueryApplicationInvariantPreparationError::new(error.to_string())
        })?;
        for record in &records {
            if budget.is_some_and(|budget| !budget.checkpoint(1)) {
                return Err(WorthQueryApplicationInvariantPreparationError::new(
                    "caller stopped",
                ));
            }
            proposed.field(&self.status, record).map_err(|error| {
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
        context: &WorthQueryApplicationInvariantContext<'_, '_, WorthUiApplicationSchema>,
        scope: &<Self as WorthQueryApplicationInvariantRule<WorthUiApplicationSchema>>::Scope,
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
            let status = proposed.field(&self.status, record).map_err(|error| {
                WorthQueryApplicationInvariantExecutionError::new(error.to_string())
            })?;
            let revision = proposed.field(&self.revision, record).map_err(|error| {
                WorthQueryApplicationInvariantExecutionError::new(error.to_string())
            })?;
            let valid = matches!(
                (&status, revision),
                (Some(status), Some(revision))
                    if !status.is_empty()
                        && status.len() <= 65_536
                        && (revision > 0 || status == "PENDING")
            );
            if !valid {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation);
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}

#[cfg(test)]
mod lease_tests;
