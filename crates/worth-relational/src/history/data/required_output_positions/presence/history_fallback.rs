//! Historical publication cannot repair missing authenticated current ownership.
use super::*;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::RequiredOutputRoutePosition as Position;
use crate::history::envelope_demand::CanonicalHistoryRead;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::publication::data::PublicationError;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::ExecutionRequest;

pub(crate) fn require_current_owned_route(
    execution: &ExecutionRequest<'_, '_>,
    history: CanonicalHistoryRead<'_>,
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
    let envelope = history.canonical_envelope(publication).ok_or_else(|| {
        refuse(Cause::PublicationOwnerUnavailable {
            publication,
            owner: meaning.source_scope,
        })
    })?;
    let owned = execution
        .run_native_with_checkpoint(|context| {
            for origin in envelope.output_origins() {
                crate::publication::ownership::read_request::checkpoint(
                    context,
                    (origin.meaning.family.len() + meaning.family.len() + 1) as u64,
                )?;
                if origin.meaning.same_route(meaning) {
                    // A fully retired route has no live custody row. Historical
                    // retirement remains the inverse's provenance, not live ownership.
                    return Ok(origin.source_scope_owned && !matches!(origin.publication,
                        crate::history::data::CanonicalOutputPublication::AuthoredRetirement { .. }
                        | crate::history::data::CanonicalOutputPublication::ProducerRetirement { .. }));
                }
            }
            Err(refuse(Cause::CorruptPublication))
        })
        .map_err(crate::publication::ownership::read_request::request)?
        .0?;
    if owned {
        return Err(refuse(Cause::CorruptCurrentOwnership { publication }));
    }
    Ok(selected)
}
