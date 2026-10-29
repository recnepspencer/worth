#[allow(unused_imports)]
use {crate::shared_support as support};

use {crate::shared_support::assumption_lifecycle};
use {crate::shared_support::compile_fail::run_compile_fail_bundle};
use {crate::shared_support::compile_pass::run_compile_pass_bundle};

#[test]
fn assumption_freshness_readmission_and_downgrade_certification() {
    run_compile_fail_bundle(&crate::shared_support::assumption_lifecycle::compile_fail_bundle());
    run_compile_pass_bundle(&crate::shared_support::assumption_lifecycle::compile_pass_bundle());
}
