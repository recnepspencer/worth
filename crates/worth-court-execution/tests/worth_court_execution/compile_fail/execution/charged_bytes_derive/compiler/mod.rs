#[test]
fn every_resident_field_requires_its_owner_contract() {
    worth_testkit_compiler::denied(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/worth_court_execution/sources/worth_execution/ui/charged_bytes/missing_owner.rs"
    ))
    .expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_execution/sources/worth_execution/ui/charged_bytes/unsupported_options.rs")).expect("prebuilt compiler denial");
    worth_testkit_compiler::denied(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worth_court_execution/sources/worth_execution/ui/charged_bytes/unsupported_items.rs")).expect("prebuilt compiler denial");
}
