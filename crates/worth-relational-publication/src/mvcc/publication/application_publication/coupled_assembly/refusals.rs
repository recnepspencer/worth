//! Typed coupled assembly and provenance refusals.
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::RecordRef;
use worth_relational_contracts::internal_port as contracts;

pub fn reject(
    cause: Cause,
    subject: contracts::identity::data::record_ids::EntityId,
) -> TransactionCommitError {
    TransactionCommitError::publication(
        contracts::publication::data::publication_error::PublicationError::authored_retirement(
            cause,
            RecordRef::Entity(subject),
        ),
    )
}

/// Describe the actual rejected route without changing its admission or cause.
pub fn reject_extra_owner(
    owners: &super::super::RelationalAuthoredCoupledOwners,
    subject: contracts::identity::data::record_ids::EntityId,
    detail: String,
) -> TransactionCommitError {
    let source_kind = match &owners.source {
        super::super::CoupledSource::Edit(_) => "edit",
        super::super::CoupledSource::Retirement { .. } => "retirement",
        super::super::CoupledSource::Restoration { .. } => "restoration",
        super::super::CoupledSource::MemberEditRetirement { .. } => "member-edit-retirement",
        super::super::CoupledSource::MemberEditRestoration { .. } => "member-edit-restoration",
    };
    let mut error =
        contracts::publication::data::publication_error::PublicationError::authored_retirement(
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
pub fn postponed(
    reason: contracts::publication::data::authored_retirement_refusal::CoupledProvenancePostponement,
    subject: contracts::identity::data::record_ids::EntityId,
) -> TransactionCommitError {
    reject(
        Cause::CoupledContinuationProvenancePostponed {
            issue: PROVENANCE_ISSUE,
            reason,
        },
        subject,
    )
}
