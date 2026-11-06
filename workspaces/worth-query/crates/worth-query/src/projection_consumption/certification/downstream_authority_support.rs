use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::identity::{
    compose_certification_row_digest, compose_digest_sequence,
};
use crate::projection_consumption::{
    discover_projection_consumption_support, ProjectionConsumptionSupportPosture,
    ProjectionFactKind, ProjectionSourceFamily,
};

use super::audits::{representative_source, ProjectionConsumptionCertifiedSourceSurface};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConsumedProjectionAuthoritySupportStatus {
    Admitted,
    AdmittedWithWarnings,
    Deferred,
    SourceMismatch,
}

impl ConsumedProjectionAuthoritySupportStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::AdmittedWithWarnings => "admitted_with_warnings",
            Self::Deferred => "deferred",
            Self::SourceMismatch => "source_mismatch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumedProjectionAuthoritySupportRow {
    surface: ProjectionConsumptionCertifiedSourceSurface,
    source_family: ProjectionSourceFamily,
    status: ConsumedProjectionAuthoritySupportStatus,
    admitted_fact_kinds: Vec<ProjectionFactKind>,
    row_digest: String,
}

impl ConsumedProjectionAuthoritySupportRow {
    pub fn surface(&self) -> ProjectionConsumptionCertifiedSourceSurface {
        self.surface
    }

    pub fn source_family(&self) -> ProjectionSourceFamily {
        self.source_family
    }

    pub fn status(&self) -> ConsumedProjectionAuthoritySupportStatus {
        self.status
    }

    pub fn admitted_fact_kinds(&self) -> &[ProjectionFactKind] {
        &self.admitted_fact_kinds
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumedProjectionAuthoritySupportMatrix {
    rows: Vec<ConsumedProjectionAuthoritySupportRow>,
    matrix_digest: String,
}

impl ConsumedProjectionAuthoritySupportMatrix {
    pub fn rows(&self) -> &[ConsumedProjectionAuthoritySupportRow] {
        &self.rows
    }

    pub fn matrix_digest(&self) -> &str {
        &self.matrix_digest
    }

    pub fn row(
        &self,
        surface: ProjectionConsumptionCertifiedSourceSurface,
    ) -> Option<&ConsumedProjectionAuthoritySupportRow> {
        self.rows.iter().find(|row| row.surface == surface)
    }
}

pub fn consumed_projection_authority_support_matrix(
) -> Result<ConsumedProjectionAuthoritySupportMatrix, IdentityDerivationDenial> {
    let rows = ProjectionConsumptionCertifiedSourceSurface::all()
        .iter()
        .copied()
        .map(support_row)
        .collect::<Result<Vec<_>, _>>()?;
    let matrix_digest = compose_digest_sequence(
        "consumed_projection_authority_support_matrix_v1",
        "row",
        rows.iter().map(|row| row.row_digest.clone()),
    )?;
    Ok(ConsumedProjectionAuthoritySupportMatrix {
        rows,
        matrix_digest,
    })
}

fn support_row(
    surface: ProjectionConsumptionCertifiedSourceSurface,
) -> Result<ConsumedProjectionAuthoritySupportRow, IdentityDerivationDenial> {
    let source = representative_source(surface)?;
    let report = discover_projection_consumption_support(&source)?;
    let admitted_fact_kinds = report
        .rows()
        .iter()
        .filter(|row| {
            matches!(
                row.posture(),
                ProjectionConsumptionSupportPosture::Admitted
                    | ProjectionConsumptionSupportPosture::AdmittedWithWarnings(_)
            )
        })
        .map(|row| row.fact_kind())
        .collect::<Vec<_>>();
    let status = aggregate_status(report.rows(), &admitted_fact_kinds);
    let admitted_shape = admitted_fact_kinds
        .iter()
        .map(ProjectionFactKind::as_str)
        .collect::<Vec<_>>()
        .join(",");
    let row_digest = compose_certification_row_digest(
        "consumed_projection_authority_support_row_v1",
        &[
            ("surface", surface.as_str()),
            ("source_family", source.family().as_str()),
            ("status", status.as_str()),
            ("admitted_facts", admitted_shape.as_str()),
        ],
    )?;
    Ok(ConsumedProjectionAuthoritySupportRow {
        surface,
        source_family: source.family(),
        status,
        admitted_fact_kinds,
        row_digest,
    })
}

fn aggregate_status(
    rows: &[crate::projection_consumption::ProjectionConsumptionSupportRow],
    admitted: &[ProjectionFactKind],
) -> ConsumedProjectionAuthoritySupportStatus {
    if !admitted.is_empty() {
        if rows.iter().any(|row| {
            matches!(
                row.posture(),
                ProjectionConsumptionSupportPosture::AdmittedWithWarnings(_)
            )
        }) {
            ConsumedProjectionAuthoritySupportStatus::AdmittedWithWarnings
        } else {
            ConsumedProjectionAuthoritySupportStatus::Admitted
        }
    } else if rows.iter().any(|row| {
        matches!(
            row.posture(),
            ProjectionConsumptionSupportPosture::Deferred(_)
        )
    }) {
        ConsumedProjectionAuthoritySupportStatus::Deferred
    } else {
        ConsumedProjectionAuthoritySupportStatus::SourceMismatch
    }
}
