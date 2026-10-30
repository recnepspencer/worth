#[allow(unused_imports)]
use crate::shared_support as support;

use crate::compile_fail::harness::run_compile_fail_family;

#[test]
fn phase_boundaries_hold() {
    let bundle = crate::shared_support::core_artifact::compile_fail_bundle();
    run_compile_fail_family(&bundle, "phase_boundaries");
}
