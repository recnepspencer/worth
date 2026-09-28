use crate::runtime::{
    WorthUiExecutionLane, WorthUiExecutionPlan, WorthUiLaneAdmission, WorthUiLaneSupportStatus,
    WorthUiOrdinaryExecutionLane, WorthUiOrdinaryLaneCounters, WorthUiOrdinaryLanePlan,
    WorthUiOrdinaryLanePlanDenial, WorthUiOrdinaryLanePlanDenialReason, WorthUiPlanNodeInputFamily,
};

const ORDINARY_FAMILIES: [WorthUiPlanNodeInputFamily; 6] = [
    WorthUiPlanNodeInputFamily::ComponentInvocation,
    WorthUiPlanNodeInputFamily::LayoutRegion,
    WorthUiPlanNodeInputFamily::ChildRange,
    WorthUiPlanNodeInputFamily::Command,
    WorthUiPlanNodeInputFamily::TokenStyle,
    WorthUiPlanNodeInputFamily::StateSlot,
];

pub(crate) struct WorthUiOrdinaryLanePlanBuilder;

impl WorthUiOrdinaryLanePlanBuilder {
    pub(crate) fn build(
        execution_plan: &WorthUiExecutionPlan,
        lane_admission: &WorthUiLaneAdmission,
    ) -> Result<WorthUiOrdinaryLanePlan, WorthUiOrdinaryLanePlanDenial> {
        let mut counters = WorthUiOrdinaryLaneCounters::default();
        if !lane_is_supported(lane_admission, WorthUiExecutionLane::OrdinaryWidgetShell) {
            counters.record_denial();
            return Err(WorthUiOrdinaryLanePlanDenial::new(
                WorthUiOrdinaryLanePlanDenialReason::LaneAdmissionMissingOrdinarySupport,
                counters,
            ));
        }

        let mut ordinary_row_count = 0;
        for family in ORDINARY_FAMILIES {
            let family_count = execution_plan.regional_family_count(family);
            if family_count == 0 {
                continue;
            }
            let lane = super::ordinary_lane_for_family(family)
                .expect("the ordinary family catalog contains only ordinary families");
            require_lane_support(lane, lane_admission, &mut counters)?;
            ordinary_row_count += family_count;
        }
        counters.record_ordinary_plan_rows(ordinary_row_count);
        counters.record_skipped_nonordinary_plan_rows(
            execution_plan.region_count() - ordinary_row_count,
        );
        if ordinary_row_count == 0 {
            counters.record_denial();
            return Err(WorthUiOrdinaryLanePlanDenial::new(
                WorthUiOrdinaryLanePlanDenialReason::NoOrdinaryRows,
                counters,
            ));
        }

        let root_shell_slots = execution_plan.regional_root_shell_slot_view();
        let digest = fold(
            execution_plan.handle_receipt().basis_digest(),
            execution_plan.regional_semantic_digest(),
        );
        Ok(WorthUiOrdinaryLanePlan::new(
            super::WorthUiOrdinaryLanePlanInput {
                handle_receipt: execution_plan.handle_receipt(),
                support_digest: lane_admission.support_digest(),
                ordinary_plan_digest: digest,
                region_store: execution_plan.regional_store_clone(),
                root_shell_slots,
                counters,
            },
        ))
    }
}

fn require_lane_support(
    ordinary_lane: WorthUiOrdinaryExecutionLane,
    lane_admission: &WorthUiLaneAdmission,
    counters: &mut WorthUiOrdinaryLaneCounters,
) -> Result<(), WorthUiOrdinaryLanePlanDenial> {
    let required_lane = match ordinary_lane {
        WorthUiOrdinaryExecutionLane::WidgetShell
        | WorthUiOrdinaryExecutionLane::ShellRegion
        | WorthUiOrdinaryExecutionLane::ChildRangeTraversal
        | WorthUiOrdinaryExecutionLane::StateSlotSupport => {
            WorthUiExecutionLane::OrdinaryWidgetShell
        }
        WorthUiOrdinaryExecutionLane::CommandSurface => WorthUiExecutionLane::CommandSurface,
        WorthUiOrdinaryExecutionLane::TokenStyleSupport => WorthUiExecutionLane::StyleToken,
    };
    if lane_is_supported(lane_admission, required_lane) {
        return Ok(());
    }
    counters.record_denial();
    Err(WorthUiOrdinaryLanePlanDenial::new(
        missing_support_reason(required_lane),
        *counters,
    ))
}

fn lane_is_supported(lane_admission: &WorthUiLaneAdmission, lane: WorthUiExecutionLane) -> bool {
    lane_admission
        .posture_for(lane)
        .is_some_and(|row| row.status() == WorthUiLaneSupportStatus::Supported)
}

fn missing_support_reason(lane: WorthUiExecutionLane) -> WorthUiOrdinaryLanePlanDenialReason {
    match lane {
        WorthUiExecutionLane::OrdinaryWidgetShell => {
            WorthUiOrdinaryLanePlanDenialReason::LaneAdmissionMissingOrdinarySupport
        }
        WorthUiExecutionLane::CommandSurface => {
            WorthUiOrdinaryLanePlanDenialReason::LaneAdmissionMissingCommandSurfaceSupport
        }
        WorthUiExecutionLane::StyleToken => {
            WorthUiOrdinaryLanePlanDenialReason::LaneAdmissionMissingStyleTokenSupport
        }
        WorthUiExecutionLane::VirtualizedData
        | WorthUiExecutionLane::CanvasSpatial
        | WorthUiExecutionLane::RealtimeOverlayHud
        | WorthUiExecutionLane::QueryBound
        | WorthUiExecutionLane::DiagnosticsProjection
        | WorthUiExecutionLane::LaneBoundary
        | WorthUiExecutionLane::RenderResource
        | WorthUiExecutionLane::SpecialCaseExtension => {
            WorthUiOrdinaryLanePlanDenialReason::LaneAdmissionMissingOrdinarySupport
        }
    }
}

fn fold(digest: u64, value: u64) -> u64 {
    (digest ^ value).wrapping_mul(0x100000001b3)
}
