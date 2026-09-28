use super::assembly_support::{
    assign_slot, basis_source_denial, ownership_posture_denial, push_child_intrinsic_lineage,
    push_host_lineage,
};
use super::{
    denial::UiMeasurementBasisDenial, HostResultSlots, UiMeasurementBasisPosture,
    UiMeasurementEvidenceSlot,
};
use crate::declaration::UiDeclaredMeasurementBasisRequirementSet;
use crate::evidence::measurement::{
    MeasurementEvidenceInput, UiChildIntrinsicMeasurementEvidence, UiMeasurementDependencyLineage,
    UiMeasurementDependencyLineageEntry, UiMeasurementDependencyLineageKind,
    UiMeasurementEvidenceCategory, UiMeasurementGenerationCompatibility, UiSettledQueryFactReceipt,
};
use worth_ui_host_contract::WorthUiHostCapabilityReport;

pub(super) struct SelectedEvidence<'a> {
    pub query_receipt: Option<&'a UiSettledQueryFactReceipt>,
    pub host_capability_report: Option<&'a WorthUiHostCapabilityReport>,
    pub host_results: HostResultSlots<'a>,
    pub child_intrinsic_measurements: Vec<&'a UiChildIntrinsicMeasurementEvidence>,
    pub sibling_resize_support: Option<&'a crate::evidence::UiMeasurementSiblingResizeSupport>,
    conflicting_slot: Option<UiMeasurementEvidenceSlot>,
}

impl<'a> SelectedEvidence<'a> {
    pub(super) fn from_inputs(
        requirements: &UiDeclaredMeasurementBasisRequirementSet,
        evidence_inputs: &'a [MeasurementEvidenceInput],
    ) -> Self {
        let has_intrinsic_host_input = evidence_inputs.iter().any(|input| {
            matches!(
                input
                    .as_host_measurement_result()
                    .map(|result| result.evidence_category()),
                Some(UiMeasurementEvidenceCategory::TextIntrinsicSize)
                    | Some(UiMeasurementEvidenceCategory::NativeControlIntrinsicSize)
            ) || input
                .as_child_intrinsic_measurement()
                .and_then(UiChildIntrinsicMeasurementEvidence::host_measurement_result)
                .is_some()
        });
        let mut selected = Self {
            query_receipt: None,
            host_capability_report: None,
            host_results: HostResultSlots::default(),
            child_intrinsic_measurements: Vec::new(),
            sibling_resize_support: None,
            conflicting_slot: None,
        };
        for input in evidence_inputs {
            if let Some(receipt) = input.as_settled_query_fact() {
                if requirements.requires_query_projection_receipt() {
                    selected.assign_query_receipt(receipt);
                }
                continue;
            }
            if let Some(report) = input.as_host_capability_report() {
                if requirements.requires_host_measurement_evidence() || has_intrinsic_host_input {
                    assign_slot(
                        &mut selected.host_capability_report,
                        report,
                        &mut selected.conflicting_slot,
                        UiMeasurementEvidenceSlot::HostCapabilityReport,
                    );
                }
                continue;
            }
            if let Some(result) = input.as_host_measurement_result() {
                match result.evidence_category() {
                    UiMeasurementEvidenceCategory::TextIntrinsicSize => {
                        assign_slot(
                            &mut selected.host_results.text_intrinsic_size,
                            result,
                            &mut selected.conflicting_slot,
                            UiMeasurementEvidenceSlot::HostTextIntrinsicSize,
                        );
                    }
                    UiMeasurementEvidenceCategory::FontMetrics
                        if requirements.requires_host_font_metrics() =>
                    {
                        assign_slot(
                            &mut selected.host_results.font_metrics,
                            result,
                            &mut selected.conflicting_slot,
                            UiMeasurementEvidenceSlot::HostFontMetrics,
                        );
                    }
                    UiMeasurementEvidenceCategory::NativeControlIntrinsicSize => {
                        assign_slot(
                            &mut selected.host_results.native_control_intrinsic_size,
                            result,
                            &mut selected.conflicting_slot,
                            UiMeasurementEvidenceSlot::HostNativeControlIntrinsicSize,
                        );
                    }
                    UiMeasurementEvidenceCategory::ViewportExtent
                        if requirements.requires_viewport_extent() =>
                    {
                        assign_slot(
                            &mut selected.host_results.viewport_extent,
                            result,
                            &mut selected.conflicting_slot,
                            UiMeasurementEvidenceSlot::ViewportExtent,
                        );
                    }
                    UiMeasurementEvidenceCategory::PortalAnchorRect
                        if requirements.requires_portal_anchor_metrics() =>
                    {
                        assign_slot(
                            &mut selected.host_results.portal_anchor_rect,
                            result,
                            &mut selected.conflicting_slot,
                            UiMeasurementEvidenceSlot::PortalAnchorRect,
                        );
                    }
                    UiMeasurementEvidenceCategory::ScrollContainerViewport
                        if requirements.requires_scroll_container_viewport() =>
                    {
                        assign_slot(
                            &mut selected.host_results.scroll_container_viewport,
                            result,
                            &mut selected.conflicting_slot,
                            UiMeasurementEvidenceSlot::ScrollContainerViewport,
                        );
                    }
                    UiMeasurementEvidenceCategory::TextBaselineMetrics
                    | UiMeasurementEvidenceCategory::FontMetrics
                    | UiMeasurementEvidenceCategory::ViewportExtent
                    | UiMeasurementEvidenceCategory::DpiScaleFactor
                    | UiMeasurementEvidenceCategory::PortalAnchorRect
                    | UiMeasurementEvidenceCategory::ScrollContainerViewport => {}
                }
                continue;
            }

            if let Some(evidence) = input.as_child_intrinsic_measurement() {
                selected.child_intrinsic_measurements.push(evidence);
                continue;
            }
            if let Some(support) = input.as_sibling_resize_support() {
                selected.sibling_resize_support.get_or_insert(support);
            }
        }
        selected
    }

    fn assign_query_receipt(&mut self, receipt: &'a UiSettledQueryFactReceipt) {
        if self.query_receipt.is_none() {
            self.query_receipt = Some(receipt);
        } else if self.conflicting_slot.is_none() {
            self.conflicting_slot = Some(UiMeasurementEvidenceSlot::QueryProjectionFactReceipt);
        }
    }

    pub(super) fn admitted_inputs(&self) -> Box<[MeasurementEvidenceInput]> {
        let mut inputs = Vec::new();
        if let Some(receipt) = self.query_receipt {
            inputs.push(MeasurementEvidenceInput::settled_query_fact(receipt));
        }
        if let Some(report) = self.host_capability_report {
            inputs.push(MeasurementEvidenceInput::host_capability_report(report));
        }
        for result in self.host_results.relevant_results().into_iter().flatten() {
            inputs.push(MeasurementEvidenceInput::host_measurement_result(result));
        }
        for evidence in &self.child_intrinsic_measurements {
            inputs.push(MeasurementEvidenceInput::ChildIntrinsicMeasurement(
                (*evidence).clone(),
            ));
        }
        if let Some(support) = self.sibling_resize_support {
            inputs.push(MeasurementEvidenceInput::SiblingResizeSupport(
                (*support).clone(),
            ));
        }
        inputs.into_boxed_slice()
    }

    pub(super) fn dependency_lineage(&self) -> UiMeasurementDependencyLineage {
        let mut entries = Vec::new();
        if let Some(receipt) = self.query_receipt {
            entries.push(UiMeasurementDependencyLineageEntry::new(
                UiMeasurementDependencyLineageKind::QueryScrollContentExtent,
                receipt.observation_identity_digest(),
                receipt.declaration_support_authority_generation().as_u64(),
            ));
        }
        push_host_lineage(
            &mut entries,
            self.host_results.text_intrinsic_size,
            UiMeasurementDependencyLineageKind::HostTextIntrinsicSize,
        );
        push_host_lineage(
            &mut entries,
            self.host_results.font_metrics,
            UiMeasurementDependencyLineageKind::HostFontMetrics,
        );
        push_host_lineage(
            &mut entries,
            self.host_results.native_control_intrinsic_size,
            UiMeasurementDependencyLineageKind::HostNativeControlIntrinsicSize,
        );
        push_host_lineage(
            &mut entries,
            self.host_results.viewport_extent,
            UiMeasurementDependencyLineageKind::HostViewportExtent,
        );
        push_host_lineage(
            &mut entries,
            self.host_results.portal_anchor_rect,
            UiMeasurementDependencyLineageKind::HostPortalAnchorRect,
        );
        push_host_lineage(
            &mut entries,
            self.host_results.scroll_container_viewport,
            UiMeasurementDependencyLineageKind::HostScrollContainerViewport,
        );
        for evidence in &self.child_intrinsic_measurements {
            push_child_intrinsic_lineage(&mut entries, evidence);
        }
        UiMeasurementDependencyLineage::new(entries)
    }

    pub(super) fn basis_posture(&self) -> UiMeasurementBasisPosture {
        let has_query = self.query_receipt.is_some()
            || self
                .child_intrinsic_measurements
                .iter()
                .any(|evidence| evidence.query_projection_fact().is_some());
        let has_host = self.host_capability_report.is_some()
            || self
                .child_intrinsic_measurements
                .iter()
                .any(|evidence| evidence.host_measurement_result().is_some());
        match (has_query, has_host) {
            (true, true) => UiMeasurementBasisPosture::QueryAndHost,
            (true, false) => UiMeasurementBasisPosture::QueryOnly,
            (false, true) | (false, false) => UiMeasurementBasisPosture::HostOnly,
        }
    }

    pub(super) fn denial_posture(
        &self,
        requirements: &UiDeclaredMeasurementBasisRequirementSet,
        generation_compatibility: &UiMeasurementGenerationCompatibility,
    ) -> Option<UiMeasurementBasisDenial> {
        let has_child_query_measurement = self
            .child_intrinsic_measurements
            .iter()
            .any(|evidence| evidence.query_projection_fact().is_some());
        if !generation_compatibility.is_compatible() {
            return Some(UiMeasurementBasisDenial::GenerationIncompatible {
                compatibility: generation_compatibility.clone(),
            });
        }
        if let Some(slot) = self.conflicting_slot {
            return Some(UiMeasurementBasisDenial::ConflictingEvidenceInputs { slot });
        }
        if requirements.requires_query_projection_receipt()
            && self.query_receipt.is_none()
            && !has_child_query_measurement
        {
            return Some(UiMeasurementBasisDenial::MissingEvidence {
                slot: UiMeasurementEvidenceSlot::QueryProjectionFactReceipt,
            });
        }
        if requirements.requires_host_measurement_evidence()
            && self.host_capability_report.is_none()
        {
            return Some(UiMeasurementBasisDenial::MissingEvidence {
                slot: UiMeasurementEvidenceSlot::HostCapabilityReport,
            });
        }
        if self.host_results.has_intrinsic_results() && self.host_capability_report.is_none() {
            return Some(UiMeasurementBasisDenial::MissingEvidence {
                slot: UiMeasurementEvidenceSlot::HostCapabilityReport,
            });
        }
        if self
            .child_intrinsic_measurements
            .iter()
            .any(|evidence| evidence.host_measurement_result().is_some())
            && self.host_capability_report.is_none()
        {
            return Some(UiMeasurementBasisDenial::MissingEvidence {
                slot: UiMeasurementEvidenceSlot::HostCapabilityReport,
            });
        }
        if requirements.requires_host_font_metrics() && self.host_results.font_metrics.is_none() {
            return Some(
                UiMeasurementBasisDenial::MissingRequiredMeasurementEvidence {
                    category: UiMeasurementEvidenceCategory::FontMetrics,
                    slot: UiMeasurementEvidenceSlot::HostFontMetrics,
                },
            );
        }
        if let Some(denial) = basis_source_denial(requirements.basis_source(), &self.host_results) {
            return Some(denial);
        }
        if let Some(denial) =
            ownership_posture_denial(requirements.ownership_posture(), &self.host_results)
        {
            return Some(denial);
        }
        if requirements.requires_portal_anchor_metrics()
            && self.host_results.portal_anchor_rect.is_none()
        {
            return Some(
                UiMeasurementBasisDenial::MissingRequiredMeasurementEvidence {
                    category: UiMeasurementEvidenceCategory::PortalAnchorRect,
                    slot: UiMeasurementEvidenceSlot::PortalAnchorRect,
                },
            );
        }
        None
    }
}
