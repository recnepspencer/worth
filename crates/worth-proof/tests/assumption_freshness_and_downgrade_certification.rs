#[allow(unused_imports)]
use crate::shared_support as support;

use support::assumption_lifecycle;
use support::compile_fail::run_compile_fail_bundle;
use support::compile_pass::run_compile_pass_bundle;

#[test]
fn assumption_freshness_readmission_and_downgrade_certification() {
    run_compile_fail_bundle(&assumption_lifecycle::compile_fail_bundle());
    run_compile_pass_bundle(&assumption_lifecycle::compile_pass_bundle());
}
