use crate::identity::hash_parts;
use crate::lower_runtime_routing::WorthQueryLowerRuntimeSeamKey;

mod rows;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimePublicSurfaceKind {
    PublicFacade,
    InternalRouteComposer,
    RuntimeBackendBoundary,
    AllowedBoundaryAdapter,
}

impl WorthQueryLowerRuntimePublicSurfaceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PublicFacade => "public-facade",
            Self::InternalRouteComposer => "internal-route-composer",
            Self::RuntimeBackendBoundary => "runtime-backend-boundary",
            Self::AllowedBoundaryAdapter => "allowed-boundary-adapter",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimePublicSurfaceRow {
    seam_key: WorthQueryLowerRuntimeSeamKey,
    surface_label: &'static str,
    implementation_path: &'static str,
    surface_kind: WorthQueryLowerRuntimePublicSurfaceKind,
    delegated_lane: &'static str,
}

impl WorthQueryLowerRuntimePublicSurfaceRow {
    pub(crate) const fn new(
        seam_key: WorthQueryLowerRuntimeSeamKey,
        surface_label: &'static str,
        implementation_path: &'static str,
        surface_kind: WorthQueryLowerRuntimePublicSurfaceKind,
        delegated_lane: &'static str,
    ) -> Self {
        Self {
            seam_key,
            surface_label,
            implementation_path,
            surface_kind,
            delegated_lane,
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

    pub fn delegated_lane(&self) -> &'static str {
        self.delegated_lane
    }

    pub fn row_digest(&self) -> String {
        hash_parts(&[
            self.seam_key.as_str().to_string(),
            self.surface_label.to_string(),
            self.implementation_path.to_string(),
            self.surface_kind.as_str().to_string(),
            self.delegated_lane.to_string(),
        ])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimePublicSurfaceInventory {
    rows: &'static [WorthQueryLowerRuntimePublicSurfaceRow],
}

impl WorthQueryLowerRuntimePublicSurfaceInventory {
    pub(crate) const fn new(rows: &'static [WorthQueryLowerRuntimePublicSurfaceRow]) -> Self {
        Self { rows }
    }

    pub fn rows(&self) -> &'static [WorthQueryLowerRuntimePublicSurfaceRow] {
        self.rows
    }

    pub fn public_surface_digest(&self) -> String {
        hash_parts(
            &self
                .rows
                .iter()
                .map(WorthQueryLowerRuntimePublicSurfaceRow::row_digest)
                .collect::<Vec<_>>(),
        )
    }
}

pub fn worth_query_lower_runtime_public_surface_inventory(
) -> WorthQueryLowerRuntimePublicSurfaceInventory {
    WorthQueryLowerRuntimePublicSurfaceInventory::new(rows::PUBLIC_SURFACE_ROWS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower_runtime_routing::{
        worth_query_lower_runtime_crossing_inventory,
        worth_query_lower_runtime_direct_import_audit, worth_query_lower_runtime_support_matrix,
        WorthQueryLowerRuntimeDirectImportPosture, WorthQueryLowerRuntimeSupportPosture,
    };

    #[test]
    fn public_surface_rows_reference_known_routed_or_audited_seams() {
        let crossing_inventory = worth_query_lower_runtime_crossing_inventory();
        let direct_import_audit = worth_query_lower_runtime_direct_import_audit();
        let support = worth_query_lower_runtime_support_matrix();

        for row in worth_query_lower_runtime_public_surface_inventory().rows() {
            let crossing_backed = crossing_inventory
                .rows()
                .iter()
                .any(|crossing| crossing.seam_key() == row.seam_key());
            let audit_backed = direct_import_audit
                .rows()
                .iter()
                .any(|audit_row| audit_row.seam_key() == row.seam_key());
            assert!(
                crossing_backed || audit_backed,
                "public surface row {} must map to the crossing inventory or direct-import audit",
                row.surface_label()
            );
            if crossing_backed {
                assert_eq!(
                    support
                        .support_for(row.seam_key())
                        .expect("crossing-backed public surface rows must resolve through support")
                        .posture(),
                    WorthQueryLowerRuntimeSupportPosture::Admitted
                );
            }
        }
    }

    #[test]
    fn public_surface_inventory_digest_is_row_order_stable() {
        let inventory = worth_query_lower_runtime_public_surface_inventory();
        let expected = hash_parts(
            &inventory
                .rows()
                .iter()
                .map(WorthQueryLowerRuntimePublicSurfaceRow::row_digest)
                .collect::<Vec<_>>(),
        );
        assert_eq!(inventory.public_surface_digest(), expected);
    }

    #[test]
    fn public_surface_inventory_covers_mutation_facades_and_internal_routing_composers() {
        let inventory = worth_query_lower_runtime_public_surface_inventory();

        for label in [
            "execute_runtime_current_read_graph(...)",
            "execute_runtime_basis_context_read_graph(...)",
            "WorthQueryWorkspace::live_view(...)",
            "WorthQueryWorkspace::live_view_request(...)",
            "WorthQueryWorkspace::write(...)",
            "WorthQueryWorkspace::assert_existing(...)",
            "WorthQueryWorkspace::verify_existing(...)",
            "WorthQueryWorkspace::update_existing_verified(...)",
            "WorthQueryWorkspace::batch(...)",
            "WorthQueryWorkspace::delete_existing_verified(...)",
            "WorthQueryRuntime::write(...)",
            "WorthQueryRuntime::write_batch(...)",
            "WorthQueryRuntime::execute_authoritative_write_command_direct(...)",
            "WorthQueryRuntime::execute_authoritative_write_batch_direct(...)",
            "projection source intake from Query receipts",
            "projection source intake from relational artifacts",
            "projection source intake from bridge artifacts",
            "causal bridge materialization adapter",
            "frontier signal evidence adapter",
        ] {
            assert!(inventory
                .rows()
                .iter()
                .any(|row| row.surface_label() == label));
        }
    }

    #[test]
    fn public_surface_inventory_reconciles_remaining_boundary_seams() {
        let inventory = worth_query_lower_runtime_public_surface_inventory();

        for seam_key in [
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
        ] {
            assert!(
                inventory
                    .rows()
                    .iter()
                    .any(|row| row.seam_key() == seam_key),
                "missing boundary reconciliation row for {}",
                seam_key.as_str()
            );
        }
    }

    #[test]
    fn allowed_boundary_adapter_rows_stay_synchronized_with_direct_import_audit() {
        let audit = worth_query_lower_runtime_direct_import_audit();

        for row in worth_query_lower_runtime_public_surface_inventory()
            .rows()
            .iter()
            .filter(|row| {
                row.surface_kind()
                    == WorthQueryLowerRuntimePublicSurfaceKind::AllowedBoundaryAdapter
            })
        {
            assert!(
                audit.rows().iter().any(|audit_row| {
                    audit_row.module_path() == row.implementation_path()
                        && audit_row.posture()
                            == WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter
                }),
                "allowed boundary adapter row {} must stay mirrored in direct-import audit",
                row.surface_label()
            );
        }
    }

    #[test]
    fn direct_import_audit_boundary_rows_stay_synchronized_with_public_surface_inventory() {
        let inventory = worth_query_lower_runtime_public_surface_inventory();

        for audit_row in worth_query_lower_runtime_direct_import_audit().rows() {
            let expected_kind = match audit_row.posture() {
                WorthQueryLowerRuntimeDirectImportPosture::RuntimeBackendBoundary => {
                    Some(WorthQueryLowerRuntimePublicSurfaceKind::RuntimeBackendBoundary)
                }
                WorthQueryLowerRuntimeDirectImportPosture::AllowedAdapter => {
                    Some(WorthQueryLowerRuntimePublicSurfaceKind::AllowedBoundaryAdapter)
                }
                WorthQueryLowerRuntimeDirectImportPosture::DownstreamRuntimeBoundarySubtree => None,
                WorthQueryLowerRuntimeDirectImportPosture::TransitionOnlyElimination => None,
            };
            let Some(expected_kind) = expected_kind else {
                continue;
            };
            let expected_path = audit_row.module_path().trim_end_matches("/*");
            assert!(
                inventory.rows().iter().any(|row| {
                    row.implementation_path().trim_end_matches("/*") == expected_path
                        && row.surface_kind() == expected_kind
                }),
                "direct-import audit row {} must stay mirrored in the certified public surface inventory",
                audit_row.module_path()
            );
        }
    }
}
