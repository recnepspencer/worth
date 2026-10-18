use crate::harness::milestone_nine_certification::rows::canonical_rows;
use crate::harness::milestone_nine_certification::rows::rejection_rows;

pub struct MilestoneNineCertificationAdapter;

impl MilestoneNineCertificationAdapter {
    pub fn policy_tenant_context_admission_test() -> MilestoneNineCertificationMatrix {
        MilestoneNineCertificationMatrix {
            suite_name: "Policy And Tenant Context Admission Test",
            rows: canonical_rows(),
            rejection_rows: rejection_rows(),
        }
    }
}
use crate::harness::milestone_nine_certification::bundles::MilestoneNineCertificationMatrix;
