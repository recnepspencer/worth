#[allow(unused_imports)]
use crate::s5_tier_movement_future_chunk_compile_fail::*;
use std::path::Path;

use worth_store_test_support::compiler_boundary::read_ui_fixture_suite;

#[test]
fn future_chunk_placeholder_boundary_misuse_does_not_compile() {
    run(FIXTURES);
}
