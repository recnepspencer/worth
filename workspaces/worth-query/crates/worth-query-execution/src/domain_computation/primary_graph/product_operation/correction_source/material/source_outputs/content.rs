//! Concrete original-output content certification and original-funded attachment.
use super::WorthQueryCorrectionOutputProjectionDenial as Denial;
use crate::domain_computation::primary_graph::{
    content::require_contract, WorthQueryApplicationCommittedChanges,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_schema::ApplicationContentBindingRecipe;
use worth_relational::facade::{
    content::{RelationalContentAttachments, RelationalContentDenial, RelationalContentFrame},
    identity::EntityId,
};
pub(super) fn certify(
    request: &ExecutionRequest<'_, '_>,
    changes: &WorthQueryApplicationCommittedChanges,
    value: &AspectValue,
    entity: EntityId,
    entity_name: &str,
    locator: &AspectFieldLocator,
    recipe: Option<&ApplicationContentBindingRecipe>,
) -> Result<Option<RelationalContentFrame>, Denial> {
    let AspectValue::ContentRef(identity) = value else {
        return Ok(None);
    };
    let recipe = recipe.ok_or(Denial::FieldMeaningMismatch)?;
    let frame = changes
        .content_for_field(request, entity, entity_name, locator)
        .map_err(Denial::Content)?;
    if frame.identity() != *identity {
        return Err(Denial::Content(
            RelationalContentDenial::EncodingInconsistency,
        ));
    }
    require_contract(request, &frame, recipe).map_err(Denial::Content)?;
    Ok(Some(frame))
}
pub(super) fn retain(
    request: &ExecutionRequest<'_, '_>,
    count: usize,
    acquire: &mut dyn FnMut(usize) -> Result<Option<RelationalContentFrame>, Denial>,
) -> Result<RelationalContentAttachments, Denial> {
    let mut retained = RelationalContentAttachments::default();
    for index in 0..count {
        crate::domain_computation::vocabulary::primary_graph::retention_work::checkpoint(request)
            .map_err(Denial::Content)?;
        if let Some(frame) = acquire(index)? {
            retained.attach(request, &frame).map_err(Denial::Content)?;
        }
    }
    Ok(retained)
}
