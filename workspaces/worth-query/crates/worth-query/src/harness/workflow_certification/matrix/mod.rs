mod inspection_lanes;
mod lowering_lanes;
mod rejections;
mod rows;
mod runtime_lanes;

use super::lane::WorkflowCertificationMatrix;
use super::row_catalog::{WORKFLOW_CANONICAL_ROW_SPECS, WORKFLOW_REJECTION_ROW_SPECS};
use inspection_lanes::{
    conflict_inspection_lane, denied_conflict_inspection_lane, post_merge_inspection_lane,
};
use lowering_lanes::{merge_lowering_lane, preview_merge_lowering_lane, writeback_lowering_lane};
use rejections::rejection_row;
use rows::canonical_row;
use runtime_lanes::{
    preview_foundation_lane, runtime_conflict_lane, runtime_merge_lane, runtime_mutation_lane,
};

pub struct MilestoneFivePointFiveWorkflowCertificationAdapter;

impl MilestoneFivePointFiveWorkflowCertificationAdapter {
    pub fn workflow_declaration_taxonomy_and_context_binding_test(
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> WorkflowCertificationMatrix {
        let runtime_conflict = runtime_conflict_lane();
        let runtime_merge =
            runtime_merge_lane(crate::workflow::WorkflowBudgetClass::AuthorityTargetBounded);
        let runtime_merge_alt_budget =
            runtime_merge_lane(crate::workflow::WorkflowBudgetClass::InspectionBounded);
        let runtime_mutation = runtime_mutation_lane();
        let preview_foundation = preview_foundation_lane();
        let merge_lowering = merge_lowering_lane();
        let writeback_lowering = writeback_lowering_lane();
        let conflict_inspection = conflict_inspection_lane(request_execution);
        let denied_conflict_inspection = denied_conflict_inspection_lane(request_execution);
        let post_merge_inspection = post_merge_inspection_lane();
        let preview_merge_lowering = preview_merge_lowering_lane();

        WorkflowCertificationMatrix {
            suite_name: "Query Workflow Lowering And Writeback Boundary Test",
            rows: WORKFLOW_CANONICAL_ROW_SPECS
                .iter()
                .map(|spec| {
                    canonical_row(
                        spec,
                        &runtime_conflict,
                        &runtime_merge,
                        &runtime_merge_alt_budget,
                        &runtime_mutation,
                        &preview_foundation,
                        &merge_lowering,
                        &writeback_lowering,
                        &conflict_inspection,
                        &denied_conflict_inspection,
                        &post_merge_inspection,
                        &preview_merge_lowering,
                    )
                })
                .collect(),
            rejection_rows: WORKFLOW_REJECTION_ROW_SPECS
                .iter()
                .map(rejection_row)
                .collect(),
        }
    }
}
