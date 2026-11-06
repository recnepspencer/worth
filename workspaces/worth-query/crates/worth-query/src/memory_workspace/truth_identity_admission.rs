use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::sync::Arc;

use worth_foundational::facade::admit_foundational_external_identity_token;

use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::identity_authority::{
    query_truth_identity_admission_authority, QueryCommitIdentityKind, QueryEntityIdentityKind,
    QueryExternalIdentityToken, QuerySnapshotIdentityKind,
    QueryTruthIdentityAdmissionAuthorityIdentity,
};

use super::{WorthQueryCommitIdentity, WorthQueryEntityIdentity, WorthQuerySnapshotIdentity};

fn preview_commit_evidence_from_admitted(
    admitted: &QueryTruthIdentityAdmissionAuthorityIdentity<Arc<str>, QueryCommitIdentityKind>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WriteReceiptCommitIdentity)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_admitted_external_commit_label_v1",
            )
            .field_value(
                WorthQueryEvidenceTag::new("external_commit_label"),
                admitted.value().as_ref(),
            )
            .seal()?,
    )
}

fn preview_snapshot_evidence_from_admitted(
    admitted: &QueryTruthIdentityAdmissionAuthorityIdentity<Arc<str>, QuerySnapshotIdentityKind>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WriteReceiptSnapshotIdentity)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_admitted_external_snapshot_label_v1",
            )
            .field_value(
                WorthQueryEvidenceTag::new("external_snapshot_label"),
                admitted.value().as_ref(),
            )
            .seal()?,
    )
}

fn preview_entity_evidence_from_admitted(
    admitted: &QueryTruthIdentityAdmissionAuthorityIdentity<Arc<str>, QueryEntityIdentityKind>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::AuthoredCommandEntityIdentity)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_admitted_authored_entity_command_v1",
            )
            .field_value(
                WorthQueryEvidenceTag::new("authored_entity_identity"),
                admitted.value().as_ref(),
            )
            .seal()?,
    )
}

pub fn admit_external_commit_token(
    token: QueryExternalIdentityToken<Arc<str>, QueryCommitIdentityKind>,
) -> Result<WorthQueryCommitIdentity, IdentityDerivationDenial> {
    let admitted = admit_foundational_external_identity_token(
        token,
        query_truth_identity_admission_authority(),
    );
    Ok(WorthQueryCommitIdentity::preview(
        preview_commit_evidence_from_admitted(&admitted)?,
    ))
}

pub fn admit_external_snapshot_token(
    token: QueryExternalIdentityToken<Arc<str>, QuerySnapshotIdentityKind>,
) -> Result<WorthQuerySnapshotIdentity, IdentityDerivationDenial> {
    let admitted = admit_foundational_external_identity_token(
        token,
        query_truth_identity_admission_authority(),
    );
    Ok(WorthQuerySnapshotIdentity::preview(
        preview_snapshot_evidence_from_admitted(&admitted)?,
    ))
}

pub fn admit_authored_entity_token(
    token: QueryExternalIdentityToken<Arc<str>, QueryEntityIdentityKind>,
) -> Result<WorthQueryEntityIdentity, IdentityDerivationDenial> {
    let admitted = admit_foundational_external_identity_token(
        token,
        query_truth_identity_admission_authority(),
    );
    Ok(WorthQueryEntityIdentity::preview(
        preview_entity_evidence_from_admitted(&admitted)?,
    ))
}

#[cfg(test)]
pub(crate) fn admit_external_commit_label(
    label: impl AsRef<str>,
) -> Result<WorthQueryCommitIdentity, IdentityDerivationDenial> {
    Ok(admit_external_commit_token(
        QueryExternalIdentityToken::new(Arc::from(label.as_ref())),
    )?)
}

pub(crate) fn admit_external_snapshot_label(
    label: impl AsRef<str>,
) -> Result<WorthQuerySnapshotIdentity, IdentityDerivationDenial> {
    Ok(admit_external_snapshot_token(
        QueryExternalIdentityToken::new(Arc::from(label.as_ref())),
    )?)
}

pub(crate) fn admit_authored_entity_label(
    label: impl AsRef<str>,
) -> Result<WorthQueryEntityIdentity, IdentityDerivationDenial> {
    Ok(admit_authored_entity_token(
        QueryExternalIdentityToken::new(Arc::from(label.as_ref())),
    )?)
}
