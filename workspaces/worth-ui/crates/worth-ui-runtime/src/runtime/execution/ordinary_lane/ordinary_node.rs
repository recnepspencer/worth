use std::rc::Rc;

use crate::runtime::{
    WorthUiOrdinaryExecutionLane, WorthUiPlanChildRange, WorthUiPlanNodeInputFamily,
    WorthUiRuntimeHandle,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiOrdinaryRegionLocator {
    plan_index: u32,
    slot_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiOrdinaryLaneNode {
    plan_index: u32,
    runtime_handle: WorthUiRuntimeHandle,
    lane: WorthUiOrdinaryExecutionLane,
    child_range: Option<WorthUiPlanChildRange>,
    linked_child_range: Option<WorthUiOrdinaryRegionLocator>,
    child_targets: Rc<[crate::runtime::planning::plan_topology::WorthUiPlanRegionHandle]>,
    ordinary_meaning:
        Option<Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>>,
}

impl WorthUiOrdinaryLaneNode {
    pub(crate) fn new(
        runtime_handle: WorthUiRuntimeHandle,
        lane: WorthUiOrdinaryExecutionLane,
        child_range: Option<WorthUiPlanChildRange>,
        linked_child_range: Option<WorthUiOrdinaryRegionLocator>,
        child_targets: Rc<[crate::runtime::planning::plan_topology::WorthUiPlanRegionHandle]>,
        ordinary_meaning: Option<
            Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>,
        >,
    ) -> Self {
        Self {
            plan_index: runtime_handle.plan_index(),
            runtime_handle,
            lane,
            child_range,
            linked_child_range,
            child_targets,
            ordinary_meaning,
        }
    }

    pub fn plan_index(&self) -> u32 {
        self.plan_index
    }

    pub fn runtime_handle(&self) -> WorthUiRuntimeHandle {
        self.runtime_handle
    }

    pub fn lane(&self) -> WorthUiOrdinaryExecutionLane {
        self.lane
    }

    pub(crate) fn linked_child_range(&self) -> Option<WorthUiOrdinaryRegionLocator> {
        self.linked_child_range
    }

    pub(crate) fn child_targets(
        &self,
    ) -> &[crate::runtime::planning::plan_topology::WorthUiPlanRegionHandle] {
        &self.child_targets
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub fn ordinary_semantic_digest(&self) -> u64 {
        self.ordinary_meaning.as_deref().map_or(
            0,
            crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning::semantic_digest,
        )
    }

    #[cfg(test)]
    pub(crate) fn ordinary_meaning(
        &self,
    ) -> Option<&crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning> {
        self.ordinary_meaning.as_deref()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn ordinary_meaning_reference(
        &self,
    ) -> Option<Rc<crate::runtime::planning::execution_plan_input::WorthUiPlanOrdinaryMeaning>>
    {
        self.ordinary_meaning.as_ref().map(Rc::clone)
    }
}

pub(crate) fn ordinary_node_from_regional(
    executable: &crate::runtime::planning::plan_topology::WorthUiPlanRegionExecutable,
    runtime_handle: WorthUiRuntimeHandle,
) -> Option<WorthUiOrdinaryLaneNode> {
    let plan_index = runtime_handle.plan_index();
    let lane = ordinary_lane_for_family(executable.family())?;
    Some(WorthUiOrdinaryLaneNode::new(
        runtime_handle,
        lane,
        executable.child_range_for_plan_index(plan_index),
        executable
            .linked_child_range()
            .map(WorthUiOrdinaryRegionLocator::from_region_handle),
        executable.child_targets_rc(),
        executable.ordinary_meaning_reference(),
    ))
}

pub(crate) fn ordinary_lane_for_family(
    family: WorthUiPlanNodeInputFamily,
) -> Option<WorthUiOrdinaryExecutionLane> {
    match family {
        WorthUiPlanNodeInputFamily::ComponentInvocation => {
            Some(WorthUiOrdinaryExecutionLane::WidgetShell)
        }
        WorthUiPlanNodeInputFamily::LayoutRegion => Some(WorthUiOrdinaryExecutionLane::ShellRegion),
        WorthUiPlanNodeInputFamily::ChildRange => {
            Some(WorthUiOrdinaryExecutionLane::ChildRangeTraversal)
        }
        WorthUiPlanNodeInputFamily::Command => Some(WorthUiOrdinaryExecutionLane::CommandSurface),
        WorthUiPlanNodeInputFamily::TokenStyle => {
            Some(WorthUiOrdinaryExecutionLane::TokenStyleSupport)
        }
        WorthUiPlanNodeInputFamily::StateSlot => {
            Some(WorthUiOrdinaryExecutionLane::StateSlotSupport)
        }
        WorthUiPlanNodeInputFamily::QueryViewBinding
        | WorthUiPlanNodeInputFamily::Accessibility
        | WorthUiPlanNodeInputFamily::DiagnosticsRef
        | WorthUiPlanNodeInputFamily::LanePartitionRef
        | WorthUiPlanNodeInputFamily::RenderResourceRef
        | WorthUiPlanNodeInputFamily::CanvasSpatial
        | WorthUiPlanNodeInputFamily::RealtimeOverlay => None,
    }
}

impl WorthUiOrdinaryRegionLocator {
    fn from_region_handle(
        handle: &crate::runtime::planning::plan_topology::WorthUiPlanRegionHandle,
    ) -> Self {
        Self {
            plan_index: u32::try_from(handle.stable_slot())
                .expect("regional slots satisfy compact handle capacity"),
            slot_generation: handle.slot_generation(),
        }
    }

    pub(crate) fn plan_index(self) -> u32 {
        self.plan_index
    }

    pub(crate) fn slot_generation(self) -> u64 {
        self.slot_generation
    }
}
