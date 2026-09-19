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
    WorthUiSealedSemanticPackage, WorthUiSemanticArtifactDeclaration, WorthUiSemanticDeclaration,
    WorthUiServiceDeclarationMeaning, WorthUiServiceFamily,
};

const COMPONENT_NAME: &str = "platform.control.activation";
const COMPONENT_ROLE_NAME: &str = "control.appearance";
const BACKDROP_ROLE_NAME: &str = "overlay.scrim";

fn slot(value: &str) -> UiThemeSlotIdentity {
    UiThemeSlotIdentity::new(value).expect("test slot identity is valid")
}

fn component_role(
    axes: [UiAppearanceStateAxis; 3],
    reverse_cells: bool,
) -> crate::UiAppearanceRoleDeclaration {
    let base = UiAppearanceCell::named("base")
        .when([
            UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::OperabilityReady),
            UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::FocusUnfocused),
            UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::ValidationUnspecified),
        ])
        .uses_slot(slot("control.background"), UiThemeValueKind::Color);
    let focused = UiAppearanceCell::named("focused")
        .when([
            UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::OperabilityReady),
            UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::FocusFocused),
            UiAppearanceAxisPredicate::exact(UiAppearanceAxisClass::ValidationUnspecified),
        ])
        .uses_slot(slot("control.background.focused"), UiThemeValueKind::Color);
    let cells = if reverse_cells {
        [focused, base]
    } else {
        [base, focused]
    };
    UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new(COMPONENT_ROLE_NAME).unwrap())
        .applies_to_component(UiDslComponentReference::new(COMPONENT_NAME).unwrap())
        .cover(
            UiAppearanceAspect::Background,
            UiAppearancePartitionAuthoring::new(
                axes.into_iter().map(UiAppearanceAxisDomain::complete),
            )
            .with_cells(cells)
            .otherwise_same_as("base"),
        )
        .expect("the 144-cell component partition should compile")
        .build()
        .expect("the component role should be valid")
}

fn backdrop_role() -> crate::UiAppearanceRoleDeclaration {
    UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new(BACKDROP_ROLE_NAME).unwrap())
        .applies_to_backdrop()
        .cover(
            UiAppearanceAspect::Background,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([])
                    .uses_slot(slot("overlay.background"), UiThemeValueKind::Color),
            ),
        )
        .expect("backdrop background should compile")
        .cover(
            UiAppearanceAspect::Opacity,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([])
                    .uses_slot(slot("overlay.opacity"), UiThemeValueKind::Opacity),
            ),
        )
        .expect("backdrop opacity should compile")
        .build()
        .expect("the backdrop role should be valid")
}

fn typed_backdrop(role: &crate::UiAppearanceRoleDeclaration) -> UiBackdropDeclaration {
    let surface = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    UiBackdropDeclaration::admit(
        UiBackdropIdentity::new(1).unwrap(),
        surface,
        UiBackdropScope::SurfaceSingleton,
        UiBackdropExtentBasis::SurfaceViewport(surface),
        UiBackdropPresenceBasis::Always,
        UiBackdropMotionBasis::None,
        UiBackdropPlacement::ImmediatelyBeforePortal(UiPortalDeclarationId::new(1).unwrap()),
        role,
    )
    .expect("typed backdrop should pass declaration admission")
}

fn rust_portal() -> WorthUiSemanticArtifactDeclaration {
    let atoms = [
        WorthUiArtifactInputBodyAtom::Identifier("anchor".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("pulse.anchor".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("layer".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("transient".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("dismiss".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("escape".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("focus".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("first_enabled".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("motion".to_owned()),
        WorthUiArtifactInputBodyAtom::Identifier("system_popover".to_owned()),
    ];
    let service = WorthUiServiceDeclarationMeaning::parse(
        WorthUiServiceFamily::Portal,
        "pulse.modal",
        &atoms,
    )
    .expect("Rust portal service should parse");
    WorthUiSemanticArtifactDeclaration::new(
        UiDslSemanticKey::new("pulse.modal"),
        UiDslSemanticFamily::RuntimeService,
    )
    .with_service_declaration(service)
}

fn compile_file(source: &str) -> WorthUiSealedSemanticPackage {
    WorthUiDslCompiler::compile_source(
        WorthUiAuthoredSourceInput::rooted_at(PathBuf::from("workspace"))
            .with_module("app/main.wui", source),
    )
    .expect("file-authored appearance corpus should compile")
}

fn compile_rust(
    axes: [UiAppearanceStateAxis; 3],
    reverse_cells: bool,
    reverse_declarations: bool,
) -> WorthUiSealedSemanticPackage {
    let component_role = component_role(axes, reverse_cells);
    let backdrop_role = backdrop_role();
    let backdrop = typed_backdrop(&backdrop_role);
    let attachment = UiAppearanceRoleAttachmentDeclaration::new(
        UiAppearanceRoleIdentity::new(COMPONENT_ROLE_NAME).unwrap(),
        UiAppearanceRoleRevision::new(1).unwrap(),
    );
    let mut module = if reverse_declarations {
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_backdrop(backdrop)
            .with_component(COMPONENT_NAME)
            .with_appearance_role(backdrop_role.clone())
            .with_semantic_declaration(rust_portal())
            .with_surface("pulse.surface")
            .with_appearance_role(component_role)
    } else {
        WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
            .with_surface("pulse.surface")
            .with_semantic_declaration(rust_portal())
            .with_appearance_role(component_role)
            .with_appearance_role(backdrop_role.clone())
            .with_backdrop(backdrop)
            .with_component(COMPONENT_NAME)
    };
    module = module
        .with_component_appearance_role(COMPONENT_NAME, attachment)
        .expect("Rust component attachment should be unique");
    WorthUiDslCompiler::compile_rust_authored(&WorthUiRustAuthoredArtifactInput::from_modules([
        module,
    ]))
    .expect("Rust-authored appearance corpus should compile")
}

fn role<'package>(
    package: &'package WorthUiSealedSemanticPackage,
    name: &str,
) -> &'package crate::UiAppearanceRoleDeclaration {
    package
        .appearance_role_declarations()
        .find(|declaration| declaration.role().role().as_str() == name)
        .map(|declaration| declaration.role())
        .expect("corpus role should be sealed")
}

fn component_attachment(
    package: &WorthUiSealedSemanticPackage,
) -> &UiAppearanceRoleAttachmentDeclaration {
    package
        .module_ids()
        .iter()
        .flat_map(|module_id| package.declaration_views(module_id).into_iter().flatten())
        .find_map(|view| match view.declaration() {
            WorthUiSemanticDeclaration::Component(component) => {
                component.appearance_role_attachment()
            }
            _ => None,
        })
        .expect("the corpus component attachment should be sealed")
}

fn assert_complete_declaration_shape(package: &WorthUiSealedSemanticPackage) {
    let component_role = role(package, COMPONENT_ROLE_NAME);
    let background = component_role
        .partitions()
        .iter()
        .find(|(aspect, _)| *aspect == UiAppearanceAspect::Background)
        .map(|(_, partition)| partition)
        .expect("the component background partition should be present");
    assert_eq!(background.cells().len(), 144);
    assert_eq!(package.appearance_role_declarations().count(), 2);
    assert_eq!(package.backdrop_declarations().count(), 1);
    assert_eq!(
        component_attachment(package).role().as_str(),
        COMPONENT_ROLE_NAME
    );
    assert_eq!(component_attachment(package).revision().value(), 1);
    assert_eq!(
        package
            .backdrop_declarations()
            .next()
            .expect("the backdrop should be sealed")
            .declaration()
            .role()
            .as_str(),
        BACKDROP_ROLE_NAME
    );
    assert!(package.overlay_relation_graph().is_some());
}

const FILE_CORPUS: &str = r#"
appearance role control.appearance applies_to platform.control.activation {
    background over [operability, focus, validation] {
        cell base when operability = ready, focus = unfocused, validation = unspecified
            use token(control.background)
        cell focused when operability = ready, focus = focused, validation = unspecified
            use token(control.background.focused)
        otherwise same_as base
    }
}
appearance role overlay.scrim applies_to backdrop {
    background use token(overlay.background)
    opacity use token(overlay.opacity)
}
surface pulse.surface {}
portal pulse.modal {
    anchor pulse.anchor
    layer transient
    dismiss escape
    focus first_enabled
    motion system_popover
}
backdrop pulse.scrim {
    scope surface_singleton
    extent surface_viewport pulse.surface
    presence always
    motion none
    place immediately_before portal pulse.modal
    appearance { role overlay.scrim }
}
component platform.control.activation {
    appearance { role control.appearance }
    ;
}
"#;

const PERMUTED_FILE_CORPUS: &str = r#"
backdrop pulse.scrim {
    appearance { role overlay.scrim }
    place immediately_before portal pulse.modal
    motion none
    presence always
    extent surface_viewport pulse.surface
    scope surface_singleton
}
component platform.control.activation {
    appearance { role control.appearance }
    ;
}
portal pulse.modal {
    anchor pulse.anchor
    layer transient
    dismiss escape
    focus first_enabled
    motion system_popover
}
surface pulse.surface {}
appearance role overlay.scrim applies_to backdrop {
    opacity use token(overlay.opacity)
    background use token(overlay.background)
}
appearance role control.appearance applies_to platform.control.activation {
    background over [validation, operability, focus] {
        cell focused when validation = unspecified, focus = focused, operability = ready
            use token(control.background.focused)
        cell base when validation = unspecified, focus = unfocused, operability = ready
            use token(control.background)
        otherwise same_as base
    }
}
"#;

#[test]
fn file_and_rust_lowering_is_byte_identical_for_the_complete_144_cell_corpus() {
    let file = compile_file(FILE_CORPUS);
    let permuted_file = compile_file(PERMUTED_FILE_CORPUS);
    let rust = compile_rust(
        [
            UiAppearanceStateAxis::Operability,
            UiAppearanceStateAxis::Focus,
            UiAppearanceStateAxis::Validation,
        ],
        false,
        false,
    );
    let permuted_rust = compile_rust(
        [
            UiAppearanceStateAxis::Validation,
            UiAppearanceStateAxis::Operability,
            UiAppearanceStateAxis::Focus,
        ],
        true,
        true,
    );
    let packages = [&file, &permuted_file, &rust, &permuted_rust];
    for package in packages {
        assert_complete_declaration_shape(package);
    }

    let canonical_role_bytes = role(&file, COMPONENT_ROLE_NAME).canonical_bytes();
    let canonical_graph_bytes = file
        .overlay_relation_graph()
        .expect("the file overlay graph should be sealed")
        .canonical_bytes();
    for package in packages {
        assert_eq!(
            role(package, COMPONENT_ROLE_NAME).canonical_bytes(),
            canonical_role_bytes
        );
        assert_eq!(
            package
                .overlay_relation_graph()
                .expect("the overlay graph should be sealed")
                .canonical_bytes(),
            canonical_graph_bytes
        );
        assert_eq!(package.identity(), file.identity());
    }
}
