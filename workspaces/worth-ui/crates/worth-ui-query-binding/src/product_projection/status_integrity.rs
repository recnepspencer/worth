use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantExecutionError, WorthQueryApplicationInvariantFieldBinding,
    WorthQueryApplicationInvariantPreparationError, WorthQueryApplicationInvariantRule,
    WorthQueryApplicationInvariantSchemaResolver, WorthQueryApplicationInvariantScopePlanner,
    WorthQueryApplicationInvariantVerdict,
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

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, WorthUiApplicationSchema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        let proposed = planner.proposed();
        let records = proposed.touched_entities(&self.status).map_err(|error| {
            WorthQueryApplicationInvariantPreparationError::new(error.to_string())
        })?;
        for record in &records {
            proposed.field(&self.status, record).map_err(|error| {
                WorthQueryApplicationInvariantPreparationError::new(error.to_string())
            })?;
            proposed.field(&self.revision, record).map_err(|error| {
                WorthQueryApplicationInvariantPreparationError::new(error.to_string())
            })?;
        }
        Ok(records)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, WorthUiApplicationSchema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        let proposed = context.proposed();
        for record in scope {
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
