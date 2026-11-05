//! Historical publication cannot repair missing authenticated current ownership.
use super::*;
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::RequiredOutputRoutePosition as Position,
    publication::data::{AuthoredRetirementRefusal as Cause, PublicationError},
    runtime::RelationalRuntime,
    transactions::data::{RecordRef, TransactionCommitError},
};
use worth_execution::ExecutionRequest;

pub(crate) fn require_current_owned_route(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    _basis: &AdmittedRelationalBranchBasis,
    meaning: &RequiredOutputPublicationMeaning,
    selected: Option<Position>,
) -> Result<Option<Position>, TransactionCommitError> {
    let Some(Position::Published(publication)) = selected else {
        return Ok(selected);
    };
    let refuse = |cause| {
        TransactionCommitError::publication(PublicationError::authored_retirement(
            cause,
            RecordRef::Entity(meaning.source_scope),
        ))
    };
    let envelope = runtime
        .history
        .canonical_envelope(publication)
        .ok_or_else(|| {
            refuse(Cause::PublicationOwnerUnavailable {
                publication,
                owner: meaning.source_scope,
            })
        })?;
    let owned = execution
        .run_native_with_checkpoint(|context| {
            for origin in envelope.output_origins() {
                crate::publication::ownership::checkpoint(
                    context,
                    (origin.meaning.family.len() + meaning.family.len() + 1) as u64,
                )?;
                if origin.meaning.same_route(meaning) {
                    return Ok(origin.source_scope_owned);
                }
            }
            Err(refuse(Cause::CorruptPublication))
        })
        .map_err(crate::publication::ownership::request)?
        .0?;
    if owned {
        return Err(refuse(Cause::CorruptCurrentOwnership { publication }));
    }
    Ok(selected)
}
