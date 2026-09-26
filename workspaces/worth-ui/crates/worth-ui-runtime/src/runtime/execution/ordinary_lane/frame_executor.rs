use crate::runtime::execution::handle_allocation::resolve_handle_row;
use crate::runtime::execution::ordinary_lane::frame_target::WorthUiOrdinaryFrameTargetKind;
use crate::runtime::{
    WorthUiHandleResolutionOutcome, WorthUiOrdinaryExecutionLane, WorthUiOrdinaryFrameTarget,
    WorthUiOrdinaryLaneCounters, WorthUiOrdinaryLaneFrameDenial,
    WorthUiOrdinaryLaneFrameDenialReason, WorthUiOrdinaryLaneFrameReceipt, WorthUiOrdinaryLaneNode,
    WorthUiOrdinaryLanePlan, WorthUiOrdinaryLaneTouchReceipt, WorthUiPlanNodeInputFamily,
    WorthUiRuntimeHandle,
};

use super::frame_receipt::WorthUiOrdinaryLaneFrameReceiptInput;

pub(crate) struct WorthUiOrdinaryLaneFrameExecutor;

impl WorthUiOrdinaryLaneFrameExecutor {
    pub(crate) fn execute(
        plan: &WorthUiOrdinaryLanePlan,
        target: WorthUiOrdinaryFrameTarget,
    ) -> Result<WorthUiOrdinaryLaneFrameReceipt, WorthUiOrdinaryLaneFrameDenial> {
        let counters = WorthUiOrdinaryLaneCounters::default();
        let Some(spec) = WorthUiIndexedTargetSpec::from_target_kind(target.kind()) else {
            return execute_root_shell(plan, target, counters);
        };
        execute_indexed_target(plan, target, spec, counters)
    }
}

#[derive(Clone, Copy)]
enum WorthUiTargetBreadth {
    Direct,
    Subtree,
}

#[derive(Clone, Copy)]
struct WorthUiIndexedTargetSpec {
    locator: crate::runtime::WorthUiRuntimeHandleLocator,
    expected_family: WorthUiPlanNodeInputFamily,
    expected_lane: WorthUiOrdinaryExecutionLane,
    breadth: WorthUiTargetBreadth,
}

impl WorthUiIndexedTargetSpec {
    fn from_target_kind(target: WorthUiOrdinaryFrameTargetKind) -> Option<Self> {
        match target {
            WorthUiOrdinaryFrameTargetKind::RootShell => None,
            WorthUiOrdinaryFrameTargetKind::Component(handle) => Some(Self::new(
                handle.locator(),
                WorthUiPlanNodeInputFamily::ComponentInvocation,
                WorthUiOrdinaryExecutionLane::WidgetShell,
                WorthUiTargetBreadth::Subtree,
            )),
            WorthUiOrdinaryFrameTargetKind::ChildRange(handle) => Some(Self::new(
                handle.locator(),
                WorthUiPlanNodeInputFamily::ChildRange,
                WorthUiOrdinaryExecutionLane::ChildRangeTraversal,
                WorthUiTargetBreadth::Subtree,
            )),
            WorthUiOrdinaryFrameTargetKind::Command(handle) => Some(Self::new(
                handle.locator(),
                WorthUiPlanNodeInputFamily::Command,
                WorthUiOrdinaryExecutionLane::CommandSurface,
                WorthUiTargetBreadth::Direct,
            )),
            WorthUiOrdinaryFrameTargetKind::TokenSupport(handle) => Some(Self::new(
                handle.locator(),
                WorthUiPlanNodeInputFamily::TokenStyle,
                WorthUiOrdinaryExecutionLane::TokenStyleSupport,
                WorthUiTargetBreadth::Direct,
            )),
            WorthUiOrdinaryFrameTargetKind::StateSlot(handle) => Some(Self::new(
                handle.locator(),
                WorthUiPlanNodeInputFamily::StateSlot,
                WorthUiOrdinaryExecutionLane::StateSlotSupport,
                WorthUiTargetBreadth::Direct,
            )),
        }
    }

    fn new(
        locator: crate::runtime::WorthUiRuntimeHandleLocator,
        expected_family: WorthUiPlanNodeInputFamily,
        expected_lane: WorthUiOrdinaryExecutionLane,
        breadth: WorthUiTargetBreadth,
    ) -> Self {
        Self {
            locator,
            expected_family,
            expected_lane,
            breadth,
        }
    }
}

struct WorthUiTouchAccumulator {
    row_count: usize,
    digest: u64,
}

impl WorthUiTouchAccumulator {
    fn new() -> Self {
        Self {
            row_count: 0,
            digest: 0x6f72_6469_6e61_7279,
        }
    }

    fn touch(&mut self, handle: WorthUiRuntimeHandle) {
        self.row_count += 1;
        self.digest = super::touch_receipt::fold_touch(self.digest, handle);
    }
}

fn execute_root_shell(
    plan: &WorthUiOrdinaryLanePlan,
    target: WorthUiOrdinaryFrameTarget,
    mut counters: WorthUiOrdinaryLaneCounters,
) -> Result<WorthUiOrdinaryLaneFrameReceipt, WorthUiOrdinaryLaneFrameDenial> {
    let roots = plan.root_shell_slots();
    if roots.is_empty() {
        counters.record_denial();
        return Err(WorthUiOrdinaryLaneFrameDenial::new(
            WorthUiOrdinaryLaneFrameDenialReason::TargetNotInOrdinaryPlan,
            None,
            counters,
        ));
    }

    let mut touches = WorthUiTouchAccumulator::new();
    let mut invalid_plan_index = None;
    roots.for_each(|stable_slot| {
        if invalid_plan_index.is_some() {
            return;
        }
        let plan_index = u32::try_from(stable_slot)
            .expect("root-shell regional slots satisfy compact handle capacity");
        let Some(row) = plan.row_for_plan_index(plan_index) else {
            invalid_plan_index = Some(plan_index);
            return;
        };
        if touch_subtree(plan, &row, &mut touches, &mut counters).is_err() {
            invalid_plan_index = Some(plan_index);
        }
    });
    if let Some(plan_index) = invalid_plan_index {
        return invalid_active_plan(plan_index, counters);
    }

    counters.record_root_shell_rows(touches.row_count);
    Ok(WorthUiOrdinaryLaneFrameReceipt::new(
        WorthUiOrdinaryLaneFrameReceiptInput {
            target,
            touch: WorthUiOrdinaryLaneTouchReceipt::root_shell(
                roots.clone(),
                touches.row_count,
                touches.digest,
            ),
            counters,
            certification: plan.certification(WorthUiOrdinaryExecutionLane::WidgetShell),
            requested_breadth: plan.counters().ordinary_plan_row_count(),
        },
    ))
}

fn execute_indexed_target(
    plan: &WorthUiOrdinaryLanePlan,
    target: WorthUiOrdinaryFrameTarget,
    spec: WorthUiIndexedTargetSpec,
    mut counters: WorthUiOrdinaryLaneCounters,
) -> Result<WorthUiOrdinaryLaneFrameReceipt, WorthUiOrdinaryLaneFrameDenial> {
    let plan_index = spec.locator.plan_index();
    let (row, resolution_evidence) = match resolve_handle_row(
        plan.handle_receipt().arena_identity(),
        spec.expected_family,
        spec.locator,
        |index| plan.row_for_plan_index(index),
        WorthUiOrdinaryLaneNode::runtime_handle,
    ) {
        Ok(resolved) => resolved,
        Err(evidence) => {
            let reason = ordinary_resolution_denial(evidence.outcome());
            if reason == WorthUiOrdinaryLaneFrameDenialReason::TargetNotInOrdinaryPlan {
                counters.record_denial();
            } else {
                counters.record_certification_failure();
            }
            return Err(
                WorthUiOrdinaryLaneFrameDenial::new(reason, Some(plan_index), counters)
                    .with_resolution_evidence(evidence),
            );
        }
    };
    if row.lane() != spec.expected_lane {
        counters.record_certification_failure();
        return Err(WorthUiOrdinaryLaneFrameDenial::new(
            WorthUiOrdinaryLaneFrameDenialReason::TargetFamilyMismatch,
            Some(plan_index),
            counters,
        ));
    }

    let requested_breadth = match spec.breadth {
        WorthUiTargetBreadth::Direct => 1,
        WorthUiTargetBreadth::Subtree => plan.counters().ordinary_plan_row_count(),
    };
    let touch = match collect_indexed_touch(plan, &row, spec.breadth, &mut counters) {
        Ok(touch) => touch,
        Err(()) => return invalid_active_plan(plan_index, counters),
    };
    Ok(
        WorthUiOrdinaryLaneFrameReceipt::new(WorthUiOrdinaryLaneFrameReceiptInput {
            target,
            touch,
            counters,
            certification: plan.certification(spec.expected_lane),
            requested_breadth,
        })
        .with_resolution_evidence(resolution_evidence),
    )
}

fn collect_indexed_touch(
    plan: &WorthUiOrdinaryLanePlan,
    row: &WorthUiOrdinaryLaneNode,
    breadth: WorthUiTargetBreadth,
    counters: &mut WorthUiOrdinaryLaneCounters,
) -> Result<WorthUiOrdinaryLaneTouchReceipt, ()> {
    match breadth {
        WorthUiTargetBreadth::Direct => {
            touch_row(row, counters);
            Ok(WorthUiOrdinaryLaneTouchReceipt::single(row))
        }
        WorthUiTargetBreadth::Subtree => {
            let mut touches = WorthUiTouchAccumulator::new();
            touch_subtree(plan, row, &mut touches, counters)?;
            counters.record_intentional_subtree_rows(touches.row_count);
            Ok(WorthUiOrdinaryLaneTouchReceipt::subtree(
                row,
                touches.row_count,
                touches.digest,
            ))
        }
    }
}

fn touch_subtree(
    plan: &WorthUiOrdinaryLanePlan,
    row: &WorthUiOrdinaryLaneNode,
    touches: &mut WorthUiTouchAccumulator,
    counters: &mut WorthUiOrdinaryLaneCounters,
) -> Result<(), ()> {
    touch_row(row, counters);
    touches.touch(row.runtime_handle());

    if let Some(linked_range) = row.linked_child_range() {
        let linked = plan
            .row_for_plan_index(linked_range.plan_index())
            .filter(|linked| {
                linked.runtime_handle().slot_generation().as_u64() == linked_range.slot_generation()
                    && linked.runtime_handle().family() == WorthUiPlanNodeInputFamily::ChildRange
            })
            .ok_or(())?;
        touch_subtree(plan, &linked, touches, counters)?;
    }
    for target in row.child_targets() {
        let plan_index = u32::try_from(target.stable_slot()).map_err(|_| ())?;
        let child = plan
            .row_for_plan_index(plan_index)
            .filter(|child| {
                child.runtime_handle().slot_generation().as_u64() == target.slot_generation()
            })
            .ok_or(())?;
        touch_subtree(plan, &child, touches, counters)?;
    }
    Ok(())
}

fn invalid_active_plan(
    plan_index: u32,
    mut counters: WorthUiOrdinaryLaneCounters,
) -> Result<WorthUiOrdinaryLaneFrameReceipt, WorthUiOrdinaryLaneFrameDenial> {
    counters.record_certification_failure();
    Err(WorthUiOrdinaryLaneFrameDenial::new(
        WorthUiOrdinaryLaneFrameDenialReason::ActivePlanNotOrdinaryExecutable,
        Some(plan_index),
        counters,
    ))
}

fn ordinary_resolution_denial(
    outcome: WorthUiHandleResolutionOutcome,
) -> WorthUiOrdinaryLaneFrameDenialReason {
    match outcome {
        WorthUiHandleResolutionOutcome::TargetMissing => {
            WorthUiOrdinaryLaneFrameDenialReason::TargetNotInOrdinaryPlan
        }
        WorthUiHandleResolutionOutcome::ForeignSessionArena => {
            WorthUiOrdinaryLaneFrameDenialReason::TargetArenaMismatch
        }
        WorthUiHandleResolutionOutcome::StaleSlotGeneration => {
            WorthUiOrdinaryLaneFrameDenialReason::TargetSlotGenerationMismatch
        }
        WorthUiHandleResolutionOutcome::WrongFamily => {
            WorthUiOrdinaryLaneFrameDenialReason::TargetFamilyMismatch
        }
        WorthUiHandleResolutionOutcome::Resolved => {
            unreachable!("resolved handle evidence is not a denial")
        }
    }
}

fn touch_row(row: &WorthUiOrdinaryLaneNode, counters: &mut WorthUiOrdinaryLaneCounters) {
    counters.record_frame_row_touch();
    match row.lane() {
        WorthUiOrdinaryExecutionLane::ChildRangeTraversal => counters.record_child_range_touch(),
        WorthUiOrdinaryExecutionLane::CommandSurface => counters.record_command_surface_touch(),
        WorthUiOrdinaryExecutionLane::TokenStyleSupport => counters.record_token_support_touch(),
        WorthUiOrdinaryExecutionLane::StateSlotSupport => counters.record_state_slot_touch(),
        WorthUiOrdinaryExecutionLane::WidgetShell | WorthUiOrdinaryExecutionLane::ShellRegion => {}
    }
}
