use super::claim_validation::reject_invalid_specialized_handle_claims;
use super::{WorthUiHandleArenaIdentity, WorthUiHandleSlotGeneration};
use crate::runtime::planning::WorthUiExecutionPlanLoweringFacts;
use crate::runtime::{
    WorthUiPlanNodeInput, WorthUiPlanNodeInputFamily, WorthUiRuntimeHandle,
    WorthUiRuntimeHandleAllocation, WorthUiRuntimeHandleAllocationBasis,
    WorthUiRuntimeHandleAllocationCounters, WorthUiRuntimeHandleAllocationDenial,
    WorthUiRuntimeHandleAllocationDenialReason, WorthUiRuntimeHandleAllocationReceipt,
    WorthUiRuntimeHandleFamilyWidths,
};

pub(crate) struct WorthUiRuntimeHandleAllocator;

struct HandleAllocationAccumulator {
    arena_identity: WorthUiHandleArenaIdentity,
    slot_generation: WorthUiHandleSlotGeneration,
    counters: WorthUiRuntimeHandleAllocationCounters,
    family_widths: WorthUiRuntimeHandleFamilyWidths,
    runtime_handles: Vec<WorthUiRuntimeHandle>,
}

impl WorthUiRuntimeHandleAllocator {
    pub(crate) fn authorize_regional_successor(
        authority: &WorthUiExecutionPlanLoweringFacts,
        arena_identity: WorthUiHandleArenaIdentity,
    ) -> WorthUiRuntimeHandleAllocation {
        let basis = WorthUiRuntimeHandleAllocationBasis::from_lowering_authority(authority);
        let receipt = WorthUiRuntimeHandleAllocationReceipt::from_basis(&basis, arena_identity);
        WorthUiRuntimeHandleAllocation::new(super::WorthUiRuntimeHandleAllocationInput {
            basis,
            receipt,
            family_widths: Default::default(),
            counters: Default::default(),
            runtime_handles: Vec::new(),
        })
    }

    pub(crate) fn allocate(
        authority: &WorthUiExecutionPlanLoweringFacts,
        arena_identity: WorthUiHandleArenaIdentity,
    ) -> Result<WorthUiRuntimeHandleAllocation, WorthUiRuntimeHandleAllocationDenial> {
        let node_inputs = authority.node_inputs();
        let basis = WorthUiRuntimeHandleAllocationBasis::from_lowering_authority(authority);
        let receipt = WorthUiRuntimeHandleAllocationReceipt::from_basis(&basis, arena_identity);
        let counters = reject_invalid_specialized_handle_claims(node_inputs)?;
        let mut allocation = HandleAllocationAccumulator::new(arena_identity, counters);

        for (plan_index, node_input) in node_inputs.iter().enumerate() {
            allocation.record_plan_node_input(plan_index, node_input)?;
        }

        Ok(WorthUiRuntimeHandleAllocation::new(
            super::WorthUiRuntimeHandleAllocationInput {
                basis,
                receipt,
                family_widths: allocation.family_widths,
                counters: allocation.counters,
                runtime_handles: allocation.runtime_handles,
            },
        ))
    }
}

impl HandleAllocationAccumulator {
    fn new(
        arena_identity: WorthUiHandleArenaIdentity,
        counters: WorthUiRuntimeHandleAllocationCounters,
    ) -> Self {
        Self {
            arena_identity,
            slot_generation: WorthUiHandleSlotGeneration::new(0),
            counters,
            family_widths: WorthUiRuntimeHandleFamilyWidths::default(),
            runtime_handles: Vec::new(),
        }
    }

    fn record_plan_node_input(
        &mut self,
        plan_index: usize,
        node_input: &WorthUiPlanNodeInput,
    ) -> Result<(), WorthUiRuntimeHandleAllocationDenial> {
        let plan_index = compact_plan_index(plan_index, self.counters)?;
        self.counters.record_plan_node_input();
        self.record_runtime_handle(plan_index, node_input.family());
        self.record_family_width(node_input);
        Ok(())
    }

    fn record_runtime_handle(&mut self, plan_index: u32, family: WorthUiPlanNodeInputFamily) {
        self.runtime_handles.push(WorthUiRuntimeHandle::new(
            family,
            plan_index,
            self.slot_generation,
            self.arena_identity,
        ));
        self.family_widths.record_runtime_handle();
    }

    fn record_family_width(&mut self, node_input: &WorthUiPlanNodeInput) {
        match node_input.family() {
            WorthUiPlanNodeInputFamily::ComponentInvocation => {
                self.family_widths.record_component_handle();
                self.counters.record_component_handle();
            }
            WorthUiPlanNodeInputFamily::Command => {
                self.family_widths.record_command_handle();
                self.counters.record_command_handle();
            }
            WorthUiPlanNodeInputFamily::TokenStyle => {
                self.family_widths.record_token_handle();
                self.counters.record_token_handle();
            }
            WorthUiPlanNodeInputFamily::ChildRange => {
                self.family_widths.record_child_range_handle();
                self.counters.record_child_range_handle();
            }
            WorthUiPlanNodeInputFamily::QueryViewBinding => {
                if node_input.query_binding_identity().is_some() {
                    self.family_widths.record_view_binding_handle();
                    self.counters.record_view_binding_handle();
                }
            }
            WorthUiPlanNodeInputFamily::LanePartitionRef => {
                self.family_widths.record_lane_handle();
                self.counters.record_lane_handle();
            }
            WorthUiPlanNodeInputFamily::StateSlot => {
                self.family_widths.record_state_slot_handle();
                self.counters.record_state_slot_handle();
            }
            WorthUiPlanNodeInputFamily::LayoutRegion
            | WorthUiPlanNodeInputFamily::Accessibility
            | WorthUiPlanNodeInputFamily::DiagnosticsRef
            | WorthUiPlanNodeInputFamily::RenderResourceRef
            | WorthUiPlanNodeInputFamily::CanvasSpatial
            | WorthUiPlanNodeInputFamily::RealtimeOverlay => {}
        }
    }
}

fn compact_plan_index(
    plan_index: usize,
    counters: WorthUiRuntimeHandleAllocationCounters,
) -> Result<u32, WorthUiRuntimeHandleAllocationDenial> {
    super::WorthUiHandleCapacity::plan_index(plan_index).map_err(|_| {
        denial(
            WorthUiRuntimeHandleAllocationDenialReason::PlanIndexCapacityExhausted,
            counters,
        )
    })
}

fn denial(
    reason: WorthUiRuntimeHandleAllocationDenialReason,
    counters: WorthUiRuntimeHandleAllocationCounters,
) -> WorthUiRuntimeHandleAllocationDenial {
    WorthUiRuntimeHandleAllocationDenial::new(reason, counters)
}
