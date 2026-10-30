use crate::projection_consumption::identity::{
    compose_certification_row_digest, compose_certified_surface_representative_digest,
    compose_digest_sequence, compose_support_traceability_digest,
    compose_support_traceability_row_digest,
};

use super::super::proof_artifacts::ProjectionConsumptionCompileFailProof;
use super::surfaces::{
    representative_source, traceability_for, ProjectionConsumptionCertifiedSourceSurface,
};
use crate::projection_consumption::facts::ProjectionFactKind;
use crate::projection_consumption::source::{ProjectionConsumptionSource, ProjectionSourceFamily};
use crate::projection_consumption::support::{
    discover_projection_consumption_support, ProjectionConsumptionSupportPosture,
    ProjectionConsumptionSupportRow,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionFamilyInventoryRow {
    certified_surface: ProjectionConsumptionCertifiedSourceSurface,
    source_family: ProjectionSourceFamily,
    representative_digest: String,
    row_digest: String,
}

impl ProjectionConsumptionFamilyInventoryRow {
    pub fn certified_surface(&self) -> ProjectionConsumptionCertifiedSourceSurface {
        self.certified_surface
    }

    pub fn source_family(&self) -> ProjectionSourceFamily {
        self.source_family
    }

    pub fn representative_digest(&self) -> &str {
        &self.representative_digest
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionFamilyInventory {
    rows: Vec<ProjectionConsumptionFamilyInventoryRow>,
    inventory_digest: String,
}

impl ProjectionConsumptionFamilyInventory {
    pub fn rows(&self) -> &[ProjectionConsumptionFamilyInventoryRow] {
        &self.rows
    }

    pub fn inventory_digest(&self) -> &str {
        &self.inventory_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionSupportMatrixRow {
    certified_surface: ProjectionConsumptionCertifiedSourceSurface,
    source_family: ProjectionSourceFamily,
    fact_kind: ProjectionFactKind,
    posture: ProjectionConsumptionSupportPosture,
    support_digest: String,
    admission_rule: &'static str,
    hostile_neighbor: &'static str,
    certification_lane: &'static str,
    structural_proof: ProjectionConsumptionCompileFailProof,
    row_digest: String,
}

impl ProjectionConsumptionSupportMatrixRow {
    pub fn certified_surface(&self) -> ProjectionConsumptionCertifiedSourceSurface {
        self.certified_surface
    }

    pub fn source_family(&self) -> ProjectionSourceFamily {
        self.source_family
    }

    pub fn fact_kind(&self) -> ProjectionFactKind {
        self.fact_kind
    }

    pub fn posture(&self) -> &ProjectionConsumptionSupportPosture {
        &self.posture
    }

    pub fn support_digest(&self) -> &str {
        &self.support_digest
    }

    pub fn admission_rule(&self) -> &'static str {
        self.admission_rule
    }

    pub fn hostile_neighbor(&self) -> &'static str {
        self.hostile_neighbor
    }

    pub fn certification_lane(&self) -> &'static str {
        self.certification_lane
    }

    pub fn structural_proof(&self) -> ProjectionConsumptionCompileFailProof {
        self.structural_proof
    }

    pub fn row_digest(&self) -> &str {
        &self.row_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionSupportMatrix {
    rows: Vec<ProjectionConsumptionSupportMatrixRow>,
    matrix_digest: String,
    support_traceability_digest: String,
}

impl ProjectionConsumptionSupportMatrix {
    pub fn rows(&self) -> &[ProjectionConsumptionSupportMatrixRow] {
        &self.rows
    }

    pub fn matrix_digest(&self) -> &str {
        &self.matrix_digest
    }

    pub fn support_traceability_digest(&self) -> &str {
        &self.support_traceability_digest
    }
}

pub fn projection_consumption_family_inventory() -> ProjectionConsumptionFamilyInventory {
    let rows = ProjectionConsumptionCertifiedSourceSurface::all()
        .iter()
        .copied()
        .map(inventory_row)
        .collect::<Vec<_>>();
    let inventory_digest = compose_digest_sequence(
        "projection_consumption_family_inventory_v1",
        "row",
        rows.iter().map(|row| row.row_digest().to_string()),
    );
    ProjectionConsumptionFamilyInventory {
        rows,
        inventory_digest,
    }
}

pub fn projection_consumption_support_matrix() -> ProjectionConsumptionSupportMatrix {
    let rows = ProjectionConsumptionCertifiedSourceSurface::all()
        .iter()
        .copied()
        .flat_map(matrix_rows_for_surface)
        .collect::<Vec<_>>();
    let matrix_digest = compose_digest_sequence(
        "projection_consumption_support_matrix_v1",
        "row",
        rows.iter().map(|row| row.row_digest().to_string()),
    );
    let support_traceability_digest = compose_support_traceability_digest(rows.iter().map(|row| {
        compose_support_traceability_row_digest(
            row.admission_rule(),
            row.hostile_neighbor(),
            row.certification_lane(),
            row.structural_proof().as_str(),
        )
    }));
    ProjectionConsumptionSupportMatrix {
        rows,
        matrix_digest,
        support_traceability_digest,
    }
}

fn inventory_row(
    surface: ProjectionConsumptionCertifiedSourceSurface,
) -> ProjectionConsumptionFamilyInventoryRow {
    let source = representative_source(surface);
    let representative_digest = compose_certified_surface_representative_digest(
        surface.as_str(),
        source.family(),
        source.source_identity(),
    );
    let row_digest = compose_certification_row_digest(
        "projection_consumption_family_inventory_row_v1",
        &[
            ("surface", surface.as_str()),
            ("family", source.family().as_str()),
            ("representative", representative_digest.as_str()),
        ],
    );
    ProjectionConsumptionFamilyInventoryRow {
        certified_surface: surface,
        source_family: source.family(),
        representative_digest,
        row_digest,
    }
}

fn matrix_rows_for_surface(
    surface: ProjectionConsumptionCertifiedSourceSurface,
) -> Vec<ProjectionConsumptionSupportMatrixRow> {
    let source = representative_source(surface);
    ProjectionFactKind::all()
        .iter()
        .copied()
        .map(|fact_kind| matrix_row(surface, &source, fact_kind))
        .collect()
}

fn matrix_row(
    surface: ProjectionConsumptionCertifiedSourceSurface,
    source: &ProjectionConsumptionSource,
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportMatrixRow {
    let support = executable_support_row(source, fact_kind);
    let traceability = traceability_for(surface, fact_kind);
    let row_digest = compose_certification_row_digest(
        "projection_consumption_support_matrix_row_v1",
        &[
            ("surface", surface.as_str()),
            ("family", source.family().as_str()),
            ("fact", fact_kind.as_str()),
            ("posture", support.posture().as_str()),
            ("support", support.support_digest()),
            ("rule", traceability.0),
            ("hostile", traceability.1),
            ("lane", traceability.2),
            ("proof", traceability.3.as_str()),
        ],
    );
    ProjectionConsumptionSupportMatrixRow {
        certified_surface: surface,
        source_family: source.family(),
        fact_kind,
        posture: support.posture().clone(),
        support_digest: support.support_digest().to_string(),
        admission_rule: traceability.0,
        hostile_neighbor: traceability.1,
        certification_lane: traceability.2,
        structural_proof: traceability.3,
        row_digest,
    }
}

fn executable_support_row(
    source: &ProjectionConsumptionSource,
    fact_kind: ProjectionFactKind,
) -> ProjectionConsumptionSupportRow {
    discover_projection_consumption_support(source)
        .rows()
        .iter()
        .find(|row| row.fact_kind() == fact_kind)
        .cloned()
        .expect("support report should contain one row for every fact kind")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn family_inventory_covers_every_certified_surface() {
        let inventory = projection_consumption_family_inventory();
        assert_eq!(
            inventory.rows().len(),
            ProjectionConsumptionCertifiedSourceSurface::all().len()
        );
        assert!(inventory
            .rows()
            .iter()
            .all(|row| !row.representative_digest().is_empty()));
        assert!(!inventory.inventory_digest().is_empty());
    }

    #[test]
    fn family_inventory_includes_retained_and_live_surfaces() {
        let inventory = projection_consumption_family_inventory();
        let families = inventory
            .rows()
            .iter()
            .map(ProjectionConsumptionFamilyInventoryRow::source_family)
            .collect::<Vec<_>>();
        assert!(families.contains(&ProjectionSourceFamily::RetainedDerivedArtifactBinding));
        assert!(families.contains(&ProjectionSourceFamily::LiveArtifactBinding));
    }

    #[test]
    fn support_matrix_rows_are_behavior_derived_and_traceable() {
        let matrix = projection_consumption_support_matrix();
        assert_eq!(
            matrix.rows().len(),
            ProjectionConsumptionCertifiedSourceSurface::all().len()
                * ProjectionFactKind::all().len()
        );
        assert!(matrix
            .rows()
            .iter()
            .all(|row| !row.support_digest().is_empty()));
        assert!(matrix.rows().iter().all(|row| {
            !row.admission_rule().is_empty()
                && !row.hostile_neighbor().is_empty()
                && !row.certification_lane().is_empty()
                && !row.row_digest().is_empty()
        }));
        assert!(!matrix.matrix_digest().is_empty());
        assert!(!matrix.support_traceability_digest().is_empty());
    }

    #[test]
    fn retained_and_live_certified_surfaces_capture_admitted_subsets() {
        let matrix = projection_consumption_support_matrix();
        let retained_rows = matrix
            .rows()
            .iter()
            .filter(|row| {
                row.certified_surface()
                    == ProjectionConsumptionCertifiedSourceSurface::RetainedDerivedArtifactBinding
            })
            .collect::<Vec<_>>();
        let live_rows = matrix
            .rows()
            .iter()
            .filter(|row| {
                row.certified_surface()
                    == ProjectionConsumptionCertifiedSourceSurface::LiveArtifactBinding
            })
            .collect::<Vec<_>>();

        assert_eq!(retained_rows.len(), ProjectionFactKind::all().len());
        assert_eq!(live_rows.len(), ProjectionFactKind::all().len());
        assert!(retained_rows.iter().any(|row| {
            row.fact_kind() == ProjectionFactKind::SourceReference
                && matches!(row.posture(), ProjectionConsumptionSupportPosture::Admitted)
        }));
        assert!(retained_rows.iter().any(|row| {
            row.fact_kind() == ProjectionFactKind::EntityIdentity
                && matches!(
                    row.posture(),
                    ProjectionConsumptionSupportPosture::SourceMismatch
                )
        }));
        assert!(live_rows.iter().any(|row| {
            row.fact_kind() == ProjectionFactKind::EntityIdentity
                && matches!(row.posture(), ProjectionConsumptionSupportPosture::Admitted)
        }));
        assert!(live_rows.iter().any(|row| {
            row.fact_kind() == ProjectionFactKind::Membership
                && matches!(
                    row.posture(),
                    ProjectionConsumptionSupportPosture::SourceMismatch
                )
        }));
    }
}
