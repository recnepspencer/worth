pub(crate) fn registration_candidate(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::capability::RegistrationCandidate {
    crate::capability::RegistrationCandidate::new(
        crate::capability::APPEARANCE_ROLE_FAMILY_NAME,
        super::identity::identity_text(role),
        crate::capability::CapabilitySupportKind::Admitted,
    )
}
