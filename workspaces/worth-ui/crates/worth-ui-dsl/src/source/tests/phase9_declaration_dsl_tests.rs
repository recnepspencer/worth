use std::path::PathBuf;

use crate::{
    UiAppearanceAspect, UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceAxisPredicate,
    UiAppearanceCell, UiAppearancePartitionAuthoring, UiAppearanceRole,
    UiAppearanceRoleAttachmentDeclaration, UiAppearanceRoleIdentity, UiAppearanceRoleRevision,
    UiAppearanceStateAxis, UiBackdropDeclarationAuthoring, UiDslComponentReference,
    UiStaticBackdropExtent, UiStaticBackdropMotion, UiStaticBackdropPlacement,
    UiStaticBackdropPresence, UiStaticBackdropScope, UiThemeSlotIdentity, UiThemeValueKind,
    WorthUiAuthoredSourceInput, WorthUiDslCompileDiagnosticCode, WorthUiDslCompiler,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredArtifactInputModule,
    WorthUiSealedSemanticPackage,
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

fn backdrop_spec(
    identity: &str,
    placement: UiStaticBackdropPlacement,
) -> crate::UiStaticBackdropDeclaration {
    UiBackdropDeclarationAuthoring::new(
        identity,
        "pulse.confirmation.surface",
        UiAppearanceRoleIdentity::new("overlay.scrim").unwrap(),
        UiAppearanceRoleRevision::new(1).unwrap(),
    )
    .expect("backdrop identity is valid")
    .with_scope(UiStaticBackdropScope::SurfaceSingleton)
    .with_extent(UiStaticBackdropExtent::SurfaceViewport(
        "pulse.confirmation.surface".into(),
    ))
    .with_presence(UiStaticBackdropPresence::Always)
    .with_motion(UiStaticBackdropMotion::None)
    .with_placement(placement)
    .admit()
    .expect("backdrop specification is valid")
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

fn first_code(report: &crate::WorthUiDslCompileReport) -> WorthUiDslCompileDiagnosticCode {
    report.diagnostics()[0].identity().code()
}

#[test]
fn rust_and_file_declarations_share_role_bytes_and_attachment_meaning() {
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
    let role = component_role();
    let rust = compile_rust(
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_appearance_role(role.clone())
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

    let file_role = file.appearance_role_declarations().next().unwrap().role();
    let rust_role = rust.appearance_role_declarations().next().unwrap().role();
    assert_eq!(file_role.canonical_bytes(), role.canonical_bytes());
    assert_eq!(rust_role.canonical_bytes(), role.canonical_bytes());
    assert_eq!(file.identity(), rust.identity());
    let file_receipt = file
        .declaration_lowering_receipts()
        .into_iter()
        .find(|receipt| {
            receipt
                .semantic_artifact()
                .appearance_role_attachment()
                .is_some()
        })
        .expect("file attachment should reach the runtime lowering receipt");
    assert_eq!(
        file_receipt
            .semantic_artifact()
            .appearance_role_attachment()
            .unwrap()
            .role()
            .as_str(),
        "action.primary"
    );
    let attachment = file
        .module(&file.module_ids()[0])
        .unwrap()
        .declarations()
        .iter()
        .find_map(|declaration| match declaration {
            crate::WorthUiSemanticDeclaration::Component(component) => {
                component.appearance_role_attachment()
            }
            _ => None,
        })
        .expect("file component should carry an explicit role attachment");
    assert_eq!(attachment.role().as_str(), "action.primary");
}
#[test]
fn rust_and_file_backdrop_declarations_share_exact_meaning() {
    let file = compile_file(
        r#"
        appearance role overlay.scrim applies_to backdrop {
            background use token(overlay.scrim.background)
            opacity use token(overlay.scrim.opacity)
        }
        backdrop pulse.confirmation.scrim {
            scope surface_singleton
            extent surface_viewport pulse.confirmation.surface
            presence always
            motion none
            place above_surface_content
            appearance { role overlay.scrim }
        }
        "#,
    )
    .expect("file backdrop declaration should compile");
    let rust = compile_rust(
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_appearance_role(backdrop_role())
            .with_backdrop(backdrop_spec(
                "pulse.confirmation.scrim",
                UiStaticBackdropPlacement::AboveSurfaceContent,
            )),
    )
    .expect("Rust backdrop declaration should compile");

    assert_eq!(file.identity(), rust.identity());
    assert_eq!(
        file.backdrop_declarations()
            .next()
            .unwrap()
            .declaration()
            .identity(),
        "pulse.confirmation.scrim"
    );
    let file_backdrop = file.backdrop_declarations().next().unwrap().declaration();
    let rust_backdrop = rust.backdrop_declarations().next().unwrap().declaration();
    assert_eq!(
        file_backdrop.canonical_bytes(),
        rust_backdrop.canonical_bytes()
    );
    assert_eq!(
        file.overlay_relation_graph().unwrap().canonical_bytes(),
        rust.overlay_relation_graph().unwrap().canonical_bytes()
    );
}

#[test]
fn source_partition_diagnostics_are_typed_and_source_linked() {
    let overlapping = compile_file(
        r#"appearance role bad applies_to button {
            background over [hover] {
                cell first when hover = outside use token(first)
                cell second when hover = outside use token(second)
            }
        }"#,
    )
    .expect_err("overlap must be denied");
    assert_eq!(
        first_code(&overlapping),
        WorthUiDslCompileDiagnosticCode::OverlappingAppearanceCells
    );
    assert!(overlapping.diagnostics()[0].identity().span().is_some());

    let missing = compile_file(
        r#"appearance role incomplete applies_to button {
            background over [hover] {
                cell outside when hover = outside use token(button.background)
            }
        }"#,
    )
    .expect_err("a hole must be denied");
    assert_eq!(
        first_code(&missing),
        WorthUiDslCompileDiagnosticCode::MissingAppearanceDeclaration
    );

    let wrong_kind = compile_file(
        r#"appearance role wrong-kind applies_to button {
            background use transparent-outline
        }"#,
    )
    .expect_err("a transparent outline cannot fill a background aspect");
    assert_eq!(
        first_code(&wrong_kind),
        WorthUiDslCompileDiagnosticCode::WrongAppearanceValueKind
    );

    let cyclic = compile_file(
        r#"appearance role cyclic applies_to button {
            background over [hover] {
                cell first when hover = outside use same_as(second)
                cell second when hover = hovered use same_as(first)
            }
        }"#,
    )
    .expect_err("cyclic cell references must be denied");
    assert_eq!(
        first_code(&cyclic),
        WorthUiDslCompileDiagnosticCode::CyclicAppearanceCellReference
    );

    let capacity = compile_file(
        r#"appearance role saturated applies_to button {
            background over [operability, focus, validation, selection] {
                otherwise use transparent-color
            }
        }"#,
    )
    .expect_err("expanded cell capacity must be denied");
    assert_eq!(
        first_code(&capacity),
        WorthUiDslCompileDiagnosticCode::AppearanceCapacityDenied
    );
}

#[test]
fn attachment_and_overlay_relation_diagnostics_are_typed() {
    let missing_role =
        compile_file("component platform.control.activation { appearance { role missing.role } }")
            .expect_err("missing attachment role must be denied");
    assert_eq!(
        first_code(&missing_role),
        WorthUiDslCompileDiagnosticCode::MissingAppearanceDeclaration
    );

    let wrong_kind = compile_file(
        r#"
        appearance role overlay.scrim applies_to backdrop {
            background use token(overlay.scrim.background)
            opacity use token(overlay.scrim.opacity)
        }
        component platform.control.activation { appearance { role overlay.scrim } }
        "#,
    )
    .expect_err("backdrop role cannot attach to a component");
    assert_eq!(
        first_code(&wrong_kind),
        WorthUiDslCompileDiagnosticCode::WrongAppearanceDeclarationKind
    );

    let missing_anchor = compile_file(
        r#"
        appearance role overlay.scrim applies_to backdrop {
            background use token(overlay.scrim.background)
            opacity use token(overlay.scrim.opacity)
        }
        backdrop pulse.scrim {
            scope per_portal_instance missing.portal
            extent surface_viewport pulse.surface
            presence while portal missing.portal presented
            motion follow portal missing.portal presentation
            place immediately_before portal missing.portal
            appearance { role overlay.scrim }
        }
        "#,
    )
    .expect_err("missing portal anchor must be denied");
    assert_eq!(
        first_code(&missing_anchor),
        WorthUiDslCompileDiagnosticCode::MissingOverlayAnchor
    );

    let ambiguous = compile_file(
        r#"
        appearance role overlay.scrim applies_to backdrop {
            background use token(overlay.scrim.background)
            opacity use token(overlay.scrim.opacity)
        }
        backdrop first {
            scope surface_singleton
            extent surface_viewport pulse.surface
            presence always
            motion none
            place above_surface_content
            appearance { role overlay.scrim }
        }
        backdrop second {
            scope surface_singleton
            extent surface_viewport pulse.surface
            presence always
            motion none
            place above_surface_content
            appearance { role overlay.scrim }
        }
        "#,
    )
    .expect_err("unrelated overlay rows must be denied as ambiguous");
    assert_eq!(
        first_code(&ambiguous),
        WorthUiDslCompileDiagnosticCode::AmbiguousOverlayRelation
    );
}
