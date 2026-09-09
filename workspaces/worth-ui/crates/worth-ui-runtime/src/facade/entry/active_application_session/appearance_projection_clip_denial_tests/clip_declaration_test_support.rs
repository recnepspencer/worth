use super::support;
use worth_ui_dsl::{WorthUiArtifactInputBodyAtom as Atom, WorthUiRustAuthoredArtifactInputModule};

pub(super) fn clip_declaration(
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
    mounted: bool,
) -> worth_ui_dsl::WorthUiRustAuthoredArtifactInput {
    let ident = |value: &str| Atom::Identifier(value.to_owned());
    let mut region = vec![
        ident("region"),
        ident("workspace.region.primary"),
        Atom::LeftBrace,
        ident("sizing"),
        ident("workspace.sizing.mosaic_support"),
        Atom::Semicolon,
    ];
    if mounted {
        region.extend([
            ident("mount"),
            ident("appearance.mosaic.surface"),
            Atom::Semicolon,
        ]);
    }
    region.push(Atom::RightBrace);
    worth_ui_dsl::WorthUiRustAuthoredArtifactInput::from_modules([
        WorthUiRustAuthoredArtifactInputModule::new("appearance/consumer")
            .with_token(support::LEGACY_STATIC_PAINT_TOKEN, "#112233")
            .with_component_body_atoms(support::APPEARANCE_NODE_A, region)
            .with_appearance_role(role.clone())
            .with_component_appearance_role(
                support::APPEARANCE_NODE_B,
                worth_ui_dsl::UiAppearanceRoleAttachmentDeclaration::new(
                    role.role().clone(),
                    role.revision(),
                ),
            )
            .unwrap(),
    ])
}
