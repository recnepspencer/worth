use worth_foundational::{
    require_foundational_profile_milestone3_production_test_readiness,
    FoundationalProfileProductionReadinessReport,
};

fn main() {
    let report: FoundationalProfileProductionReadinessReport =
        worth_foundational::foundational_profile_milestone3_readiness_report();
    let _ = require_foundational_profile_milestone3_production_test_readiness(&report);
}
