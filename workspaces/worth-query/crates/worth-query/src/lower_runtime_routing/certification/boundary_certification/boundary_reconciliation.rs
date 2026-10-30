use crate::identity::hash_parts;
use crate::lower_runtime_routing::{
    worth_query_lower_runtime_direct_import_audit, WorthQueryLowerRuntimeDirectImportPosture,
    WorthQueryLowerRuntimeSeamKey,
};

use super::public_surface::{
    worth_query_lower_runtime_public_surface_inventory, WorthQueryLowerRuntimePublicSurfaceKind,
};

const REMAINING_PHASE_SIX_BOUNDARY_SEAMS: &[WorthQueryLowerRuntimeSeamKey] = &[
    WorthQueryLowerRuntimeSeamKey::RuntimeBackendBoundaryModules,
    WorthQueryLowerRuntimeSeamKey::ExecuteRuntimeCurrentReadGraph,
    WorthQueryLowerRuntimeSeamKey::ExecuteRuntimeBasisContextReadGraph,
    WorthQueryLowerRuntimeSeamKey::SubscriptionContinuity,
    WorthQueryLowerRuntimeSeamKey::BasisReadmissionFromTruthViewEvidence,
    WorthQueryLowerRuntimeSeamKey::BasisReadmissionFromSubscriptionEvidence,
    WorthQueryLowerRuntimeSeamKey::HistoricalBridgeLowering,
    WorthQueryLowerRuntimeSeamKey::EffectBackedRelationalMutation,
    WorthQueryLowerRuntimeSeamKey::EffectBackedRelationalMerge,
    WorthQueryLowerRuntimeSeamKey::EffectBackedBridgeWriteback,
    WorthQueryLowerRuntimeSeamKey::RuntimeIntentAuthorityAdapter,
    WorthQueryLowerRuntimeSeamKey::IntentRuntimeExecution,
    WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromQueryReceipts,
    WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromRelationalArtifacts,
    WorthQueryLowerRuntimeSeamKey::ProjectionSourceIntakeFromBridgeArtifacts,
    WorthQueryLowerRuntimeSeamKey::CausalBridgeMaterialization,
    WorthQueryLowerRuntimeSeamKey::FrontierEvidenceIntake,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeBoundaryReconciliationRow {
    seam_key: WorthQueryLowerRuntimeSeamKey,
    surface_label: &'static str,
    implementation_path: &'static str,
    surface_kind: WorthQueryLowerRuntimePublicSurfaceKind,
    direct_import_posture: Option<WorthQueryLowerRuntimeDirectImportPosture>,
}

impl WorthQueryLowerRuntimeBoundaryReconciliationRow {
    fn new(
        seam_key: WorthQueryLowerRuntimeSeamKey,
        surface_label: &'static str,
        implementation_path: &'static str,
        surface_kind: WorthQueryLowerRuntimePublicSurfaceKind,
        direct_import_posture: Option<WorthQueryLowerRuntimeDirectImportPosture>,
    ) -> Self {
        Self {
            seam_key,
            surface_label,
            implementation_path,
            surface_kind,
            direct_import_posture,
        }
    }

    pub fn seam_key(&self) -> WorthQueryLowerRuntimeSeamKey {
        self.seam_key
    }

    pub fn surface_label(&self) -> &'static str {
        self.surface_label
    }

    pub fn implementation_path(&self) -> &'static str {
        self.implementation_path
    }

    pub fn surface_kind(&self) -> WorthQueryLowerRuntimePublicSurfaceKind {
        self.surface_kind
    }

    pub fn direct_import_posture(&self) -> Option<WorthQueryLowerRuntimeDirectImportPosture> {
        self.direct_import_posture
    }

    fn row_digest(&self) -> String {
        hash_parts(&[
            self.seam_key.as_str().to_string(),
            self.surface_label.to_string(),
            self.implementation_path.to_string(),
            self.surface_kind.as_str().to_string(),
            self.direct_import_posture
                .map(|posture| posture.as_str().to_string())
                .unwrap_or_else(|| "none".to_string()),
        ])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeBoundaryReconciliationReport {
    rows: Vec<WorthQueryLowerRuntimeBoundaryReconciliationRow>,
    report_digest: String,
}

impl WorthQueryLowerRuntimeBoundaryReconciliationReport {
    fn new(rows: Vec<WorthQueryLowerRuntimeBoundaryReconciliationRow>) -> Self {
        let report_digest = hash_parts(
            &rows
                .iter()
                .map(WorthQueryLowerRuntimeBoundaryReconciliationRow::row_digest)
                .collect::<Vec<_>>(),
        );
        Self {
            rows,
            report_digest,
        }
    }

    pub fn rows(&self) -> &[WorthQueryLowerRuntimeBoundaryReconciliationRow] {
        &self.rows
    }

    pub fn report_digest(&self) -> &str {
        &self.report_digest
    }
}

pub fn worth_query_lower_runtime_boundary_reconciliation_report(
) -> WorthQueryLowerRuntimeBoundaryReconciliationReport {
    let audit = worth_query_lower_runtime_direct_import_audit();
    let rows = worth_query_lower_runtime_public_surface_inventory()
        .rows()
        .iter()
        .map(|row| {
            let direct_import_posture = audit
                .rows()
                .iter()
                .find(|audit_row| audit_row.module_path() == row.implementation_path())
                .map(|audit_row| audit_row.posture());
            WorthQueryLowerRuntimeBoundaryReconciliationRow::new(
                row.seam_key(),
                row.surface_label(),
                row.implementation_path(),
                row.surface_kind(),
                direct_import_posture,
            )
        })
        .collect::<Vec<_>>();

    for seam_key in REMAINING_PHASE_SIX_BOUNDARY_SEAMS {
        assert!(
            rows.iter().any(|row| row.seam_key == *seam_key),
            "remaining phase six seam {} must stay boundary-reconciled",
            seam_key.as_str()
        );
    }

    WorthQueryLowerRuntimeBoundaryReconciliationReport::new(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconciliation_report_covers_remaining_boundary_seams() {
        let report = worth_query_lower_runtime_boundary_reconciliation_report();

        for seam_key in REMAINING_PHASE_SIX_BOUNDARY_SEAMS {
            assert!(report.rows().iter().any(|row| row.seam_key() == *seam_key));
        }
    }

    #[test]
    fn allowed_boundary_adapters_stay_audit_backed_inside_reconciliation_report() {
        let report = worth_query_lower_runtime_boundary_reconciliation_report();

        for row in report.rows() {
            match row.surface_kind() {
                WorthQueryLowerRuntimePublicSurfaceKind::AllowedBoundaryAdapter => {
                    assert_eq!(
                        row.direct_import_posture(),
                        Some(WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter)
                    );
                }
                WorthQueryLowerRuntimePublicSurfaceKind::RuntimeBackendBoundary => {
                    assert_eq!(
                        row.direct_import_posture(),
                        Some(WorthQueryLowerRuntimeDirectImportPosture::RuntimeBackendBoundary)
                    );
                }
                _ => {}
            }
        }
    }
}
