use worth_store_test_support::compiler_boundary::UiCompilerEnvironment;

#[test]
fn caller_defined_rules_cannot_open_wal_layouts() {
    crate::compile_fail_support::assert_compile_fails_in_ui_dir(
        UiCompilerEnvironment::Certification,
        "strategy_admission",
        "caller_defined_rule_cannot_open_wal_layout.rs",
        &["private field", "WalSegmentInspection"],
    );
}
