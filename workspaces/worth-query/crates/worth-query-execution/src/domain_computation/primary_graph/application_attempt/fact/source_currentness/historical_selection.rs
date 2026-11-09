//! The revision-fact owner alone dispatches historical revision correspondence.
use super::WorthQueryApplicationObservedFact as Fact;
use worth_relational::facade::runtime::{
    RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial,
    VisibilityProjectionView,
};
impl Fact {
    /// Supplemental revision comparison after the ordinary fact has moved.
    /// Every other fact keeps its own ordinary currentness contract.
    pub(in crate::domain_computation::primary_graph) fn historical_revision_matches(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        current: &VisibilityProjectionView,
        evidence: &RelationalHistoricalSelectionRead,
    ) -> Result<bool, RelationalHistoricalSelectionReadDenial> {
        match self {
            Self::SourceFieldRevision {
                entity_id,
                locator,
                native_revision: Some(expected),
            } => {
                evidence.matches_field_revision(execution, current, *entity_id, locator, *expected)
            }
            Self::SourceAspectRevision {
                entity_id,
                aspect,
                native_revision,
            } => evidence.matches_aspect_revision(
                execution,
                current,
                *entity_id,
                aspect,
                *native_revision,
            ),
            _ => Ok(false),
        }
    }
}
