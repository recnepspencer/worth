use crate::logic::planner::types::{
    ConcurrentApplyPlan, ConcurrentApplyReductionPlan, LoweredApplyPlan, LoweredTask,
    ReductionOrderingContract, ReductionWorkClass, ResolvedSignalPlannerPolicy,
};

pub(super) fn build_lowered_apply_plan(
    tasks: &[LoweredTask],
    policy: &ResolvedSignalPlannerPolicy,
) -> LoweredApplyPlan {
    let groups = super::concurrent_packets::build_stage_apply_groups(tasks, *policy);
    LoweredApplyPlan::GroupedConcurrent(ConcurrentApplyPlan {
        groups,
        reduction: ConcurrentApplyReductionPlan {
            ordering_contract: ReductionOrderingContract::StageTaskIndexOrder,
            allowed_work: ReductionWorkClass::DeterministicPublicationOnly,
        },
    })
}
