use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::identity::{
    compose_certification_row_digest, compose_forbidden_fallback_audit_digest,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionForbiddenFallbackSeam {
    ConsumeScalarFields,
    DecodeRowPair,
    DecodeRowTriple,
    VerifyScalarAlignment,
    ReadLiveArtifactBundle,
    BindLiveArtifact,
    ReadLiveArtifactBinding,
}

impl ProjectionConsumptionForbiddenFallbackSeam {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConsumeScalarFields => "consume_scalar_fields",
            Self::DecodeRowPair => "decode_retained_row_pair",
            Self::DecodeRowTriple => "decode_retained_row_triple",
            Self::VerifyScalarAlignment => "verify_scalar_alignment",
            Self::ReadLiveArtifactBundle => "read_live_artifact_bundle",
            Self::BindLiveArtifact => "bind_live_artifact",
            Self::ReadLiveArtifactBinding => "read_live_artifact_binding",
        }
    }

    fn needle(&self) -> &'static str {
        match self {
            Self::ConsumeScalarFields => "consume_scalar_fields",
            Self::DecodeRowPair => "decode_retained_row_pair",
            Self::DecodeRowTriple => "decode_retained_row_triple",
            Self::VerifyScalarAlignment => "verify_scalar_alignment",
            Self::ReadLiveArtifactBundle => "read_live_artifact_bundle",
            Self::BindLiveArtifact => "bind_live_artifact",
            Self::ReadLiveArtifactBinding => "read_live_artifact_binding",
        }
    }

    fn all() -> &'static [Self] {
        &[
            Self::ConsumeScalarFields,
            Self::DecodeRowPair,
            Self::DecodeRowTriple,
            Self::VerifyScalarAlignment,
            Self::ReadLiveArtifactBundle,
            Self::BindLiveArtifact,
            Self::ReadLiveArtifactBinding,
        ]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionConsumptionOrdinaryPathSurface {
    CommonPathDx,
}

impl ProjectionConsumptionOrdinaryPathSurface {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CommonPathDx => "common_path_dx",
        }
    }

    fn source(&self) -> &'static str {
        match self {
            Self::CommonPathDx => include_str!("../../dx.rs"),
        }
    }

    fn all() -> &'static [Self] {
        &[Self::CommonPathDx]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionForbiddenFallbackAuditRow {
    ordinary_surface: ProjectionConsumptionOrdinaryPathSurface,
    forbidden_seam: ProjectionConsumptionForbiddenFallbackSeam,
    occurrence_count: usize,
    row_digest: String,
}

impl ProjectionConsumptionForbiddenFallbackAuditRow {
    pub fn ordinary_surface(&self) -> ProjectionConsumptionOrdinaryPathSurface {
        self.ordinary_surface
    }

    pub fn forbidden_seam(&self) -> ProjectionConsumptionForbiddenFallbackSeam {
        self.forbidden_seam
    }

    pub fn occurrence_count(&self) -> usize {
        self.occurrence_count
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionForbiddenFallbackAudit {
    rows: Vec<ProjectionConsumptionForbiddenFallbackAuditRow>,
    total_occurrence_count: usize,
    audit_digest: String,
}

impl ProjectionConsumptionForbiddenFallbackAudit {
    pub fn rows(&self) -> &[ProjectionConsumptionForbiddenFallbackAuditRow] {
        &self.rows
    }

    pub fn total_occurrence_count(&self) -> usize {
        self.total_occurrence_count
    }

    pub fn audit_digest(&self) -> &str {
        &self.audit_digest
    }
}

pub fn projection_consumption_forbidden_fallback_audit(
) -> Result<ProjectionConsumptionForbiddenFallbackAudit, IdentityDerivationDenial> {
    let rows = ProjectionConsumptionOrdinaryPathSurface::all()
        .iter()
        .copied()
        .map(rows_for_surface)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let total_occurrence_count = rows.iter().map(|row| row.occurrence_count()).sum();
    let audit_digest = compose_forbidden_fallback_audit_digest(
        rows.iter().map(|row| row.row_digest()),
        total_occurrence_count,
    )?;
    Ok(ProjectionConsumptionForbiddenFallbackAudit {
        rows,
        total_occurrence_count,
        audit_digest,
    })
}

fn rows_for_surface(
    ordinary_surface: ProjectionConsumptionOrdinaryPathSurface,
) -> Result<Vec<ProjectionConsumptionForbiddenFallbackAuditRow>, IdentityDerivationDenial> {
    Ok(ProjectionConsumptionForbiddenFallbackSeam::all()
        .iter()
        .copied()
        .map(|forbidden_seam| row(ordinary_surface, forbidden_seam))
        .collect::<Result<Vec<_>, _>>()?)
}

fn row(
    ordinary_surface: ProjectionConsumptionOrdinaryPathSurface,
    forbidden_seam: ProjectionConsumptionForbiddenFallbackSeam,
) -> Result<ProjectionConsumptionForbiddenFallbackAuditRow, IdentityDerivationDenial> {
    let occurrence_count = ordinary_surface
        .source()
        .matches(forbidden_seam.needle())
        .count();
    let occurrences = occurrence_count.to_string();
    let row_digest = compose_certification_row_digest(
        "projection_consumption_forbidden_fallback_row_v1",
        &[
            ("surface", ordinary_surface.as_str()),
            ("seam", forbidden_seam.as_str()),
            ("occurrences", occurrences.as_str()),
        ],
    )?;
    Ok(ProjectionConsumptionForbiddenFallbackAuditRow {
        ordinary_surface,
        forbidden_seam,
        occurrence_count,
        row_digest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forbidden_fallback_audit_stays_zero_for_ordinary_path_sources(
    ) -> Result<(), IdentityDerivationDenial> {
        let audit = projection_consumption_forbidden_fallback_audit()?;
        assert_eq!(
            audit.rows().len(),
            ProjectionConsumptionOrdinaryPathSurface::all().len()
                * ProjectionConsumptionForbiddenFallbackSeam::all().len()
        );
        assert_eq!(audit.total_occurrence_count(), 0);
        assert!(audit.rows().iter().all(|row| row.occurrence_count() == 0));
        assert!(!audit.audit_digest().is_empty());

        Ok(())
    }
}
