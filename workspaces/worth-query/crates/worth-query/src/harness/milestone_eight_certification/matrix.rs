use super::*;

pub struct MilestoneEightCertificationAdapter;

impl MilestoneEightCertificationAdapter {
    pub fn scope_template_view_shape_semantic_parity_test(
        resource_request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> MilestoneEightCertificationMatrix {
        MilestoneEightCertificationMatrix {
            suite_name: "Scope / Template / View-Shape Semantic Parity Test",
            rows: canonical_rows(resource_request),
            rejection_rows: rejection_rows(resource_request),
        }
    }
}
