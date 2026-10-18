#[allow(unused_imports)]
use crate::shared_support as support;

use support::compile_fail::run_compile_fail_family;
use support::core_artifact;

#[test]
fn phase_boundaries_hold() {
    let bundle = core_artifact::compile_fail_bundle();
    run_compile_fail_family(&bundle, "phase_boundaries");
}
