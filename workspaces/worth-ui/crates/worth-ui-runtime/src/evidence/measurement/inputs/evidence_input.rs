use crate::capability::{CapabilitySnapshot, MosaicSizingContractId};
use crate::declaration::stable_text_digest;
use crate::graph::UiGraphNodeIdentity;
use crate::host::UiHostMeasurementAssumptionProfile;
use crate::runtime::WorthUiAdmittedDurableResizeInput;
use worth_ui_host_contract::WorthUiHostCapabilityReport;

use super::{
    child_intrinsic_evidence::UiChildIntrinsicMeasurementEvidence,
    sibling_resize_support::UiMeasurementSiblingResizeSupport,
};
use crate::evidence::measurement::{
    UiMeasurementEvidenceCategory, UiMeasurementResult, UiMeasurementValue,
    UiSettledQueryFactReceipt,
};

#[derive(Clone, Debug, PartialEq)]
pub enum MeasurementEvidenceInput {
    SettledQueryFact(UiSettledQueryFactReceipt),
    HostMeasurementResult(UiMeasurementResult),
    HostCapabilityReport(WorthUiHostCapabilityReport),
    ChildIntrinsicMeasurement(UiChildIntrinsicMeasurementEvidence),
    SiblingResizeSupport(UiMeasurementSiblingResizeSupport),
}

impl MeasurementEvidenceInput {
    pub(crate) fn operationally_matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::SettledQueryFact(left), Self::SettledQueryFact(right)) => left == right,
            (Self::HostMeasurementResult(left), Self::HostMeasurementResult(right)) => {
                left.operationally_matches(right)
            }
            (Self::HostCapabilityReport(left), Self::HostCapabilityReport(right)) => {
                left.profile_identity_digest() == right.profile_identity_digest()
            }
            (Self::ChildIntrinsicMeasurement(left), Self::ChildIntrinsicMeasurement(right)) => {
                left.operationally_matches(right)
            }
            (Self::SiblingResizeSupport(left), Self::SiblingResizeSupport(right)) => left == right,
            _ => false,
        }
    }

    pub fn settled_query_fact(receipt: &UiSettledQueryFactReceipt) -> Self {
        Self::SettledQueryFact(receipt.clone())
    }

    pub fn host_measurement_result(result: &UiMeasurementResult) -> Self {
        Self::HostMeasurementResult(result.clone())
    }

    pub fn host_capability_report(report: &WorthUiHostCapabilityReport) -> Self {
        Self::HostCapabilityReport(report.clone())
    }

    pub fn child_query_projection_fact(
        contributor_graph_node_identity: UiGraphNodeIdentity,
        receipt: &UiSettledQueryFactReceipt,
    ) -> Self {
        Self::ChildIntrinsicMeasurement(
            UiChildIntrinsicMeasurementEvidence::for_query_projection_fact(
                contributor_graph_node_identity,
                receipt,
            ),
        )
    }

    pub fn child_host_measurement_result(
        contributor_graph_node_identity: UiGraphNodeIdentity,
        result: &UiMeasurementResult,
    ) -> Self {
        Self::ChildIntrinsicMeasurement(
            UiChildIntrinsicMeasurementEvidence::for_host_measurement_result(
                contributor_graph_node_identity,
                result,
            ),
        )
    }

    pub fn mosaic_sibling_resize_support(
        snapshot: &CapabilitySnapshot,
        target_graph_node_identity: UiGraphNodeIdentity,
        sizing_contract_id: &MosaicSizingContractId,
    ) -> Option<Self> {
        UiMeasurementSiblingResizeSupport::from_mosaic_sizing_snapshot(
            snapshot,
            target_graph_node_identity,
            sizing_contract_id,
        )
        .map(Self::SiblingResizeSupport)
    }

    pub fn runtime_durable_resize_support(
        input: &WorthUiAdmittedDurableResizeInput,
        target_graph_node_identity: UiGraphNodeIdentity,
        axis_scope: crate::evidence::UiConstraintAxisScope,
        sizing_contract_id: Option<&MosaicSizingContractId>,
    ) -> Option<Self> {
        UiMeasurementSiblingResizeSupport::from_runtime_durable_resize_input(
            input,
            target_graph_node_identity,
            axis_scope,
            sizing_contract_id,
        )
        .map(Self::SiblingResizeSupport)
    }

    pub(crate) fn identity_digest(&self) -> u64 {
        match self {
            Self::SettledQueryFact(receipt) => {
                let (binding_digest, settlement_digest) = receipt.reference_reporting_digests();
                stable_text_digest("measurement-evidence-input:settled-query-fact")
                    ^ stable_text_digest(receipt.view_binding_id().as_str()).rotate_left(11)
                    ^ binding_digest.rotate_left(19)
                    ^ settlement_digest.rotate_left(29)
                    ^ receipt.observation_identity_digest().rotate_left(37)
            }
            Self::HostMeasurementResult(result) => {
                stable_text_digest("measurement-evidence-input:host-measurement-result")
                    ^ result.request_identity().as_u64().rotate_left(7)
                    ^ result.request_shape_digest().rotate_left(11)
                    ^ measurement_category_digest(result.evidence_category()).rotate_left(13)
                    ^ result
                        .assumption_profile()
                        .profile_identity_digest()
                        .rotate_left(19)
                    ^ measurement_shape_digest(result).rotate_left(23)
                    ^ measurement_value_digest(result.value()).rotate_left(29)
            }
            Self::HostCapabilityReport(report) => {
                stable_text_digest("measurement-evidence-input:host-capability-report")
                    ^ report.profile_identity_digest().rotate_left(7)
                    ^ report.observation_generation().as_u64().rotate_left(13)
            }
            Self::ChildIntrinsicMeasurement(evidence) => {
                stable_text_digest("measurement-evidence-input:child-intrinsic-measurement")
                    ^ evidence.identity_digest().rotate_left(7)
            }
            Self::SiblingResizeSupport(support) => {
                stable_text_digest("measurement-evidence-input:sibling-resize-support")
                    ^ support.identity_digest().rotate_left(7)
            }
        }
    }

    pub(crate) fn as_settled_query_fact(&self) -> Option<&UiSettledQueryFactReceipt> {
        match self {
            Self::SettledQueryFact(receipt) => Some(receipt),
            Self::HostMeasurementResult(_)
            | Self::HostCapabilityReport(_)
            | Self::ChildIntrinsicMeasurement(_)
            | Self::SiblingResizeSupport(_) => None,
        }
    }

    pub(crate) fn as_host_measurement_result(&self) -> Option<&UiMeasurementResult> {
        match self {
            Self::HostMeasurementResult(result) => Some(result),
            Self::SettledQueryFact(_)
            | Self::HostCapabilityReport(_)
            | Self::ChildIntrinsicMeasurement(_)
            | Self::SiblingResizeSupport(_) => None,
        }
    }

    pub(crate) fn as_host_capability_report(&self) -> Option<&WorthUiHostCapabilityReport> {
        match self {
            Self::HostCapabilityReport(report) => Some(report),
            Self::SettledQueryFact(_)
            | Self::HostMeasurementResult(_)
            | Self::ChildIntrinsicMeasurement(_)
            | Self::SiblingResizeSupport(_) => None,
        }
    }

    pub(crate) fn as_child_intrinsic_measurement(
        &self,
    ) -> Option<&UiChildIntrinsicMeasurementEvidence> {
        match self {
            Self::ChildIntrinsicMeasurement(evidence) => Some(evidence),
            Self::SettledQueryFact(_)
            | Self::HostMeasurementResult(_)
            | Self::HostCapabilityReport(_)
            | Self::SiblingResizeSupport(_) => None,
        }
    }

    pub(crate) fn as_sibling_resize_support(&self) -> Option<&UiMeasurementSiblingResizeSupport> {
        match self {
            Self::SiblingResizeSupport(support) => Some(support),
            Self::SettledQueryFact(_)
            | Self::HostMeasurementResult(_)
            | Self::HostCapabilityReport(_)
            | Self::ChildIntrinsicMeasurement(_) => None,
        }
    }
}

fn measurement_shape_digest(result: &UiMeasurementResult) -> u64 {
    stable_text_digest("measurement-result-shape")
        ^ result.evidence_generation().as_u64().rotate_left(7)
        ^ unit_posture_digest(result.assumption_profile()).rotate_left(13)
        ^ rounding_coordinate_digest(result).rotate_left(19)
}

fn unit_posture_digest(profile: UiHostMeasurementAssumptionProfile) -> u64 {
    profile.profile_identity_digest()
}

fn rounding_coordinate_digest(result: &UiMeasurementResult) -> u64 {
    stable_text_digest(result.unit_posture().as_str()).rotate_left(7)
        ^ stable_text_digest(result.coordinate_space().as_str()).rotate_left(13)
        ^ stable_text_digest(result.rounding_posture().as_str()).rotate_left(19)
}

fn measurement_category_digest(category: UiMeasurementEvidenceCategory) -> u64 {
    stable_text_digest(match category {
        UiMeasurementEvidenceCategory::TextIntrinsicSize => "text-intrinsic-size",
        UiMeasurementEvidenceCategory::TextBaselineMetrics => "text-baseline-metrics",
        UiMeasurementEvidenceCategory::FontMetrics => "font-metrics",
        UiMeasurementEvidenceCategory::NativeControlIntrinsicSize => {
            "native-control-intrinsic-size"
        }
        UiMeasurementEvidenceCategory::ViewportExtent => "viewport-extent",
        UiMeasurementEvidenceCategory::DpiScaleFactor => "dpi-scale-factor",
        UiMeasurementEvidenceCategory::PortalAnchorRect => "portal-anchor-rect",
        UiMeasurementEvidenceCategory::ScrollContainerViewport => "scroll-container-viewport",
    })
}

fn measurement_value_digest(value: &UiMeasurementValue) -> u64 {
    match value {
        UiMeasurementValue::TextIntrinsicSize(value) => {
            stable_text_digest("text-intrinsic-size")
                ^ f32_digest(value.width).rotate_left(7)
                ^ f32_digest(value.height).rotate_left(13)
        }
        UiMeasurementValue::TextBaselineMetrics(value) => {
            stable_text_digest("text-baseline-metrics")
                ^ f32_digest(value.ascent).rotate_left(7)
                ^ f32_digest(value.descent).rotate_left(13)
                ^ f32_digest(value.baseline).rotate_left(19)
        }
        UiMeasurementValue::FontMetrics(value) => {
            stable_text_digest("font-metrics")
                ^ f32_digest(value.ascent).rotate_left(7)
                ^ f32_digest(value.descent).rotate_left(13)
                ^ f32_digest(value.line_gap).rotate_left(19)
        }
        UiMeasurementValue::NativeControlIntrinsicSize(value) => {
            stable_text_digest("native-control-intrinsic-size")
                ^ f32_digest(value.width).rotate_left(7)
                ^ f32_digest(value.height).rotate_left(13)
        }
        UiMeasurementValue::ViewportExtent(value) => {
            stable_text_digest("viewport-extent")
                ^ f32_digest(value.width).rotate_left(7)
                ^ f32_digest(value.height).rotate_left(13)
        }
        UiMeasurementValue::DpiScaleFactor(value) => {
            stable_text_digest("dpi-scale-factor") ^ f32_digest(value.scale_factor).rotate_left(7)
        }
        UiMeasurementValue::PortalAnchorRect(value) => {
            stable_text_digest("portal-anchor-rect")
                ^ f32_digest(value.x).rotate_left(7)
                ^ f32_digest(value.y).rotate_left(13)
                ^ f32_digest(value.width).rotate_left(19)
                ^ f32_digest(value.height).rotate_left(23)
        }
        UiMeasurementValue::ScrollContainerViewport(value) => {
            stable_text_digest("scroll-container-viewport")
                ^ f32_digest(value.width).rotate_left(7)
                ^ f32_digest(value.height).rotate_left(13)
        }
    }
}

const fn f32_digest(value: f32) -> u64 {
    value.to_bits() as u64
}
