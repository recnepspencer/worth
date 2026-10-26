use std::collections::BTreeMap;

use super::bridge_backed_verification_profile::{
    default_bridge_backed_verification_support_rows,
    WorthQueryBridgeBackedVerificationSupportProfileRow,
};
use super::{
    default_graph_composition_capability_support_rows, WorthQueryBranchBasisAdmission,
    WorthQueryGraphCompositionCapabilitySupportRow, WorthQueryPreviewBasisAdmission,
    WorthQueryRuntimeBackendPosture, WorthQueryRuntimeBatchAuthority,
    WorthQueryRuntimeEvidenceAuthority, WorthQueryRuntimeFacadeFamily,
    WorthQueryRuntimeFamilySupport, WorthQueryRuntimeFamilySupportStatus,
    WorthQueryRuntimeInspectionEvidence, WorthQueryRuntimeSupportDenial,
};
use crate::runtime::WorthQueryGraphIndexSupportRow;
use crate::runtime::{WorthQueryAuthorityLane, WorthQueryEffectPolicy};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeSupportProfile {
    posture: WorthQueryRuntimeBackendPosture,
    pub(super) batch_authority: WorthQueryRuntimeBatchAuthority,
    rows: BTreeMap<WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport>,
    bridge_backed_verification_support_rows:
        Vec<WorthQueryBridgeBackedVerificationSupportProfileRow>,
    graph_composition_capability_support_rows: Vec<WorthQueryGraphCompositionCapabilitySupportRow>,
    pub(super) graph_index_support_rows: Vec<WorthQueryGraphIndexSupportRow>,
}

impl WorthQueryRuntimeSupportProfile {
    pub fn new(rows: impl IntoIterator<Item = WorthQueryRuntimeFamilySupport>) -> Self {
        Self {
            posture: WorthQueryRuntimeBackendPosture::Primary,
            batch_authority: WorthQueryRuntimeBatchAuthority::BackendAtomicFull,
            rows: rows.into_iter().map(|row| (row.family(), row)).collect(),
            bridge_backed_verification_support_rows:
                default_bridge_backed_verification_support_rows(),
            graph_composition_capability_support_rows:
                default_graph_composition_capability_support_rows(),
            graph_index_support_rows: default_graph_index_support_rows(),
        }
    }

    pub fn scaffold_backend_profile() -> Self {
        Self::new([
            WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::Read,
                [WorthQueryAuthorityLane::AuthoritativeTruth],
                [],
                ["backend-live-read"],
            ),
            WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::Live,
                [WorthQueryAuthorityLane::AuthoritativeTruth],
                [],
                ["backend-live-declaration"],
            ),
            WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::Computed,
                [WorthQueryAuthorityLane::DerivedRuntimeState],
                [WorthQueryEffectPolicy::DeriveOnly],
                ["query-local-derived-view-runtime"],
            ),
            WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::SharedRead,
                [WorthQueryAuthorityLane::DerivedRuntimeState],
                [],
                ["published-derived-artifact-consumption"],
            ),
            WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::Submission,
                [
                    WorthQueryAuthorityLane::PendingWriteIntent,
                    WorthQueryAuthorityLane::AuthoritativeTruth,
                ],
                [],
                [
                    "deterministic-submission-seam",
                    "authoritative-mutation-evidence",
                ],
            ),
            WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::Replay,
                [WorthQueryAuthorityLane::AuthoritativeTruth],
                [],
                [
                    "journal-segment-identity",
                    "journal-replay-outcome",
                    "published-derived-artifact-consumption",
                ],
            ),
            WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::Effect,
                [WorthQueryAuthorityLane::EffectDeliveryState],
                [WorthQueryEffectPolicy::AuthoritativeAllowed],
                ["query-local-effect-delivery-runtime"],
            ),
            WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::BranchPreview,
                [
                    WorthQueryAuthorityLane::PreviewTruth,
                    WorthQueryAuthorityLane::BranchLocalTruth,
                ],
                [
                    WorthQueryEffectPolicy::DeriveOnly,
                    WorthQueryEffectPolicy::Muted,
                    WorthQueryEffectPolicy::Redirected,
                    WorthQueryEffectPolicy::SandboxedWriteIntent,
                    WorthQueryEffectPolicy::AuthoritativeAllowed,
                ],
                ["preview-session-admission"],
            ),
            WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::Write,
                [WorthQueryAuthorityLane::AuthoritativeTruth],
                [],
                ["backend-write-authority", "authoritative-mutation-evidence"],
            ),
            WorthQueryRuntimeFamilySupport::unsupported(
                WorthQueryRuntimeFacadeFamily::Intent,
                "intent commit strategies are not admitted by this runtime batch",
            ),
            WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::Inspect,
                [
                    WorthQueryAuthorityLane::AuthoritativeTruth,
                    WorthQueryAuthorityLane::BranchLocalTruth,
                    WorthQueryAuthorityLane::DerivedRuntimeState,
                    WorthQueryAuthorityLane::EffectDeliveryState,
                    WorthQueryAuthorityLane::PendingWriteIntent,
                    WorthQueryAuthorityLane::PreviewTruth,
                ],
                [],
                ["retained-runtime-artifact-inspection"],
            ),
            WorthQueryRuntimeFamilySupport::supported_with_teaching_posture(
                WorthQueryRuntimeFacadeFamily::Temporal,
                super::WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
                [WorthQueryAuthorityLane::TemporalExecutionState],
                [],
                ["runtime-backed-temporal-basis-state-inspection"],
            ),
            WorthQueryRuntimeFamilySupport::supported_with_teaching_posture(
                WorthQueryRuntimeFacadeFamily::AsyncResource,
                super::WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
                [WorthQueryAuthorityLane::AsyncResourceState],
                [],
                ["runtime-backed-async-resource-state-inspection"],
            ),
            WorthQueryRuntimeFamilySupport::supported_with_teaching_posture(
                WorthQueryRuntimeFacadeFamily::MixedCauseDelivery,
                super::WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly,
                [WorthQueryAuthorityLane::BridgeExternalState],
                [],
                ["runtime-backed-mixed-cause-delivery-state-inspection"],
            ),
            WorthQueryRuntimeFamilySupport::deferred(
                WorthQueryRuntimeFacadeFamily::StoreBackedExecution,
                "store-backed execution parity is deferred to Milestone 10",
            ),
            WorthQueryRuntimeFamilySupport::deferred(
                WorthQueryRuntimeFacadeFamily::DurableArtifacts,
                "durable restart and artifact reload are deferred to Milestone 11",
            ),
        ])
        .with_posture(WorthQueryRuntimeBackendPosture::Scaffold)
    }

    pub fn bridge_backed(
        subscription_activation_evidence: impl Into<String>,
        preview_basis_evidence: impl Into<String>,
        inspector_evidence: impl Into<String>,
    ) -> Self {
        let subscription_activation_evidence = subscription_activation_evidence.into();
        let preview_basis_evidence = preview_basis_evidence.into();
        let inspector_evidence = inspector_evidence.into();
        Self::scaffold_backend_profile()
            .with_family_support(WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::Live,
                [WorthQueryAuthorityLane::AuthoritativeTruth],
                [],
                [
                    "backend-live-declaration".to_string(),
                    subscription_activation_evidence,
                ],
            ))
            .with_family_support(WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::BranchPreview,
                [
                    WorthQueryAuthorityLane::PreviewTruth,
                    WorthQueryAuthorityLane::BranchLocalTruth,
                ],
                [
                    WorthQueryEffectPolicy::DeriveOnly,
                    WorthQueryEffectPolicy::Muted,
                    WorthQueryEffectPolicy::Redirected,
                    WorthQueryEffectPolicy::SandboxedWriteIntent,
                    WorthQueryEffectPolicy::AuthoritativeAllowed,
                ],
                [
                    "preview-session-admission".to_string(),
                    preview_basis_evidence,
                ],
            ))
            .with_family_support(WorthQueryRuntimeFamilySupport::supported(
                WorthQueryRuntimeFacadeFamily::Inspect,
                [
                    WorthQueryAuthorityLane::AuthoritativeTruth,
                    WorthQueryAuthorityLane::BranchLocalTruth,
                    WorthQueryAuthorityLane::DerivedRuntimeState,
                    WorthQueryAuthorityLane::EffectDeliveryState,
                    WorthQueryAuthorityLane::PendingWriteIntent,
                    WorthQueryAuthorityLane::PreviewTruth,
                ],
                [],
                [
                    "retained-runtime-artifact-inspection".to_string(),
                    inspector_evidence,
                ],
            ))
            .with_posture(WorthQueryRuntimeBackendPosture::Primary)
    }

    pub fn support_for(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Option<&WorthQueryRuntimeFamilySupport> {
        self.rows.get(&family)
    }

    pub fn posture(&self) -> WorthQueryRuntimeBackendPosture {
        self.posture
    }

    pub fn with_posture(mut self, posture: WorthQueryRuntimeBackendPosture) -> Self {
        self.posture = posture;
        self.batch_authority = match posture {
            WorthQueryRuntimeBackendPosture::Primary => {
                WorthQueryRuntimeBatchAuthority::BackendAtomicFull
            }
            WorthQueryRuntimeBackendPosture::Scaffold => {
                WorthQueryRuntimeBatchAuthority::Unsupported
            }
        };
        self
    }

    pub fn with_family_support(mut self, row: WorthQueryRuntimeFamilySupport) -> Self {
        self.rows.insert(row.family(), row);
        self
    }

    pub(crate) fn bridge_backed_verification_support_rows(
        &self,
    ) -> &[WorthQueryBridgeBackedVerificationSupportProfileRow] {
        &self.bridge_backed_verification_support_rows
    }

    pub(crate) fn graph_composition_capability_support_rows(
        &self,
    ) -> &[WorthQueryGraphCompositionCapabilitySupportRow] {
        &self.graph_composition_capability_support_rows
    }

    pub(crate) fn with_bridge_backed_verification_support_row(
        mut self,
        row: WorthQueryBridgeBackedVerificationSupportProfileRow,
    ) -> Self {
        let rows = &mut self.bridge_backed_verification_support_rows;
        if let Some(index) = rows.iter().position(|candidate| {
            candidate.operation_family() == row.operation_family()
                && candidate.target_binding_family() == row.target_binding_family()
        }) {
            rows[index] = row;
        } else {
            rows.push(row);
        }
        self
    }

    pub fn with_bridge_backed_verification_support(
        self,
        operation_family: impl Into<String>,
        target_binding_family: impl Into<String>,
        scaffold_profile_supported: bool,
        primary_bridge_backed_runtime_supported: bool,
        denial_class_when_primary_unsupported: Option<&str>,
    ) -> Self {
        self.with_bridge_backed_verification_support_row(
            WorthQueryBridgeBackedVerificationSupportProfileRow::new(
                operation_family,
                target_binding_family,
                scaffold_profile_supported,
                primary_bridge_backed_runtime_supported,
                denial_class_when_primary_unsupported,
            ),
        )
    }

    pub fn with_graph_composition_capability_support_row(
        mut self,
        row: WorthQueryGraphCompositionCapabilitySupportRow,
    ) -> Self {
        if let Some(index) = self
            .graph_composition_capability_support_rows
            .iter()
            .position(|candidate| {
                candidate.capability_family() == row.capability_family()
                    && candidate.capability_class() == row.capability_class()
            })
        {
            self.graph_composition_capability_support_rows[index] = row;
        } else {
            self.graph_composition_capability_support_rows.push(row);
        }
        self
    }

    pub fn rows(&self) -> impl Iterator<Item = &WorthQueryRuntimeFamilySupport> {
        self.rows.values()
    }

    pub fn admit(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Result<(), WorthQueryRuntimeSupportDenial> {
        let Some(row) = self.support_for(family) else {
            return Err(WorthQueryRuntimeSupportDenial::unsupported(
                family,
                "backend support profile does not declare this facade family",
            ));
        };

        match row.status() {
            WorthQueryRuntimeFamilySupportStatus::Supported => Ok(()),
            WorthQueryRuntimeFamilySupportStatus::DeferredDebt => {
                Err(WorthQueryRuntimeSupportDenial::new(
                    family,
                    WorthQueryRuntimeFamilySupportStatus::DeferredDebt,
                    Some(row.teaching_posture()),
                    row.denial_reason()
                        .unwrap_or("backend support profile marks this facade family deferred"),
                ))
            }
            WorthQueryRuntimeFamilySupportStatus::Unsupported => {
                Err(WorthQueryRuntimeSupportDenial::new(
                    family,
                    WorthQueryRuntimeFamilySupportStatus::Unsupported,
                    Some(row.teaching_posture()),
                    row.denial_reason()
                        .unwrap_or("backend support profile marks this facade family unsupported"),
                ))
            }
        }
    }

    pub(crate) fn validate_backend_claims(
        &self,
        has_intent_authority: bool,
    ) -> Result<(), WorthQueryRuntimeSupportDenial> {
        if self
            .support_for(WorthQueryRuntimeFacadeFamily::Intent)
            .is_some_and(|row| row.status() == WorthQueryRuntimeFamilySupportStatus::Supported)
            && !has_intent_authority
        {
            return Err(WorthQueryRuntimeSupportDenial::new(
                WorthQueryRuntimeFacadeFamily::Intent,
                WorthQueryRuntimeFamilySupportStatus::Supported,
                self.support_for(WorthQueryRuntimeFacadeFamily::Intent)
                    .map(WorthQueryRuntimeFamilySupport::teaching_posture),
                "intent support requires an executable intent authority adapter",
            ));
        }

        Ok(())
    }
}

pub(super) fn default_graph_index_support_rows() -> Vec<WorthQueryGraphIndexSupportRow> {
    crate::runtime::WorthQueryGraphIndexInventory::current_runtime_support_rows()
}

fn _keep_types_visible(
    _authority: &WorthQueryRuntimeEvidenceAuthority,
    _preview: &WorthQueryPreviewBasisAdmission,
    _branch: &WorthQueryBranchBasisAdmission,
    _inspection: &WorthQueryRuntimeInspectionEvidence,
) {
}
