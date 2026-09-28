use crate::runtime::WorthUiPlanNodeInputFamily;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiPlanLookupIndex {
    component_plan_indexes: Vec<u32>,
    command_plan_indexes: Vec<u32>,
    token_plan_indexes: Vec<u32>,
    query_plan_indexes: Vec<u32>,
    lane_plan_indexes: Vec<u32>,
    render_resource_plan_indexes: Vec<u32>,
}

impl WorthUiPlanLookupIndex {
    pub(crate) fn new() -> Self {
        Self {
            component_plan_indexes: Vec::new(),
            command_plan_indexes: Vec::new(),
            token_plan_indexes: Vec::new(),
            query_plan_indexes: Vec::new(),
            lane_plan_indexes: Vec::new(),
            render_resource_plan_indexes: Vec::new(),
        }
    }

    pub(crate) fn record(&mut self, family: WorthUiPlanNodeInputFamily, plan_index: u32) -> bool {
        match family {
            WorthUiPlanNodeInputFamily::ComponentInvocation => {
                self.component_plan_indexes.push(plan_index);
                true
            }
            WorthUiPlanNodeInputFamily::Command => {
                self.command_plan_indexes.push(plan_index);
                true
            }
            WorthUiPlanNodeInputFamily::TokenStyle => {
                self.token_plan_indexes.push(plan_index);
                true
            }
            WorthUiPlanNodeInputFamily::QueryViewBinding => {
                self.query_plan_indexes.push(plan_index);
                true
            }
            WorthUiPlanNodeInputFamily::LanePartitionRef => {
                self.lane_plan_indexes.push(plan_index);
                true
            }
            WorthUiPlanNodeInputFamily::RenderResourceRef => {
                self.render_resource_plan_indexes.push(plan_index);
                true
            }
            WorthUiPlanNodeInputFamily::ChildRange
            | WorthUiPlanNodeInputFamily::StateSlot
            | WorthUiPlanNodeInputFamily::LayoutRegion
            | WorthUiPlanNodeInputFamily::Accessibility
            | WorthUiPlanNodeInputFamily::DiagnosticsRef
            | WorthUiPlanNodeInputFamily::CanvasSpatial
            | WorthUiPlanNodeInputFamily::RealtimeOverlay => false,
        }
    }

    pub fn component_plan_indexes(&self) -> &[u32] {
        &self.component_plan_indexes
    }

    pub fn command_plan_indexes(&self) -> &[u32] {
        &self.command_plan_indexes
    }

    pub fn token_plan_indexes(&self) -> &[u32] {
        &self.token_plan_indexes
    }

    pub fn query_plan_indexes(&self) -> &[u32] {
        &self.query_plan_indexes
    }

    pub fn lane_plan_indexes(&self) -> &[u32] {
        &self.lane_plan_indexes
    }

    pub fn render_resource_plan_indexes(&self) -> &[u32] {
        &self.render_resource_plan_indexes
    }

    pub fn entry_count(&self) -> usize {
        self.component_plan_indexes.len()
            + self.command_plan_indexes.len()
            + self.token_plan_indexes.len()
            + self.query_plan_indexes.len()
            + self.lane_plan_indexes.len()
            + self.render_resource_plan_indexes.len()
    }
}
