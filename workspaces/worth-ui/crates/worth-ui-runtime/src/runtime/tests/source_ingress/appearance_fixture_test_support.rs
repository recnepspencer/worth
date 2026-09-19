pub(crate) fn appearance_fixture(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> crate::facade::WorthUiRustAuthoredDeclarationFixture {
    let attachment = worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
        role.role().clone(),
        role.revision(),
    );
    crate::facade::WorthUiRustAuthoredDeclarationFixture::named("appearance-consumer-current")
        .with_appearance_role("appearance/consumer", role.clone())
        .with_component_appearance_role("appearance/consumer", super::ACTIVE_COMPONENT, attachment)
}

pub(super) fn appearance_fixture_without_attachment(
) -> crate::facade::WorthUiRustAuthoredDeclarationFixture {
    crate::facade::WorthUiRustAuthoredDeclarationFixture::named("appearance-capable-current")
        .with_semantic_artifact_spec(
            worth_ui_dsl::UiDslSemanticArtifactSpec::new(
                worth_ui_dsl::UiDslSemanticKey::new(super::ACTIVE_COMPONENT),
                worth_ui_dsl::UiDslSemanticFamily::Control,
                worth_ui_dsl::UiDslSourceProvenance::rust_authored("appearance/consumer", 0),
            )
            .with_structural_token(worth_ui_dsl::UiDslStructuralToken::new(
                "control:appearance-consumer",
            ))
            .with_component_reference(
                worth_ui_dsl::UiDslComponentReference::new(super::ACTIVE_COMPONENT).unwrap(),
            )
            .unwrap(),
        )
}
