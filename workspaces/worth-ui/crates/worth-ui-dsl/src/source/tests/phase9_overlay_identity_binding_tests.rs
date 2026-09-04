use std::path::PathBuf;

use crate::{
    UiAppearanceAspect, UiAppearanceCell, UiAppearancePartitionAuthoring, UiAppearanceRole,
    UiAppearanceRoleIdentity, UiBackdropDeclaration, UiBackdropExtentBasis, UiBackdropIdentity,
    UiBackdropMotionBasis, UiBackdropPlacement, UiBackdropPresenceBasis, UiBackdropScope,
    UiDslSemanticFamily, UiDslSemanticKey, UiMosaicRegionDeclarationIdentity,
    UiSemanticSurfaceDeclarationIdentity, UiThemeSlotIdentity, UiThemeValueKind,
    WorthUiArtifactInputBodyAtom, WorthUiAuthoredSourceInput, WorthUiDslCompileDiagnosticCode,
    WorthUiDslCompiler, WorthUiIntentInteractionFamily, WorthUiIntentInteractionRoute,
    WorthUiPortalDismissalSet, WorthUiPortalLayer, WorthUiRustAuthoredArtifactInput,
    WorthUiRustAuthoredArtifactInputModule, WorthUiSemanticArtifactDeclaration,
    WorthUiSemanticDeclaration, WorthUiServiceDeclarationMeaning, WorthUiServiceFamily,
};

fn compile_file(
    source: &str,
) -> Result<crate::WorthUiSealedSemanticPackage, crate::WorthUiDslCompileReport> {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("app/main.wui", source),
    )
}

fn compile_rust(
    module: WorthUiRustAuthoredArtifactInputModule,
) -> Result<crate::WorthUiSealedSemanticPackage, crate::WorthUiDslCompileReport> {
    WorthUiDslCompiler::compile_rust_authored(&WorthUiRustAuthoredArtifactInput::from_modules([
        module,
    ]))
}

fn first_component_route(
    package: &crate::WorthUiSealedSemanticPackage,
) -> WorthUiIntentInteractionRoute {
    for module_id in package.module_ids() {
        for view in package
            .declaration_views(module_id)
            .expect("sealed package contains its source modules")
        {
            if let WorthUiSemanticDeclaration::Component(component) = view.declaration() {
                if let Some(route) = component.structure().interaction_routes().first() {
                    return route.clone();
                }
            }
        }
    }
    panic!("source fixture must contain one component interaction route");
}

fn first_portal_surface(package: &crate::WorthUiSealedSemanticPackage) -> Option<&str> {
    package
        .service_declarations()
        .find_map(|(service, _)| match service {
            WorthUiServiceDeclarationMeaning::Portal(portal) => portal.surface(),
            _ => None,
        })
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

fn rust_backdrop_role() -> crate::UiAppearanceRoleDeclaration {
    UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new("overlay.scrim").unwrap())
        .applies_to_backdrop()
        .cover(
            UiAppearanceAspect::Background,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([]).uses_slot(
                    UiThemeSlotIdentity::new("overlay.scrim.background").unwrap(),
                    UiThemeValueKind::Color,
                ),
            ),
        )
        .expect("Rust backdrop background should compile")
        .cover(
            UiAppearanceAspect::Opacity,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([]).uses_slot(
                    UiThemeSlotIdentity::new("overlay.scrim.opacity").unwrap(),
                    UiThemeValueKind::Opacity,
                ),
            ),
        )
        .expect("Rust backdrop opacity should compile")
        .build()
        .expect("Rust backdrop role should compile")
}

fn typed_region_backdrop(role: &crate::UiAppearanceRoleDeclaration) -> UiBackdropDeclaration {
    let surface = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let region = UiMosaicRegionDeclarationIdentity::new(7).unwrap();
    UiBackdropDeclaration::admit(
        UiBackdropIdentity::new(1).unwrap(),
        surface,
        UiBackdropScope::SurfaceSingleton,
        UiBackdropExtentBasis::PresentedMosaicRegion { surface, region },
        UiBackdropPresenceBasis::Always,
        UiBackdropMotionBasis::None,
        UiBackdropPlacement::AboveSurfaceContent,
        role,
    )
    .expect("typed Rust mosaic-region backdrop should pass admission")
}

#[test]
fn file_overlay_bindings_preserve_backdrop_and_region_ids_when_declarations_move() {
    let source = r#"
        surface alpha.surface {}
        appearance role overlay.scrim applies_to backdrop {
            background use token(overlay.scrim.background)
            opacity use token(overlay.scrim.opacity)
        }
        backdrop alpha.scrim {
            scope surface_singleton
            extent presented_mosaic_region alpha.surface alpha.region
            presence always
            motion none
            place above_surface_content
            appearance { role overlay.scrim }
        }
    "#;
    let permuted = r#"
        backdrop alpha.scrim {
            appearance { role overlay.scrim }
            place above_surface_content
            motion none
            presence always
            extent presented_mosaic_region alpha.surface alpha.region
            scope surface_singleton
        }
        surface alpha.surface {}
        appearance role overlay.scrim applies_to backdrop {
            opacity use token(overlay.scrim.opacity)
            background use token(overlay.scrim.background)
        }
    "#;
    let package = compile_file(source).expect("source overlay should compile");
    let permuted_package = compile_file(permuted).expect("permuted overlay should compile");
    let bindings = package.overlay_declaration_bindings();
    let permuted_bindings = permuted_package.overlay_declaration_bindings();

    let backdrop_identity = bindings
        .backdrop_named("alpha.scrim")
        .expect("source must issue a backdrop identity");
    let permuted_backdrop_identity = permuted_bindings
        .backdrop_named("alpha.scrim")
        .expect("permuted source must issue a backdrop identity");
    let surface_identity = bindings
        .surface_named("alpha.surface")
        .expect("source must issue a surface identity");
    let permuted_surface_identity = permuted_bindings
        .surface_named("alpha.surface")
        .expect("permuted source must issue a surface identity");
    let region_identity = bindings
        .region_named("alpha.surface", "alpha.region")
        .expect("source must issue a mosaic-region identity");
    let permuted_region_identity = permuted_bindings
        .region_named("alpha.surface", "alpha.region")
        .expect("permuted source must issue a mosaic-region identity");
    assert_eq!(backdrop_identity, permuted_backdrop_identity);
    assert_eq!(surface_identity, permuted_surface_identity);
    assert_eq!(region_identity, permuted_region_identity);
    let backdrop = package
        .backdrop_declarations()
        .next()
        .expect("source must seal a backdrop")
        .declaration();
    let permuted_backdrop = permuted_package
        .backdrop_declarations()
        .next()
        .expect("permuted source must seal a backdrop")
        .declaration();
    assert_eq!(backdrop, permuted_backdrop);
    assert_eq!(backdrop.identity(), backdrop_identity);
    assert_eq!(backdrop.surface(), surface_identity);
    assert_eq!(
        backdrop.extent(),
        UiBackdropExtentBasis::PresentedMosaicRegion {
            surface: surface_identity,
            region: region_identity,
        }
    );
    assert_eq!(permuted_backdrop.identity(), permuted_backdrop_identity);
    assert_eq!(permuted_backdrop.surface(), permuted_surface_identity);
    assert_eq!(
        permuted_backdrop.extent(),
        UiBackdropExtentBasis::PresentedMosaicRegion {
            surface: permuted_surface_identity,
            region: permuted_region_identity,
        }
    );
}

#[test]
fn duplicate_portal_identity_is_denied_by_compiler_binding_collection() {
    let report = compile_file(
        r#"
        portal overlay.menu {
            anchor first.anchor
            layer transient
            dismiss escape
            focus first_enabled
            motion system_popover
        }
        portal overlay.menu {
            anchor second.anchor
            layer transient
            dismiss escape
            focus first_enabled
            motion system_popover
        }
        "#,
    )
    .expect_err("duplicate portal names must not create a second identity lane");

    assert_eq!(
        report.diagnostics()[0].identity().code(),
        WorthUiDslCompileDiagnosticCode::DuplicateAppearanceDeclaration
    );
}

#[test]
fn duplicate_rust_portal_identity_is_denied_by_compiler_binding_collection() {
    let report = compile_rust(
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_semantic_declaration(rust_portal("overlay.menu", "first.anchor"))
            .with_semantic_declaration(rust_portal("overlay.menu", "second.anchor")),
    )
    .expect_err("duplicate Rust portal names must be denied by the compiler");

    assert_eq!(
        report.diagnostics()[0].identity().code(),
        WorthUiDslCompileDiagnosticCode::DuplicateAppearanceDeclaration
    );
    assert_eq!(
        report.diagnostics()[0].message(),
        "Rust-authored input could not be normalized into a sealed DSL package"
    );
}

#[test]
fn file_and_rust_routes_retain_the_explicit_compiler_portal_association() {
    let file = compile_file(
        r#"
        surface workspace.surface.overlay {}
        portal overlay.menu {
            surface workspace.surface.overlay
            anchor workspace.anchor
            layer transient
            dismiss escape
            focus first_enabled
            motion system_popover
        }
        component workspace.component.overlay {
            interaction activate routes workspace.intent.open opens portal overlay.menu;
        }
        "#,
    )
    .expect("file route and Portal declaration should compile");
    let rust = compile_rust(
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_surface("workspace.surface.overlay")
            .with_portal_declaration(
                "overlay.menu",
                "workspace.surface.overlay",
                "workspace.anchor",
                WorthUiPortalLayer::Transient,
                WorthUiPortalDismissalSet::from_flags(true, false, false, false)
                    .expect("Rust Portal dismissal is typed"),
                true,
                false,
                "system_popover",
            )
            .expect("Rust Portal declaration should be authored through the typed entry point")
            .with_control_routes(
                "workspace.component.overlay",
                [WorthUiIntentInteractionRoute::product(
                    WorthUiIntentInteractionFamily::Activate,
                    "workspace.intent.open",
                )
                .opens_portal("overlay.menu")],
            ),
    )
    .expect("Rust route and Portal declaration should compile");

    assert_eq!(
        first_component_route(&file),
        first_component_route(&rust),
        "file and Rust authoring must emit one explicit route association"
    );
    let file_bindings = file.overlay_declaration_bindings();
    let rust_bindings = rust.overlay_declaration_bindings();
    assert_eq!(
        file_bindings.portal_named("overlay.menu"),
        rust_bindings.portal_named("overlay.menu")
    );
    assert_eq!(
        file_bindings.surface_named("workspace.surface.overlay"),
        rust_bindings.surface_named("workspace.surface.overlay")
    );
    assert_eq!(
        first_portal_surface(&file),
        Some("workspace.surface.overlay")
    );
    assert_eq!(first_portal_surface(&file), first_portal_surface(&rust));
}

#[test]
fn rust_typed_mosaic_region_backdrop_preserves_region_truth_without_region_name_metadata() {
    let role = rust_backdrop_role();
    let backdrop = typed_region_backdrop(&role);
    let expected_identity = backdrop.identity();
    let expected_extent = backdrop.extent();
    let package = compile_rust(
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_surface("workspace.surface")
            .with_appearance_role(role)
            .with_backdrop(backdrop.clone()),
    )
    .expect("typed Rust mosaic-region backdrop should compile");
    let bindings = package.overlay_declaration_bindings();
    let surface_identity = bindings
        .surface_named("workspace.surface")
        .expect("Rust surface authoring must issue a surface identity");
    assert!(
        bindings
            .region_named("workspace.surface", "workspace.region")
            .is_none(),
        "Rust typed backdrop authoring must not invent region-name metadata"
    );

    let sealed_backdrop = package
        .backdrop_declarations()
        .next()
        .expect("typed Rust backdrop must survive sealing")
        .declaration();
    assert_eq!(sealed_backdrop, &backdrop);
    assert_eq!(sealed_backdrop.identity(), expected_identity);
    assert_eq!(sealed_backdrop.surface(), surface_identity);
    assert_eq!(sealed_backdrop.extent(), expected_extent);
    assert_eq!(
        sealed_backdrop.extent(),
        UiBackdropExtentBasis::PresentedMosaicRegion {
            surface: surface_identity,
            region: match expected_extent {
                UiBackdropExtentBasis::PresentedMosaicRegion { region, .. } => region,
                UiBackdropExtentBasis::SurfaceViewport(_) => {
                    panic!("typed Rust fixture must use a mosaic region extent")
                }
            },
        }
    );
}
