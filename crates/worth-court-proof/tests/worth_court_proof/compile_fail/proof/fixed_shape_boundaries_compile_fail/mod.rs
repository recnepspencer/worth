#[allow(unused_imports)]
use crate::shared_support as support;

use crate::compile_fail::harness::run_compile_fail_families;

#[test]
fn fixed_shape_and_proven_collection_boundaries_hold() {
    let bundle = crate::shared_support::core_artifact::compile_fail_bundle();
    run_compile_fail_families(
        &bundle,
        &["proven_collection_boundaries", "fixed_shape_boundaries"],
    );
}
