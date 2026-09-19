use worth_ui_certification::topology::audit_appearance_owner_export_topology;

#[test]
fn appearance_adapters_use_sealed_owner_exports_and_close_only_seals_them() {
    let violations = audit_appearance_owner_export_topology(super::workspace_source_inventory());
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}
