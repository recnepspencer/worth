use crate::runtime::planning::WorthUiExecutionPlanLoweringFacts;
use crate::runtime::{
    WorthUiExecutionPlan, WorthUiLaneAdmission, WorthUiPlanTopologyCounters,
    WorthUiPlanTopologyDenial, WorthUiRuntimeHandleAllocation,
};

use super::assembly::{construct_execution_plan, construct_regional_successor_plan};
use super::validation::{
    verify_child_range_handles, verify_handle_allocation_receipt, verify_lane_admission,
    verify_runtime_handles,
};

pub(crate) struct WorthUiPlanTopologyAssembler;

impl WorthUiPlanTopologyAssembler {
    pub(crate) fn assemble_from_authority_with_lane_admission(
        authority: &WorthUiExecutionPlanLoweringFacts,
        handle_allocation: &WorthUiRuntimeHandleAllocation,
    ) -> Result<(WorthUiExecutionPlan, WorthUiLaneAdmission), WorthUiPlanTopologyDenial> {
        let lane_admission =
            crate::runtime::execution::lane_admission::WorthUiLaneAdmissionPlanner::admit(
                authority,
                authority
                    .candidate_application_authority()
                    .execution_lane_support(),
            )
            .map_err(|_| {
                WorthUiPlanTopologyDenial::new(
                    crate::runtime::WorthUiPlanTopologyDenialReason::LaneAdmissionMismatch,
                    Default::default(),
                )
            })?;
        let plan =
            Self::assemble_with_lane_admission(authority, handle_allocation, &lane_admission)?;
        Ok((plan, lane_admission))
    }

    pub(crate) fn assemble_with_lane_admission(
        authority: &WorthUiExecutionPlanLoweringFacts,
        handle_allocation: &WorthUiRuntimeHandleAllocation,
        lane_admission: &WorthUiLaneAdmission,
    ) -> Result<WorthUiExecutionPlan, WorthUiPlanTopologyDenial> {
        let mut counters = WorthUiPlanTopologyCounters::default();
        let node_inputs = authority.node_inputs();
        verify_handle_allocation_receipt(authority, handle_allocation, &mut counters)?;
        verify_lane_admission(
            node_inputs,
            lane_admission,
            crate::runtime::WorthUiRuntimeHandleAllocationBasis::from_lowering_authority(authority)
                .digest(),
            &mut counters,
        )?;
        verify_runtime_handles(node_inputs, handle_allocation, &mut counters)?;
        verify_child_range_handles(node_inputs, handle_allocation, &mut counters)?;
        construct_execution_plan(
            authority,
            node_inputs,
            handle_allocation,
            lane_admission.counters(),
            None,
            counters,
        )
    }

    pub(crate) fn assemble_successor_from_authority_with_lane_admission(
        authority: &WorthUiExecutionPlanLoweringFacts,
        handle_allocation: &WorthUiRuntimeHandleAllocation,
        predecessor_proof: super::WorthUiPredecessorRegionProof,
    ) -> Result<(WorthUiExecutionPlan, WorthUiLaneAdmission), WorthUiPlanTopologyDenial> {
        let mut counters = WorthUiPlanTopologyCounters::default();
        verify_handle_allocation_receipt(authority, handle_allocation, &mut counters)?;
        let region_successor = super::WorthUiPlanRegionSuccessorBuilder::build(predecessor_proof)
            .map_err(|denial| match denial {
            super::WorthUiPlanRegionSuccessorDenial::HandleCapacity(exhaustion) => {
                WorthUiPlanTopologyDenial::new(
                    crate::runtime::WorthUiPlanTopologyDenialReason::HandleCapacityExhausted(
                        exhaustion,
                    ),
                    Default::default(),
                )
            }
            super::WorthUiPlanRegionSuccessorDenial::CreateCollidesWithPredecessor
            | super::WorthUiPlanRegionSuccessorDenial::MissingPredecessorRegion
            | super::WorthUiPlanRegionSuccessorDenial::DuplicateMutation
            | super::WorthUiPlanRegionSuccessorDenial::RegionalStore(_) => regional_denial(),
        })?;
        let lane_admission =
            crate::runtime::execution::lane_admission::WorthUiLaneAdmissionPlanner::admit_regional_successor(
                authority,
                authority
                    .candidate_application_authority()
                    .execution_lane_support(),
                region_successor.store(),
            )
            .map_err(|_| regional_denial())?;
        let plan = construct_regional_successor_plan(
            authority,
            handle_allocation,
            lane_admission.counters(),
            region_successor,
            counters,
        );
        Ok((plan, lane_admission))
    }
}

fn regional_denial() -> WorthUiPlanTopologyDenial {
    WorthUiPlanTopologyDenial::new(
        crate::runtime::WorthUiPlanTopologyDenialReason::RegionalSuccessorMismatch,
        Default::default(),
    )
}
