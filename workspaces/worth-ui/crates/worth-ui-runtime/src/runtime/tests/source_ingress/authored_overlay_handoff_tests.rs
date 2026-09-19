use crate::capability::{
    ComponentChildPolicy, ComponentDescriptor, ComponentId, ComponentPropSchema,
    ComponentStateOwnership, SurfaceDescriptor, SurfaceId, SurfaceKind, SurfacePlacementClass,
    SurfaceStateClass,
};
use crate::facade::WorthUi;
use crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission;
use crate::runtime::{WorthUiSourceProvider, WorthUiWatcherEvent};

const OVERLAY_SOURCE: &str = r#"
surface workspace.surface.overlay {}
portal overlay.menu {
    anchor workspace.anchor
    layer transient
    dismiss escape
    focus first_enabled
    motion system_popover
}
appearance role overlay.scrim applies_to backdrop {
    background use token(overlay.scrim.background)
    opacity use token(overlay.scrim.opacity)
}
backdrop overlay.scrim {
    scope surface_singleton
    extent surface_viewport workspace.surface.overlay
    presence always
    motion none
    place immediately_before portal overlay.menu
    appearance { role overlay.scrim }
}
"#;

fn overlay_builder() -> crate::facade::entry::WorthUiCertificationApplicationBuilder {
    WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .register_appearance_role(backdrop_role())
        .expect("the authored backdrop role is valid")
        .register_appearance_theme_bundle(theme_bundle())
        .expect("the authored backdrop theme is valid")
        .register_component(ComponentDescriptor::new(
            ComponentId::new("workspace.component.overlay").expect("valid component identity"),
            ComponentPropSchema::named("workspace.component.overlay.props"),
            ComponentChildPolicy::no_children(),
            ComponentStateOwnership::runtime_owned(),
        ))
        .register_surface(SurfaceDescriptor::new(
            SurfaceId::new("workspace.surface.overlay").expect("valid surface identity"),
            SurfaceKind::overlay_content(),
            ComponentId::new("workspace.component.overlay").expect("valid component identity"),
            SurfacePlacementClass::overlay_layer(),
            SurfaceStateClass::restorable(),
        ))
}

fn backdrop_role() -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    use worth_ui_dsl::*;
    let partition = |identity, kind| {
        UiAppearancePartitionAuthoring::new([]).with_cell(
            UiAppearanceCell::when([]).uses_slot(UiThemeSlotIdentity::new(identity).unwrap(), kind),
        )
    };
    UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new("overlay.scrim").unwrap())
        .applies_to_backdrop()
        .cover(
            UiAppearanceAspect::Background,
            partition("overlay.scrim.background", UiThemeValueKind::Color),
        )
        .unwrap()
        .cover(
            UiAppearanceAspect::Opacity,
            partition("overlay.scrim.opacity", UiThemeValueKind::Opacity),
        )
        .unwrap()
        .build()
        .unwrap()
}

fn theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    use crate::capability::*;
    use worth_ui_dsl::{UiThemeColor, UiThemeOpacity, UiThemeValue, UiThemeValueKind};
    let declarations = [
        ("overlay.scrim.background", UiThemeValueKind::Color),
        ("overlay.scrim.opacity", UiThemeValueKind::Opacity),
    ];
    let catalog = UiThemeSlotCatalog::admit(
        1,
        declarations.map(|(identity, kind)| {
            UiThemeSlotDeclaration::new(
                ThemeTokenId::new(identity).unwrap(),
                ThemeTokenFamily::surface(),
                kind,
                ThemeTokenSource::application(),
                UiThemeSlotDisclosure::Public,
                UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            )
        }),
    )
    .unwrap();
    let identity = UiThemeDefinitionIdentity::new("theme.overlay.handoff").unwrap();
    let definition = UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        [
            (
                ThemeTokenId::new("overlay.scrim.background").unwrap(),
                UiThemeValue::Color(UiThemeColor::from_channels([4, 8, 12, 255])),
            ),
            (
                ThemeTokenId::new("overlay.scrim.opacity").unwrap(),
                UiThemeValue::Opacity(UiThemeOpacity::from_ratio(1, 2).unwrap()),
            ),
        ],
    )
    .unwrap();
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition]).unwrap()
}

#[test]
fn file_source_overlay_truth_reaches_prepared_authority_without_reconstruction() {
    let capability_app = overlay_builder()
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("overlay capability snapshot should prepare");
    let submission = lower_file_submission(
        WorthUiSourceProvider::in_memory("overlay-source")
            .with_file("app/main.wui", OVERLAY_SOURCE),
        [WorthUiWatcherEvent::provider_revision("overlay-source")],
        capability_app.capabilities(),
    );
    let expected_handoff = submission.composition_basis().semantic_handoff().clone();
    let expected_material = expected_handoff.authored_overlay_material().clone();

    let prepared = overlay_builder()
        .with_candidate_submission(submission)
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("the source candidate should prepare");
    let authority = prepared.prepared_authority();
    let material = authority.authored_overlay_material();

    assert_eq!(authority.semantic_handoff(), &expected_handoff);
    assert_eq!(material, &expected_material);
    let portal_id = material
        .overlay_declaration_bindings()
        .portal_named("overlay.menu")
        .expect("compiler should issue the portal identity");
    assert_eq!(material.portal_anchor_bindings().len(), 1);
    let portal_binding = &material.portal_anchor_bindings()[0];
    assert_eq!(portal_binding.portal_declaration_id(), portal_id);
    assert_eq!(portal_binding.portal().identity(), "overlay.menu");
    assert_eq!(portal_binding.anchor(), "workspace.anchor");
    assert_eq!(portal_binding.provenance().module_path(), "app/main.wui");

    assert_eq!(material.backdrop_declarations().len(), 1);
    let backdrop = material.backdrop_declarations()[0]
        .declaration()
        .declaration();
    assert_eq!(
        backdrop.identity(),
        material
            .overlay_declaration_bindings()
            .backdrop_named("overlay.scrim")
            .expect("compiler should issue the backdrop identity")
    );
    assert_eq!(
        backdrop.surface(),
        material
            .overlay_declaration_bindings()
            .surface_named("workspace.surface.overlay")
            .expect("compiler should issue the surface identity")
    );
    assert_eq!(
        material.backdrop_declarations()[0]
            .provenance()
            .module_path(),
        "app/main.wui"
    );
    assert!(material.overlay_relation_graph().is_some());
}
