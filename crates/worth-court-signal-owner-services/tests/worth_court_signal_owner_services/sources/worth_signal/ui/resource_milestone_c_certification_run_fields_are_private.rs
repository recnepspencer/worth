use worth_signal::facade::{
    ResourceMilestoneCCertificationRun, ResourceMilestoneCCertificationRunSummary,
    ResourceMilestoneCPolicyCertificationBundle, ResourceMilestoneCPolicyPerformanceCloseout,
    ResourceMilestoneCPolicyScenarioMatrix,
};

fn forged_bundle() -> ResourceMilestoneCPolicyCertificationBundle {
    panic!("compile-fail boundary fixture should never execute")
}

fn forged_scenario_matrix() -> ResourceMilestoneCPolicyScenarioMatrix {
    panic!("compile-fail boundary fixture should never execute")
}

fn forged_performance_closeout() -> ResourceMilestoneCPolicyPerformanceCloseout {
    panic!("compile-fail boundary fixture should never execute")
}

fn forged_summary() -> ResourceMilestoneCCertificationRunSummary {
    ResourceMilestoneCCertificationRunSummary {
        required_family_count: 7,
        certified_family_count: 7,
        failed_family_count: 0,
        bundle_digest: String::new(),
        required_scenario_count: 8,
        certified_scenario_count: 8,
        scenario_matrix_digest: String::new(),
        required_performance_claim_count: 5,
        certified_performance_claim_count: 5,
        performance_closeout_digest: String::new(),
    }
}

fn main() {
    let _forged = ResourceMilestoneCCertificationRun {
        schema_version: String::new(),
        bundle: forged_bundle(),
        scenario_matrix: forged_scenario_matrix(),
        performance_closeout: forged_performance_closeout(),
        summary: forged_summary(),
        run_digest: String::new(),
        passed: true,
    };
}
