use worth_foundational::{
    require_foundational_performance_milestone8_production_test_readiness,
    FoundationalPerformanceProductionReadinessReport,
};

fn main() {
    let report: FoundationalPerformanceProductionReadinessReport =
        worth_foundational::foundational_performance_milestone8_readiness_report();
    let _ = require_foundational_performance_milestone8_production_test_readiness(&report);
}
