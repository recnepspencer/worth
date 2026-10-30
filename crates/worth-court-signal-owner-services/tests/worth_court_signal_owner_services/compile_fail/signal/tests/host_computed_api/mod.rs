#[test]
fn host_computed_compile_fail_boundaries_hold() {
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/admitted_host_computed_read_set_fields_are_private.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/host_computed_descriptor_constructors_are_private.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/host_computed_descriptor_fields_are_private.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/host_computed_dependency_patch_fields_are_private.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/host_computed_evaluation_request_fields_are_private.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/host_computed_prepared_response_fields_are_private.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/host_computed_evaluation_response_fields_are_private.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/committed_host_computed_artifact_fields_are_private.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/staged_host_computed_artifact_fields_are_private.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_signal_owner_services/sources/worth_signal/ui/prepared_host_computed_evaluation_fields_are_private.rs")).expect("prebuilt compiler denial");
}
