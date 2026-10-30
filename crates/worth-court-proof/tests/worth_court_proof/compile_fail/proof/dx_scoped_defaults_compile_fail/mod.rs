#[allow(unused_imports)]
use crate::shared_support as support;

use crate::compile_fail::harness::run_compile_fail_bundle;

#[test]
fn pleasant_scoped_defaults_do_not_hide_missing_progression() {
    run_compile_fail_bundle(&crate::shared_support::dx::compile_fail_bundle());
}
