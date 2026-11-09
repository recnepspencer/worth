use crate::evidence_identity::worth_query_evidence_identity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::WorthQueryRuntimeFacadeFamily;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::evaluation::WorthQuerySupportPinFinding;
use super::observed_row::WorthQueryObservedSupportPin;
use super::requirement::WorthQuerySupportPinRequirement;

pub(crate) fn derive_support_pin_requirement_identity(
    requirement: &WorthQuerySupportPinRequirement,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerSupportPinRequirement)
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                requirement.family().as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("surface"), requirement.surface())
            .field_shape(
                WorthQueryEvidenceTag::new("required_status"),
                requirement.required_status().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("required_teaching_posture"),
                requirement.required_teaching_posture().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("pinned_live_row_digest"),
                requirement.pinned_live_row_digest(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("pinned_snapshot_row_digest"),
                requirement.pinned_snapshot_row_digest(),
            )
            .seal()?,
    )
}

pub(crate) fn derive_support_pin_observed_row_identity(
    observed: &WorthQueryObservedSupportPin,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerSupportPinObservedRow)
            .field_shape(
                WorthQueryEvidenceTag::new("family"),
                observed.family().as_str(),
            )
            .field_shape(WorthQueryEvidenceTag::new("surface"), observed.surface())
            .field_shape(
                WorthQueryEvidenceTag::new("observed_status"),
                observed.observed_status(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("observed_teaching_posture"),
                observed.observed_teaching_posture(),
            )
            .optional_value(
                WorthQueryEvidenceTag::new("observed_live_row_digest"),
                observed.observed_live_row_digest(),
            )
            .seal()?,
    )
}

pub(crate) fn derive_support_pin_contract_identity(
    consumer_name: &str,
    contract_schema_identity: &str,
    pinned_vocabulary_identity: &str,
    support_snapshot_schema_identity: &str,
    source_matrix_digest: &str,
    requirement_identities: &[WorthQueryEvidenceIdentity],
    observed_identities: &[WorthQueryEvidenceIdentity],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerSupportPinContract)
            .field_shape(WorthQueryEvidenceTag::new("consumer_name"), consumer_name)
            .field_value(
                WorthQueryEvidenceTag::new("contract_schema_identity"),
                contract_schema_identity,
            )
            .field_value(
                WorthQueryEvidenceTag::new("pinned_vocabulary_identity"),
                pinned_vocabulary_identity,
            )
            .field_value(
                WorthQueryEvidenceTag::new("support_snapshot_schema_identity"),
                support_snapshot_schema_identity,
            )
            .field_value(
                WorthQueryEvidenceTag::new("source_matrix_digest"),
                source_matrix_digest,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("requirement_count"),
                requirement_identities.len(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("requirement_identity"),
                requirement_identities.iter(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("observed_count"),
                observed_identities.len(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("observed_identity"),
                observed_identities.iter(),
            )
            .seal()?,
    )
}

pub(crate) fn derive_support_pin_contract_document_identity(
    contract_digest: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerSupportPinContractDocument)
            .field_value(
                WorthQueryEvidenceTag::new("contract_digest"),
                contract_digest,
            )
            .seal()?,
    )
}

pub(crate) fn derive_support_pin_finding_identity(
    finding: &WorthQuerySupportPinFinding,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerSupportPinFinding)
            .field_shape(WorthQueryEvidenceTag::new("kind"), finding.kind().as_str())
            .optional_shape(
                WorthQueryEvidenceTag::new("family"),
                finding.family().map(WorthQueryRuntimeFacadeFamily::as_str),
            )
            .field_shape(WorthQueryEvidenceTag::new("surface"), finding.surface())
            .optional_value(WorthQueryEvidenceTag::new("expected"), finding.expected())
            .optional_value(WorthQueryEvidenceTag::new("found"), finding.found())
            .field_bool(WorthQueryEvidenceTag::new("blocking"), finding.blocking())
            .seal()?,
    )
}

pub(crate) fn derive_support_pin_report_identity(
    consumer_name: &str,
    contract_digest: &str,
    observed_schema_identity: &str,
    observed_source_matrix_digest: &str,
    observed_snapshot_digest: &str,
    requirement_count: usize,
    observed_count: usize,
    matched_required_count: usize,
    snapshot_row_count: usize,
    finding_identities: &[WorthQueryEvidenceIdentity],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::ConsumerSupportPinReport)
            .field_shape(WorthQueryEvidenceTag::new("consumer_name"), consumer_name)
            .field_value(
                WorthQueryEvidenceTag::new("contract_digest"),
                contract_digest,
            )
            .field_value(
                WorthQueryEvidenceTag::new("observed_schema_identity"),
                observed_schema_identity,
            )
            .field_value(
                WorthQueryEvidenceTag::new("observed_source_matrix_digest"),
                observed_source_matrix_digest,
            )
            .field_value(
                WorthQueryEvidenceTag::new("observed_snapshot_digest"),
                observed_snapshot_digest,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("requirement_count"),
                requirement_count,
            )
            .field_usize(WorthQueryEvidenceTag::new("observed_count"), observed_count)
            .field_usize(
                WorthQueryEvidenceTag::new("matched_required_count"),
                matched_required_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("snapshot_row_count"),
                snapshot_row_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("finding_count"),
                finding_identities.len(),
            )
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("finding_identity"),
                finding_identities.iter(),
            )
            .seal()?,
    )
}
