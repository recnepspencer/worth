use std::path::PathBuf;

use crate::{
    UiAppearanceAspect, UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceAxisPredicate,
    UiAppearanceCell, UiAppearancePartitionAuthoring, UiAppearanceRole,
    UiAppearanceRoleAttachmentDeclaration, UiAppearanceRoleIdentity, UiAppearanceRoleRevision,
    UiAppearanceStateAxis, UiBackdropDeclaration, UiBackdropExtentBasis, UiBackdropIdentity,
    UiBackdropMotionBasis, UiBackdropPlacement, UiBackdropPresenceBasis, UiBackdropScope,
    UiDslComponentReference, UiDslSemanticFamily, UiDslSemanticKey, UiPortalDeclarationId,
    UiSemanticSurfaceDeclarationIdentity, UiThemeSlotIdentity, UiThemeValueKind,
    WorthUiArtifactInputBodyAtom, WorthUiAuthoredSourceInput, WorthUiDslCompiler,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
    WorthUiSealedSemanticPackage, WorthUiSemanticArtifactDeclaration,
    WorthUiServiceDeclarationMeaning, WorthUiServiceFamily,
};

fn slot(value: &str) -> UiThemeSlotIdentity {
    UiThemeSlotIdentity::new(value).expect("test slot identity is valid")
}

fn component_role() -> crate::UiAppearanceRoleDeclaration {
    UiAppearanceRole::new(UiAppearanceRoleIdentity::new("action.primary").unwrap())
        .applies_to_component(UiDslComponentReference::new("platform.control.activation").unwrap())
        .cover(
            UiAppearanceAspect::Background,
            UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
                UiAppearanceStateAxis::Hover,
            )])
            .with_cell(
                UiAppearanceCell::named("outside")
                    .when([UiAppearanceAxisPredicate::exact(
                        UiAppearanceAxisClass::HoverOutside,
                    )])
                    .uses_slot(slot("action.primary.background"), UiThemeValueKind::Color),
            )
            .otherwise_same_as("outside"),
        )
        .expect("component role partition is valid")
        .build()
        .expect("component role is valid")
}

fn backdrop_role() -> crate::UiAppearanceRoleDeclaration {
    UiAppearanceRole::new(UiAppearanceRoleIdentity::new("overlay.scrim").unwrap())
        .applies_to_backdrop()
        .cover(
            UiAppearanceAspect::Background,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([])
                    .uses_slot(slot("overlay.scrim.background"), UiThemeValueKind::Color),
            ),
        )
        .expect("backdrop color partition is valid")
        .cover(
            UiAppearanceAspect::Opacity,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([])
                    .uses_slot(slot("overlay.scrim.opacity"), UiThemeValueKind::Opacity),
            ),
        )
        .expect("backdrop opacity partition is valid")
        .build()
        .expect("backdrop role is valid")
}

fn compile_file(
    source: &str,
) -> Result<WorthUiSealedSemanticPackage, crate::WorthUiDslCompileReport> {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("app/main.wui", source),
    )
}

fn compile_rust(
    module: WorthUiRustAuthoredArtifactInputModule,
) -> Result<WorthUiSealedSemanticPackage, crate::WorthUiDslCompileReport> {
    WorthUiDslCompiler::compile_rust_authored(&WorthUiRustAuthoredArtifactInput::from_modules([
        module,
    ]))
}

fn typed_backdrop(
    identity: u64,
    surface: u64,
    placement: UiBackdropPlacement,
    role: &crate::UiAppearanceRoleDeclaration,
) -> UiBackdropDeclaration {
    let surface = UiSemanticSurfaceDeclarationIdentity::new(surface).unwrap();
    UiBackdropDeclaration::admit(
        UiBackdropIdentity::new(identity).unwrap(),
        surface,
        UiBackdropScope::SurfaceSingleton,
        UiBackdropExtentBasis::SurfaceViewport(surface),
        UiBackdropPresenceBasis::Always,
        UiBackdropMotionBasis::None,
        placement,
        role,
    )
    .expect("typed backdrop should pass Gate-0 admission")
}

fn rust_portal(name: &str, anchor: &str) -> WorthUiSemanticArtifactDeclaration {
    let atoms = [
        WorthUiArtifactInputBodyAtom::Identifier("anchor".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier(anchor.to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("layer".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("transient".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("dismiss".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("escape".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("focus".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("first_enabled".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("motion".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("system_popover".to_owned()),
    ];
    let service =
        WorthUiServiceDeclarationMeaning::parse(WorthUiServiceFamily::Portal, name, &atoms)
            .expect("Rust portal service should parse");
    WorthUiSemanticArtifactDeclaration::new(
        UiDslSemanticKey::new(name),
        UiDslSemanticFamily::RuntimeService,
    )
    .with_service_declaration(service)
}

#[test]
fn rust_and_file_component_authoring_keep_explicit_role_attachment() {
    let file = compile_file(
        r#"
        appearance role action.primary applies_to platform.control.activation {
            background over [hover] {
                cell outside when hover = outside use token(action.primary.background)
                otherwise same_as outside
            }
        }
        component platform.control.activation {
            appearance { role action.primary }
            ;
        }
        "#,
    )
    .expect("file declaration should compile");
    let rust = compile_rust(
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_appearance_role(component_role())
            .with_component_appearance_role(
                "platform.control.activation",
                UiAppearanceRoleAttachmentDeclaration::new(
                    UiAppearanceRoleIdentity::new("action.primary").unwrap(),
                    UiAppearanceRoleRevision::new(1).unwrap(),
                ),
            )
            .expect("Rust component attachment should be unique"),
    )
    .expect("Rust declaration should compile");
    let has_attachment = file
        .declaration_lowering_receipts()
        .into_iter()
        .any(|receipt| {
            receipt
                .semantic_artifact()
                .appearance_role_attachment()
                .is_some()
        });
    assert!(
        has_attachment,
        "file attachment should reach the lowering receipt"
    );
    assert_eq!(
        file.appearance_role_declarations()
            .next()
            .unwrap()
            .role()
            .canonical_bytes(),
        rust.appearance_role_declarations()
            .next()
            .unwrap()
            .role()
            .canonical_bytes()
    );
}

#[test]
fn file_and_rust_backdrops_converge_on_equal_typed_graph_and_bytes() {
    let source = r#"
        surface beta.surface {}
        portal beta.portal {
            anchor beta.anchor
            layer transient
            dismiss escape
            focus first_enabled
            motion system_popover
        }
        appearance role overlay.scrim applies_to backdrop {
            background use token(overlay.scrim.background)
            opacity use token(overlay.scrim.opacity)
        }
        backdrop z.scrim {
            scope surface_singleton
            extent surface_viewport beta.surface
            presence always
            motion none
            place immediately_before portal beta.portal
            appearance { role overlay.scrim }
        }
        surface alpha.surface {}
        portal alpha.portal {
            anchor alpha.anchor
            layer transient
            dismiss escape
            focus first_enabled
            motion system_popover
        }
        backdrop a.scrim {
            scope surface_singleton
            extent surface_viewport alpha.surface
            presence always
            motion none
            place immediately_before portal alpha.portal
            appearance { role overlay.scrim }
        }
    "#;
    let permuted = r#"
        appearance role overlay.scrim applies_to backdrop {
            background use token(overlay.scrim.background)
            opacity use token(overlay.scrim.opacity)
        }
        backdrop a.scrim {
            scope surface_singleton
            extent surface_viewport alpha.surface
            presence always
            motion none
            place immediately_before portal alpha.portal
            appearance { role overlay.scrim }
        }
        portal alpha.portal {
            anchor alpha.anchor
            layer transient
            dismiss escape
            focus first_enabled
            motion system_popover
        }
        surface alpha.surface {}
        backdrop z.scrim {
            scope surface_singleton
            extent surface_viewport beta.surface
            presence always
            motion none
            place immediately_before portal beta.portal
            appearance { role overlay.scrim }
        }
        portal beta.portal {
            anchor beta.anchor
            layer transient
            dismiss escape
            focus first_enabled
            motion system_popover
        }
        surface beta.surface {}
    "#;
    let file = compile_file(source).expect("file declarations should compile");
    let permuted_file = compile_file(permuted).expect("permuted file declarations should compile");
    let role = backdrop_role();
    let rust = compile_rust(
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_surface("beta.surface")
            .with_semantic_declaration(rust_portal("beta.portal", "beta.anchor"))
            .with_appearance_role(role.clone())
            .with_backdrop(typed_backdrop(
                2,
                2,
                UiBackdropPlacement::ImmediatelyBeforePortal(
                    UiPortalDeclarationId::new(2).unwrap(),
                ),
                &role,
            ))
            .with_surface("alpha.surface")
            .with_semantic_declaration(rust_portal("alpha.portal", "alpha.anchor"))
            .with_backdrop(typed_backdrop(
                1,
                1,
                UiBackdropPlacement::ImmediatelyBeforePortal(
                    UiPortalDeclarationId::new(1).unwrap(),
                ),
                &role,
            )),
    )
    .expect("Rust declarations should compile");
    let file_backdrops = file
        .backdrop_declarations()
        .map(|declaration| declaration.declaration().clone())
        .collect::<Vec<_>>();
    let rust_backdrops = rust
        .backdrop_declarations()
        .map(|declaration| declaration.declaration().clone())
        .collect::<Vec<_>>();
    let permuted_backdrops = permuted_file
        .backdrop_declarations()
        .map(|declaration| declaration.declaration().clone())
        .collect::<Vec<_>>();
    assert_eq!(file_backdrops, rust_backdrops);
    assert_eq!(file_backdrops, permuted_backdrops);
    assert_eq!(file.overlay_relation_graph(), rust.overlay_relation_graph());
    assert_eq!(
        file.overlay_relation_graph().unwrap().canonical_bytes(),
        rust.overlay_relation_graph().unwrap().canonical_bytes()
    );
    assert_eq!(file.identity(), rust.identity());
    assert_eq!(file.identity(), permuted_file.identity());
}
