use worth_relational::facade::history::BranchId;
use worth_relational::facade::merge::{
    LoweredMergePlanSummary, MergeExecutionRequest, MergeIntent, RelationalMergeInspectionInput,
};

fn main() {
    let _ = RelationalMergeInspectionInput {
        request: MergeExecutionRequest {
            target_branch: BranchId("main".to_string()),
            source_branch: BranchId("feature".to_string()),
            merge_intent: MergeIntent::ReconcileIntoTarget,
        },
        lowered_plan: LoweredMergePlanSummary {
            record_count: 0,
            admitted_count: 0,
            blocked_count: 0,
            rejected_count: 0,
            fully_execution_ready: false,
            records: Vec::new().into(),
        },
    };
}
