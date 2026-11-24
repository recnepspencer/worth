//! Typed coupled assembly and provenance refusals.
use crate::mvcc::publication::application_publication::custody;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use worth_relational_contracts::internal_port::publication::data::publication_error::PublicationErrorOwnerPort as _;

pub(super) fn reject(
    cause: Cause,
    subject: crate::identity::data::EntityId,
) -> TransactionCommitError {
    TransactionCommitError::publication(
        crate::publication::data::PublicationError::authored_retirement(
            cause,
            RecordRef::Entity(subject),
        ),
    )
}

/// Describe the actual rejected route without changing its admission or cause.
pub(super) fn reject_extra_owner(
    owners: &super::super::RelationalAuthoredCoupledOwners,
    subject: crate::identity::data::EntityId,
    detail: String,
) -> TransactionCommitError {
    let source_kind = match &owners.source {
        super::super::CoupledSource::Edit(_) => "edit",
        super::super::CoupledSource::Retirement { .. } => "retirement",
        super::super::CoupledSource::Restoration { .. } => "restoration",
        super::super::CoupledSource::MemberEditRetirement { .. } => "member-edit-retirement",
        super::super::CoupledSource::MemberEditRestoration { .. } => "member-edit-restoration",
    };
    let mut error = crate::publication::data::PublicationError::authored_retirement(
        Cause::ExtraCoupledOwner,
        RecordRef::Entity(subject),
    );
    error.detail.push_str(&format!(
        "; {detail}; source_kind={source_kind}; source_root={:?}",
        owners.source.subject(),
    ));
    TransactionCommitError::publication(error)
}

/// Publishing a second producing round needs multi-input provenance (public #119).
const PROVENANCE_ISSUE: u16 = 119;

/// The one #119 postponement refusal, for a continued round or a successor fold.
pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn postponed(
    reason: crate::publication::data::CoupledProvenancePostponement,
    subject: crate::identity::data::EntityId,
) -> TransactionCommitError {
    reject(
        Cause::CoupledContinuationProvenancePostponed {
            issue: PROVENANCE_ISSUE,
            reason,
        },
        subject,
    )
}

pub(super) fn overflow() -> TransactionCommitError {
    custody::execution_stop(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
