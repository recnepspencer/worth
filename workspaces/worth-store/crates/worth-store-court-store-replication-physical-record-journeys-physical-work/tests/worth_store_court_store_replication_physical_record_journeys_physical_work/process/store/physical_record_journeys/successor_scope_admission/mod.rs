#[allow(unused_imports)]
use crate::physical_record_journeys::successor_scope_admission::*;

#[test]
fn successor_scopes_are_exact_isolated_global_and_released() {
    let root = tempfile::tempdir().unwrap();
    let serving = successor_scope_runtime(root.path());
    assert_exact_scope_ceilings(&serving);
    assert_scope_isolation(&serving);
    assert_global_envelope_and_release(&serving);
}
