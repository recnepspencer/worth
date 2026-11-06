use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::super::inventory::CausalEvidenceFamily;
use super::super::materialization::QueryCausalInspectionArtifact;
use super::error::{CausalInspectionCertificationError, CausalInspectionCertificationErrorKind};
use super::failure_evidence::CausalInspectionCertificationFailureEvidence;
use super::matrix_kind::CausalInspectionRepresentativeKind;
use super::matrix_validation::{
    validate_failure_evidence_kind, validate_failure_kind, validate_kind_matches_artifact,
    validate_missing_evidence_kind, validate_required_representatives,
};
use super::row_digest::CausalInspectionRepresentativeRowDigestSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionRepresentativeEvidence {
    kind: CausalInspectionRepresentativeKind,
    artifact_digest: Option<String>,
    failure_digest: Option<String>,
    missing_evidence_family: Option<CausalEvidenceFamily>,
    row_digest_set: CausalInspectionRepresentativeRowDigestSet,
    representative_digest: String,
}

impl CausalInspectionRepresentativeEvidence {
    pub fn from_query_artifact(
        kind: CausalInspectionRepresentativeKind,
        artifact: &QueryCausalInspectionArtifact,
    ) -> Result<Self, CausalInspectionCertificationError> {
        let artifact_digest = artifact.artifact_identity().as_str().to_string();
        let row_digest_set =
            CausalInspectionRepresentativeRowDigestSet::from_query_artifact(kind, artifact)?;
        validate_kind_matches_artifact(kind, artifact, &row_digest_set)?;
        let representative_digest = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "causal_inspection_representative_evidence_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("artifact"),
            artifact.artifact_identity().evidence_identity(),
        )
        .field_shape(WorthQueryEvidenceTag::new("failure"), "none")
        .field_usize(
            WorthQueryEvidenceTag::new("reference_count"),
            artifact.evidence_reference_count(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("row_digest_set"),
            row_digest_set.row_digest(),
        )
        .seal()?
        .as_str()
        .to_string();
        Ok(Self {
            kind,
            artifact_digest: Some(artifact_digest),
            failure_digest: None,
            missing_evidence_family: None,
            row_digest_set,
            representative_digest,
        })
    }

    pub fn from_missing_evidence(
        kind: CausalInspectionRepresentativeKind,
        family: CausalEvidenceFamily,
        failure_digest: &str,
    ) -> Result<Self, CausalInspectionCertificationError> {
        validate_non_empty_failure(kind, failure_digest)?;
        validate_missing_evidence_kind(kind, family)?;
        let row_digest_set = CausalInspectionRepresentativeRowDigestSet::from_missing_evidence(
            kind,
            family,
            failure_digest,
        )?;
        Ok(Self::failure_row(
            kind,
            Some(family),
            failure_digest,
            row_digest_set,
        )?)
    }

    pub fn from_failure(
        kind: CausalInspectionRepresentativeKind,
        failure_digest: &str,
    ) -> Result<Self, CausalInspectionCertificationError> {
        validate_non_empty_failure(kind, failure_digest)?;
        validate_failure_kind(kind)?;
        let expected = CausalInspectionCertificationFailureEvidence::for_representative_kind(kind)?;
        if expected.failure_digest() != failure_digest {
            return Err(CausalInspectionCertificationError::new(
                CausalInspectionCertificationErrorKind::RepresentativeMatrixMismatch,
                "failure representative digest must come from typed certification failure evidence",
                &[
                    format!("kind:{}", kind.as_str()),
                    format!("expected:{}", expected.failure_digest()),
                    format!("actual:{failure_digest}"),
                ],
            ));
        }
        let row_digest_set = CausalInspectionRepresentativeRowDigestSet::from_failure(
            kind,
            expected.kind().as_str(),
            failure_digest,
        )?;
        Ok(Self::failure_row(
            kind,
            None,
            failure_digest,
            row_digest_set,
        )?)
    }

    pub fn from_failure_evidence(
        kind: CausalInspectionRepresentativeKind,
        evidence: &CausalInspectionCertificationFailureEvidence,
    ) -> Result<Self, CausalInspectionCertificationError> {
        validate_failure_kind(kind)?;
        validate_failure_evidence_kind(kind, evidence)?;
        let row_digest_set = CausalInspectionRepresentativeRowDigestSet::from_failure(
            kind,
            evidence.kind().as_str(),
            evidence.failure_digest(),
        )?;
        Ok(Self::failure_row(
            kind,
            None,
            evidence.failure_digest(),
            row_digest_set,
        )?)
    }

    fn failure_row(
        kind: CausalInspectionRepresentativeKind,
        family: Option<CausalEvidenceFamily>,
        failure_digest: &str,
        row_digest_set: CausalInspectionRepresentativeRowDigestSet,
    ) -> Result<Self, IdentityDerivationDenial> {
        let representative_digest = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "causal_inspection_representative_evidence_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_shape(WorthQueryEvidenceTag::new("artifact"), "none")
        .field_shape(WorthQueryEvidenceTag::new("failure"), failure_digest)
        .field_shape(
            WorthQueryEvidenceTag::new("missing_evidence_family"),
            family.map_or("none", |family| family.as_str()),
        )
        .field_usize(WorthQueryEvidenceTag::new("reference_count"), 0)
        .field_value(
            WorthQueryEvidenceTag::new("row_digest_set"),
            row_digest_set.row_digest(),
        )
        .seal()?
        .as_str()
        .to_string();
        Ok(Self {
            kind,
            artifact_digest: None,
            failure_digest: Some(failure_digest.to_string()),
            missing_evidence_family: family,
            row_digest_set,
            representative_digest,
        })
    }

    pub fn kind(&self) -> CausalInspectionRepresentativeKind {
        self.kind
    }

    pub fn representative_digest(&self) -> &str {
        &self.representative_digest
    }

    pub fn artifact_digest(&self) -> Option<&str> {
        self.artifact_digest.as_deref()
    }

    pub fn failure_digest(&self) -> Option<&str> {
        self.failure_digest.as_deref()
    }

    pub fn missing_evidence_family(&self) -> Option<CausalEvidenceFamily> {
        self.missing_evidence_family
    }

    pub fn row_digest_set(&self) -> &CausalInspectionRepresentativeRowDigestSet {
        &self.row_digest_set
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CausalInspectionRepresentativeMatrix {
    representative_count: usize,
    missing_evidence_row_count: usize,
    query_only_consumer_row_count: usize,
    representative_digests: Vec<String>,
    row_digest_set_digests: Vec<String>,
    missing_evidence_families: Vec<CausalEvidenceFamily>,
    matrix_digest: String,
}

impl CausalInspectionRepresentativeMatrix {
    pub fn from_representatives(
        representatives: &[CausalInspectionRepresentativeEvidence],
    ) -> Result<Self, CausalInspectionCertificationError> {
        validate_required_representatives(representatives)?;
        let representative_count = representatives.len();
        let missing_evidence_row_count = representatives
            .iter()
            .filter(|row| row.missing_evidence_family().is_some())
            .count();
        let query_only_consumer_row_count = representatives
            .iter()
            .filter(|row| {
                row.kind() == CausalInspectionRepresentativeKind::WorthStyleQueryOnlyConsumer
            })
            .count();
        let representative_digests = representatives
            .iter()
            .map(|row| row.representative_digest().to_string())
            .collect::<Vec<_>>();
        let row_digest_set_digests = representatives
            .iter()
            .map(|row| row.row_digest_set().row_digest().to_string())
            .collect::<Vec<_>>();
        let missing_evidence_families = representatives
            .iter()
            .filter_map(CausalInspectionRepresentativeEvidence::missing_evidence_family)
            .collect::<Vec<_>>();
        let matrix_digest = matrix_digest(
            representatives,
            &row_digest_set_digests,
            &missing_evidence_families,
            representative_count,
            missing_evidence_row_count,
            query_only_consumer_row_count,
        )?;
        Ok(Self {
            representative_count,
            missing_evidence_row_count,
            query_only_consumer_row_count,
            representative_digests,
            row_digest_set_digests,
            missing_evidence_families,
            matrix_digest,
        })
    }

    pub fn representative_count(&self) -> usize {
        self.representative_count
    }

    pub fn missing_evidence_row_count(&self) -> usize {
        self.missing_evidence_row_count
    }

    pub fn query_only_consumer_row_count(&self) -> usize {
        self.query_only_consumer_row_count
    }

    pub fn representative_digests(&self) -> &[String] {
        &self.representative_digests
    }

    pub fn row_digest_set_digests(&self) -> &[String] {
        &self.row_digest_set_digests
    }

    pub fn missing_evidence_families(&self) -> &[CausalEvidenceFamily] {
        &self.missing_evidence_families
    }

    pub fn matrix_digest(&self) -> &str {
        &self.matrix_digest
    }
}

fn matrix_digest(
    rows: &[CausalInspectionRepresentativeEvidence],
    row_digest_set_digests: &[String],
    missing_evidence_families: &[CausalEvidenceFamily],
    representative_count: usize,
    missing_evidence_row_count: usize,
    query_only_consumer_row_count: usize,
) -> Result<String, IdentityDerivationDenial> {
    Ok(WorthQueryEvidenceIdentity::compose(
        WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
    )
    .field_shape(
        WorthQueryEvidenceTag::new("identity_family"),
        "causal_inspection_representative_matrix_v1",
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("representative"),
        rows.iter()
            .map(CausalInspectionRepresentativeEvidence::representative_digest),
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("row_digest_set"),
        row_digest_set_digests.iter(),
    )
    .field_value_sequence(
        WorthQueryEvidenceTag::new("missing_evidence_family"),
        missing_evidence_families
            .iter()
            .map(CausalEvidenceFamily::as_str),
    )
    .field_usize(
        WorthQueryEvidenceTag::new("representative_count"),
        representative_count,
    )
    .field_usize(
        WorthQueryEvidenceTag::new("missing_evidence_count"),
        missing_evidence_row_count,
    )
    .field_usize(
        WorthQueryEvidenceTag::new("query_only_count"),
        query_only_consumer_row_count,
    )
    .seal()?
    .as_str()
    .to_string())
}

fn validate_non_empty_failure(
    kind: CausalInspectionRepresentativeKind,
    failure_digest: &str,
) -> Result<(), CausalInspectionCertificationError> {
    if failure_digest.is_empty() {
        return Err(CausalInspectionCertificationError::new(
            CausalInspectionCertificationErrorKind::RepresentativeMatrixMismatch,
            "failure representatives require a non-empty typed failure digest",
            &[
                format!("kind:{}", kind.as_str()),
                format!("failure-empty:{}", failure_digest.is_empty()),
            ],
        ));
    }
    Ok(())
}
