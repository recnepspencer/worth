#[test]
fn host_computed_compile_fail_boundaries_hold() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/host_computed_public_evaluator_boundary_is_implementable.rs");
    cases.compile_fail("tests/ui/admitted_host_computed_read_set_fields_are_private.rs");
    cases.compile_fail("tests/ui/host_computed_descriptor_constructors_are_private.rs");
    cases.compile_fail("tests/ui/host_computed_descriptor_fields_are_private.rs");
    cases.compile_fail("tests/ui/host_computed_dependency_patch_fields_are_private.rs");
    cases.compile_fail("tests/ui/host_computed_evaluation_request_fields_are_private.rs");
    cases.compile_fail("tests/ui/host_computed_prepared_response_fields_are_private.rs");
    cases.compile_fail("tests/ui/host_computed_evaluation_response_fields_are_private.rs");
    cases.compile_fail("tests/ui/committed_host_computed_artifact_fields_are_private.rs");
    cases.compile_fail("tests/ui/staged_host_computed_artifact_fields_are_private.rs");
    cases.compile_fail("tests/ui/prepared_host_computed_evaluation_fields_are_private.rs");
}
