#[allow(unused_imports)] use crate::blob_harness_public_boundary_compile_fail::*;
use {std::path::Path};

use {worth_store_test_support::compiler_boundary::cargo_dependency_manifest, worth_store_test_support::compiler_boundary::run_cargo_ui_fixture_suite};

#[test]
fn blob_harness_envelope_raw_constructor_is_not_public() {
    run_case(
        "blob-harness-envelope",
        false,
        "blob_harness_envelope_raw_constructor_is_private.rs",
        &["new", "private"],
    );
}

#[test]
fn blob_harness_executed_witness_cannot_be_forged_from_raw_fields() {
    run_case(
        "blob-harness-witness",
        true,
        "blob_harness_executed_witness_fields_are_private.rs",
        &["BlobHarnessExecutedWitness", "private"],
    );
}

#[test]
fn blob_harness_execution_authority_is_not_public_on_default_surface() {
    run_case(
        "blob-harness-default",
        false,
        "blob_harness_execution_authority_is_not_public.rs",
        &["execute_blob_harness", "BlobHarnessExecutionInput"],
    );
}
