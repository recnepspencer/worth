//! Whether a selected output entity is a live entity of the family's kind
//! at the selecting view (public #429). The `current_output` read and the
//! re-resolution of its `FamilySelection` fact both ask this one question.

use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::runtime::{ProjectionAspectScope, VisibilityProjectionView};
use worth_relational::facade::storage::RecordLifecycleState;
use worth_relational::facade::transactions::RelationalExecutionDenialCause;

use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;

/// Why a selected entity cannot be the family's output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiveSelectionRefusal {
    /// It is absent or not live in the view.
    OutputUnavailable,
    /// It is live but of another entity kind.
    EntityMismatch,
}

/// The live kind and entity name of `entity_id` in `view`, when it is a live
/// `identifier` entity. A request stop is the read's to report.
pub fn live_selection_identity<'layout>(
    view: Option<&VisibilityProjectionView>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity_id: EntityId,
    layout: &'layout WorthQueryPrimaryGraphLayout,
    identifier: &str,
) -> Result<Result<(KindId, &'layout str), LiveSelectionRefusal>, RelationalExecutionDenialCause> {
    let projected = view
        .map(|view| {
            view.entity_record_with_projection_scope(
                execution,
                entity_id,
                ProjectionAspectScope::empty(),
                |record| Some((record.kind_id(), record.lifecycle())),
            )
        })
        .transpose()?
        .flatten()
        .filter(|(_, lifecycle)| *lifecycle == RecordLifecycleState::Live);
    let Some((kind, _)) = projected else {
        return Ok(Err(LiveSelectionRefusal::OutputUnavailable));
    };
    Ok(match layout.entity_name(kind) {
        Some(entity) if entity == identifier => Ok((kind, entity)),
        _ => Err(LiveSelectionRefusal::EntityMismatch),
    })
}
