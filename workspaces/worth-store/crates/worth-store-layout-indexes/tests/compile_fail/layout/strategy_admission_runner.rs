use super::compile_fail_support;

#[test]
fn caller_defined_rules_cannot_open_wal_layouts() {
    compile_fail_support::assert_compile_fails_in_ui_dir(
        "strategy_admission",
        "caller_defined_rule_cannot_open_wal_layout.rs",
        &["private field", "WalSegmentInspection"],
        &["worth_store_wal"],
    );
}
