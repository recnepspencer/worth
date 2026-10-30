use worth_foundational::require_milestone1_production_test_readiness;

fn main() {
    let closeout = "Milestone 1 is ready.";
    let _ = require_milestone1_production_test_readiness(&closeout);
}
