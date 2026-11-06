//! One explicit advancement leaves the actual owning root with its caller on every error.
use super::super::*;

pub(crate) fn advance_group(
    group: &mut Group<'_>,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        DocumentRetentionSchema,
    >,
) -> Result<Posture, WorthQueryHistoryGroupDenial> {
    // compiled-example: begin
    let (document, execution) = group.parts();
    let posture = execution.advance(document, request)?;
    if posture == Posture::ReadyToComplete {
        execution.complete(document, request)?;
    }
    Ok(group.posture())
    // compiled-example: end
}
