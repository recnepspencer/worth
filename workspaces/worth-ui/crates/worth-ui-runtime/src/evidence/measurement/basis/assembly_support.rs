use crate::declaration::{UiDeclaredMeasurementBasisSource, UiDeclaredMeasurementOwnershipPosture};
use worth_ui_host_contract::WorthUiHostCapabilityReport;
use worth_ui_inspection::UiEvidenceAuthorityGeneration;

use super::HostResultSlots;
use super::{denial::UiMeasurementBasisDenial, UiMeasurementEvidenceSlot};
use crate::evidence::measurement::{
    MeasurementEvidenceInput, UiChildIntrinsicMeasurementEvidence,
    UiMeasurementDependencyLineageEntry, UiMeasurementDependencyLineageKind,
    UiMeasurementEvidenceCategory, UiMeasurementGenerationCompatibility, UiMeasurementResult,
    UiSettledQueryFactReceipt,
};

pub(super) fn assign_slot<'a, T>(
    slot: &mut Option<&'a T>,
    value: &'a T,
    conflicting_slot: &mut Option<UiMeasurementEvidenceSlot>,
    evidence_slot: UiMeasurementEvidenceSlot,
) {
    if slot.is_none() {
        *slot = Some(value);
    } else if conflicting_slot.is_none() {
        *conflicting_slot = Some(evidence_slot);
    }
}

pub(super) fn basis_source_denial(
    basis_source: Option<UiDeclaredMeasurementBasisSource>,
    host_results: &HostResultSlots<'_>,
) -> Option<UiMeasurementBasisDenial> {
    match basis_source {
        Some(UiDeclaredMeasurementBasisSource::ViewportExtent)
            if host_results.viewport_extent.is_none() =>
        {
            Some(UiMeasurementBasisDenial::MissingBasisSourceEvidence {
                basis_source: UiDeclaredMeasurementBasisSource::ViewportExtent,
                slot: UiMeasurementEvidenceSlot::ViewportExtent,
            })
        }
        Some(UiDeclaredMeasurementBasisSource::ScrollViewport)
            if host_results.viewport_extent.is_none() =>
        {
            Some(UiMeasurementBasisDenial::MissingBasisSourceEvidence {
                basis_source: UiDeclaredMeasurementBasisSource::ScrollViewport,
                slot: UiMeasurementEvidenceSlot::ViewportExtent,
            })
        }
        Some(UiDeclaredMeasurementBasisSource::PortalAnchor)
            if host_results.portal_anchor_rect.is_none() =>
        {
            Some(UiMeasurementBasisDenial::MissingBasisSourceEvidence {
                basis_source: UiDeclaredMeasurementBasisSource::PortalAnchor,
                slot: UiMeasurementEvidenceSlot::PortalAnchorRect,
            })
        }
        _ => None,
    }
}

pub(super) fn ownership_posture_denial(
    ownership_posture: Option<UiDeclaredMeasurementOwnershipPosture>,
    host_results: &HostResultSlots<'_>,
) -> Option<UiMeasurementBasisDenial> {
    match ownership_posture {
        Some(UiDeclaredMeasurementOwnershipPosture::ScrollContainerBasis)
            if host_results.scroll_container_viewport.is_none() =>
        {
            Some(UiMeasurementBasisDenial::MissingOwnershipEvidence {
                ownership_posture: UiDeclaredMeasurementOwnershipPosture::ScrollContainerBasis,
                slot: UiMeasurementEvidenceSlot::ScrollContainerViewport,
            })
        }
        Some(UiDeclaredMeasurementOwnershipPosture::PortalAnchorBasisRequired)
            if host_results.portal_anchor_rect.is_none() =>
        {
            Some(UiMeasurementBasisDenial::MissingOwnershipEvidence {
                ownership_posture: UiDeclaredMeasurementOwnershipPosture::PortalAnchorBasisRequired,
                slot: UiMeasurementEvidenceSlot::PortalAnchorRect,
            })
        }
        _ => None,
    }
}

pub(super) fn host_result_compatibility(
    result: Option<&UiMeasurementResult>,
    report: &WorthUiHostCapabilityReport,
    declaration_support_authority_generation: UiEvidenceAuthorityGeneration,
) -> Option<UiMeasurementGenerationCompatibility> {
    let result = result?;
    if result.evidence_generation() != declaration_support_authority_generation {
        return Some(UiMeasurementGenerationCompatibility::StaleHostEvidence {
            expected: declaration_support_authority_generation,
            observed: result.evidence_generation(),
        });
    }
    if result
        .assumption_profile()
        .capability_observation_generation()
        != report.observation_generation()
    {
        return Some(UiMeasurementGenerationCompatibility::StaleHostCapability {
            expected: report.observation_generation(),
            observed: result
                .assumption_profile()
                .capability_observation_generation(),
        });
    }
    if result.assumption_profile().capability_profile_digest() != report.profile_identity_digest() {
        return Some(
            UiMeasurementGenerationCompatibility::IncompatibleHostProfile {
                expected_profile_digest: report.profile_identity_digest(),
                observed_profile_digest: result.assumption_profile().capability_profile_digest(),
            },
        );
    }
    None
}

pub(super) fn settled_query_receipt_compatibility(
    receipt: &UiSettledQueryFactReceipt,
    world_profile: &crate::graph::UiGraphWorldProfile,
    declaration_support_authority_generation: UiEvidenceAuthorityGeneration,
) -> Option<UiMeasurementGenerationCompatibility> {
    if receipt.declaration_support_authority_generation()
        != declaration_support_authority_generation
    {
        return Some(
            UiMeasurementGenerationCompatibility::StaleQueryFactReceipt {
                expected: declaration_support_authority_generation,
                observed: receipt.declaration_support_authority_generation(),
            },
        );
    }
    let crate::graph::UiGraphWorldProfile::SettledQueryBinding {
        view_binding_id,
        binding_reference,
    } = world_profile
    else {
        return Some(UiMeasurementGenerationCompatibility::IncompatibleWorld {
            reason: crate::evidence::UiQueryWorldCompatibilityFailure::QueryAuthorityUnavailable,
        });
    };
    if view_binding_id != receipt.view_binding_id()
        || binding_reference != receipt.binding_reference()
    {
        return Some(UiMeasurementGenerationCompatibility::IncompatibleWorld {
            reason: crate::evidence::UiQueryWorldCompatibilityFailure::InstalledAuthorityMismatch,
        });
    }
    None
}

pub(super) fn push_host_lineage(
    entries: &mut Vec<UiMeasurementDependencyLineageEntry>,
    result: Option<&UiMeasurementResult>,
    kind: UiMeasurementDependencyLineageKind,
) {
    if let Some(result) = result {
        entries.push(UiMeasurementDependencyLineageEntry::new(
            kind,
            MeasurementEvidenceInput::host_measurement_result(result).identity_digest(),
            result.evidence_generation().as_u64(),
        ));
    }
}

pub(super) fn child_intrinsic_query_compatibility(
    evidence: &UiChildIntrinsicMeasurementEvidence,
    world_profile: &crate::graph::UiGraphWorldProfile,
    declaration_support_authority_generation: UiEvidenceAuthorityGeneration,
) -> Option<UiMeasurementGenerationCompatibility> {
    evidence.query_projection_fact().and_then(|receipt| {
        settled_query_receipt_compatibility(
            receipt,
            world_profile,
            declaration_support_authority_generation,
        )
    })
}

pub(super) fn child_intrinsic_host_compatibility(
    evidence: &UiChildIntrinsicMeasurementEvidence,
    report: &WorthUiHostCapabilityReport,
    declaration_support_authority_generation: UiEvidenceAuthorityGeneration,
) -> Option<UiMeasurementGenerationCompatibility> {
    host_result_compatibility(
        evidence.host_measurement_result(),
        report,
        declaration_support_authority_generation,
    )
}

pub(super) fn push_child_intrinsic_lineage(
    entries: &mut Vec<UiMeasurementDependencyLineageEntry>,
    evidence: &UiChildIntrinsicMeasurementEvidence,
) {
    if let Some(receipt) = evidence.query_projection_fact() {
        entries.push(UiMeasurementDependencyLineageEntry::new(
            UiMeasurementDependencyLineageKind::QueryScrollContentExtent,
            evidence.identity_digest(),
            receipt.declaration_support_authority_generation().as_u64(),
        ));
    }
    if let Some(result) = evidence.host_measurement_result() {
        let kind = match result.evidence_category() {
            UiMeasurementEvidenceCategory::TextIntrinsicSize => {
                UiMeasurementDependencyLineageKind::HostTextIntrinsicSize
            }
            UiMeasurementEvidenceCategory::NativeControlIntrinsicSize => {
                UiMeasurementDependencyLineageKind::HostNativeControlIntrinsicSize
            }
            UiMeasurementEvidenceCategory::TextBaselineMetrics
            | UiMeasurementEvidenceCategory::FontMetrics
            | UiMeasurementEvidenceCategory::ViewportExtent
            | UiMeasurementEvidenceCategory::DpiScaleFactor
            | UiMeasurementEvidenceCategory::PortalAnchorRect
            | UiMeasurementEvidenceCategory::ScrollContainerViewport => return,
        };
        entries.push(UiMeasurementDependencyLineageEntry::new(
            kind,
            evidence.identity_digest(),
            result.evidence_generation().as_u64(),
        ));
    }
}
