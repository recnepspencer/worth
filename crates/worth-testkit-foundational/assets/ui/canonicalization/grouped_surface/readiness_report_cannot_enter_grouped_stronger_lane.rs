use worth_foundational::canonicalization_api::stronger_lane::readiness;

fn impossible<T>() -> T {
    panic!("compile-fail fixture should not run")
}

fn main() {
    let report: readiness::CanonicalProductionReadinessReport = impossible();
    let _ = readiness::require_canonical_production_test_readiness(&report);
}
