//! Resolve an exact recorded route against the selected current provenance.
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use worth_relational_contracts::internal_port::publication::data::publication_error::PublicationErrorOwnerPort as _;
use super::{authored_cascade, CodecRetirementClosureStop as Stop};
use crate::execution::request_stage::checkpoint;
use crate::history::data::{CanonicalRequiredOutputOrigin, SelectedOutputPublication};
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture;
use crate::publication::data::{AuthoredRetirementRefusal, PublicationError};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use std::sync::Arc;

pub(super) fn require_current_origin<'a>(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    required: &CanonicalRequiredOutputOrigin,
    current: &'a [SelectedOutputPublication],
) -> Result<&'a Arc<CanonicalRequiredOutputOrigin>, TransactionCommitError> {
    for item in current {
        checkpoint(
            context,
            (required.meaning.family.len() + item.origin.meaning.family.len() + 65) as u64,
        )
        .map_err(|cause| authored_cascade::stopped(Stop::Request(cause.into())))?;
    }
    let origin = current.iter().find(|item| {
        item.origin.meaning.same_route(&required.meaning)
            && item
                .origin
                .members
                .iter()
                .any(|m| m.posture != RequiredOutputRolePosture::Retire)
    });
    origin.map(|item| &item.origin).ok_or_else(|| {
        let owner = required.meaning.source_scope;
        TransactionCommitError::publication(PublicationError::authored_retirement(
            AuthoredRetirementRefusal::PublicationOwnerUnavailable {
                publication: required.commit.commit_id,
                owner,
            },
            RecordRef::Entity(owner),
        ))
    })
}
