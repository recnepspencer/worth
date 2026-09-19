pub(crate) fn identity_text(role: &worth_ui_dsl::UiAppearanceRoleDeclaration) -> &str {
    role.role().as_str()
}
