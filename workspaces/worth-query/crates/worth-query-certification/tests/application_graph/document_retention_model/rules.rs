//! The two document-retention rule implementations the Relational invariant
//! owner actually runs.
//!
//! Each rule reads the proposed retention of every touched document and
//! decides on it. Neither knows anything about programs, branches or
//! activation: whether a rule is asked at all is decided for it, and what it
//! answers is decided only by the value in front of it.

use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantExecutionError, WorthQueryApplicationInvariantFieldBinding,
    WorthQueryApplicationInvariantPreparationError, WorthQueryApplicationInvariantRule,
    WorthQueryApplicationInvariantSchemaResolver, WorthQueryApplicationInvariantVerdict,
};

use super::schema::{Document, DocumentRetentionField, DocumentRetentionSchema};

/// The longest retention, in days, `document-retention-v1` admits.
pub const V1_CEILING: u64 = 10;
/// The shortest retention, in days, `document-retention-v2` admits.
pub const V2_FLOOR: u64 = 5;
/// The longest retention, in days, `document-retention-v2` admits.
pub const V2_CEILING: u64 = 20;

type TouchedDocuments =
    Vec<WorthQueryApplicationInvariantEntity<DocumentRetentionSchema, Document>>;
type DocumentRetentionBinding =
    WorthQueryApplicationInvariantFieldBinding<DocumentRetentionSchema, Document, u64>;

/// `document-retention-v1`: a document may not be retained longer than
/// [`V1_CEILING`] days.
pub struct DocumentRetentionV1Rule {
    retention_days: DocumentRetentionBinding,
}

/// `document-retention-v2`: a document must be retained between [`V2_FLOOR`]
/// and [`V2_CEILING`] days inclusive.
pub struct DocumentRetentionV2Rule {
    retention_days: DocumentRetentionBinding,
}

pub fn resolve_first_rule(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, DocumentRetentionSchema>,
) -> Result<DocumentRetentionV1Rule, String> {
    Ok(DocumentRetentionV1Rule {
        retention_days: resolve_retention(resolver)?,
    })
}

pub fn resolve_second_rule(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, DocumentRetentionSchema>,
) -> Result<DocumentRetentionV2Rule, String> {
    Ok(DocumentRetentionV2Rule {
        retention_days: resolve_retention(resolver)?,
    })
}

impl WorthQueryApplicationInvariantRule<DocumentRetentionSchema> for DocumentRetentionV1Rule {
    type Scope = TouchedDocuments;

    fn prepare_scope(
        &self,
        planner: &mut worth_query_host::facade::application_invariants::WorthQueryApplicationInvariantScopePlanner<
            '_,
            '_,
            DocumentRetentionSchema,
        >,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        plan_touched_documents(planner, &self.retention_days)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, DocumentRetentionSchema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        for retention_days in proposed_retentions(context, &self.retention_days, scope)? {
            if !matches!(retention_days, Some(value) if value <= V1_CEILING) {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation);
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}

impl WorthQueryApplicationInvariantRule<DocumentRetentionSchema> for DocumentRetentionV2Rule {
    type Scope = TouchedDocuments;

    fn prepare_scope(
        &self,
        planner: &mut worth_query_host::facade::application_invariants::WorthQueryApplicationInvariantScopePlanner<
            '_,
            '_,
            DocumentRetentionSchema,
        >,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        plan_touched_documents(planner, &self.retention_days)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, DocumentRetentionSchema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        for retention_days in proposed_retentions(context, &self.retention_days, scope)? {
            if !matches!(retention_days, Some(value) if (V2_FLOOR..=V2_CEILING).contains(&value)) {
                return Ok(WorthQueryApplicationInvariantVerdict::Violation);
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}

fn resolve_retention(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, DocumentRetentionSchema>,
) -> Result<DocumentRetentionBinding, String> {
    resolver
        .typed_field(DocumentRetentionField::reference())
        .ok_or_else(|| "the document retention field is not installed".to_owned())
}

/// Declares the touched documents and the one field either rule reads.
fn plan_touched_documents(
    planner: &mut worth_query_host::facade::application_invariants::WorthQueryApplicationInvariantScopePlanner<
        '_,
        '_,
        DocumentRetentionSchema,
    >,
    retention_days: &DocumentRetentionBinding,
) -> Result<TouchedDocuments, WorthQueryApplicationInvariantPreparationError> {
    let proposed = planner.proposed();
    let documents = proposed
        .touched_entities(retention_days)
        .map_err(|error| WorthQueryApplicationInvariantPreparationError::new(error.to_string()))?;
    for document in &documents {
        proposed.field(retention_days, document).map_err(|error| {
            WorthQueryApplicationInvariantPreparationError::new(error.to_string())
        })?;
    }
    Ok(documents)
}

/// Reads the retention each touched document would carry if this candidate landed.
fn proposed_retentions(
    context: &WorthQueryApplicationInvariantContext<'_, '_, DocumentRetentionSchema>,
    retention_days: &DocumentRetentionBinding,
    scope: &TouchedDocuments,
) -> Result<Vec<Option<u64>>, WorthQueryApplicationInvariantExecutionError> {
    let proposed = context.proposed();
    scope
        .iter()
        .map(|document| {
            proposed.field(retention_days, document).map_err(|error| {
                WorthQueryApplicationInvariantExecutionError::new(error.to_string())
            })
        })
        .collect()
}
