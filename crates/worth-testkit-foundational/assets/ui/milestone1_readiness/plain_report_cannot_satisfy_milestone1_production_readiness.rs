use worth_foundational::{
    milestone1_migration_readiness_report, require_milestone1_production_test_readiness,
};

fn main() {
    let report = milestone1_migration_readiness_report();
    let _ = require_milestone1_production_test_readiness(&report);
}
