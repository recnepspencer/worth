use worth_foundational::require_canonical_production_test_readiness;

fn main() {
    let closeout = "Milestone 2 is ready for production testing.";

    let _ = require_canonical_production_test_readiness(&closeout);
}
