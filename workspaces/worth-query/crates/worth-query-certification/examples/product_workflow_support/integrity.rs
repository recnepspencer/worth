use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantExecutionError, WorthQueryApplicationInvariantFieldBinding,
    WorthQueryApplicationInvariantPreparationError, WorthQueryApplicationInvariantRule,
    WorthQueryApplicationInvariantSchemaResolver, WorthQueryApplicationInvariantScopePlanner,
    WorthQueryApplicationInvariantVerdict,
};

use super::schema::{
    IntentLifecycleField, IntentRevisionField, TemporalHostSchema, TemporalIntent,
};

pub struct TemporalIntegrityRule {
    revision: WorthQueryApplicationInvariantFieldBinding<TemporalHostSchema, TemporalIntent, u64>,
    lifecycle:
        WorthQueryApplicationInvariantFieldBinding<TemporalHostSchema, TemporalIntent, String>,
}

pub fn resolve_rule(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, TemporalHostSchema>,
) -> Result<TemporalIntegrityRule, String> {
    Ok(TemporalIntegrityRule {
        revision: resolver
            .typed_field(IntentRevisionField::reference())
            .ok_or("temporal revision field is not installed")?,
        lifecycle: resolver
            .typed_field(IntentLifecycleField::reference())
            .ok_or("temporal lifecycle field is not installed")?,
    })
}

impl WorthQueryApplicationInvariantRule<TemporalHostSchema> for TemporalIntegrityRule {
    type Scope = Vec<WorthQueryApplicationInvariantEntity<TemporalHostSchema, TemporalIntent>>;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, TemporalHostSchema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        let proposed = planner.proposed();
        let records = proposed.touched_entities(&self.revision).map_err(|error| {
            WorthQueryApplicationInvariantPreparationError::new(error.to_string())
        })?;
        for record in &records {
            proposed.field(&self.revision, record).map_err(|error| {
                WorthQueryApplicationInvariantPreparationError::new(error.to_string())
            })?;
            proposed.field(&self.lifecycle, record).map_err(|error| {
                WorthQueryApplicationInvariantPreparationError::new(error.to_string())
            })?;
        }
        Ok(records)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, TemporalHostSchema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        let proposed = context.proposed();
        for record in scope {
            let revision = proposed.field(&self.revision, record).map_err(|error| {
                WorthQueryApplicationInvariantExecutionError::new(error.to_string())
            })?;
            let lifecycle = proposed.field(&self.lifecycle, record).map_err(|error| {
                WorthQueryApplicationInvariantExecutionError::new(error.to_string())
            })?;
            if !matches!(
                (revision, lifecycle.as_deref()),
                (Some(revision), Some("active" | "completed")) if revision > 0
            ) {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation);
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
