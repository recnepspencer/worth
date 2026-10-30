use worth_foundational::{
    canonical_milestone2_production_readiness_report,
    require_canonical_production_test_readiness,
};

fn main() {
    let report = canonical_milestone2_production_readiness_report();

    let _ = require_canonical_production_test_readiness(&report);
}
