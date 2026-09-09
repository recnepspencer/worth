use super::super::{support, test_support};
use crate::runtime::tests::source_ingress_boundary_test_support::{
    source_backed_package_region, source_backed_package_sizing,
};
use worth_ui_dsl::*;

pub(super) fn session(
    role: &UiAppearanceRoleDeclaration,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    session_with_declarations(role, support::appearance_fixture(role))
}

pub(super) fn session_with_declarations(
    role: &UiAppearanceRoleDeclaration,
    declarations: crate::facade::WorthUiRustAuthoredDeclarationFixture,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::staged_appearance_capability_report());
    let observer = host.clone();
    let (_, _, world_profile) =
        crate::evidence::measurement::projection::fact_test_support::display_field_projection_context(
            "appearance-pointer-mounting",
        );
    let token = crate::capability::ThemeTokenId::new(support::LEGACY_STATIC_PAINT_TOKEN).unwrap();
    let component = |name| {
        let allocation = crate::capability::ComponentAllocationMeasurementContract::viewport_region(
            crate::capability::ComponentViewportRegion::new(
                crate::capability::ComponentViewportAxisPlacement::fixed_from_start(
                    if name == support::APPEARANCE_NODE_A {
                        24
                    } else {
                        200
                    },
                    100,
                )
                .unwrap(),
                crate::capability::ComponentViewportAxisPlacement::fixed_from_start(24, 60)
                    .unwrap(),
            ),
        );
        support::static_paint_component_with_allocation(name, token.clone(), allocation)
            .with_surface_paint_order(65_536)
            .with_appearance_aspect_contract(role.aspect_contract().clone())
            .unwrap()
            .with_hit_test(
                crate::capability::ComponentHitTestContract::allocation_bounds(
                    crate::capability::ComponentHitTestOrder::front_to_back(u32::from(
                        name != support::APPEARANCE_NODE_A,
                    )),
                    allocation,
                ),
            )
    };
    let session = crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(world_profile)
        .register_component(component(support::APPEARANCE_NODE_A))
        .register_component(component(support::APPEARANCE_NODE_B))
        .register_theme_token(support::appearance_theme_token(token))
        .register_mosaic_region_kind(source_backed_package_region())
        .register_mosaic_sizing_contract(source_backed_package_sizing())
        .register_appearance_role(role.clone())
        .unwrap()
        .register_appearance_theme_bundle(test_support::theme_bundle())
        .unwrap()
        .with_rust_authored_declaration_fixture(declarations)
        .freeze()
        .map(|application| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                application,
                host,
            )
        })
        .expect("pointer appearance source must prepare")
        .launch()
        .expect("pointer appearance owner must launch");
    (session, observer)
}

pub(super) fn role() -> UiAppearanceRoleDeclaration {
    let mut table = UiAppearancePartitionAuthoring::new([
        UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Hover),
        UiAppearanceAxisDomain::complete(UiAppearanceStateAxis::Pressed),
    ]);
    for (hover, pressed, red) in [
        (
            UiAppearanceAxisClass::HoverOutside,
            UiAppearanceAxisClass::PressedIdle,
            10,
        ),
        (
            UiAppearanceAxisClass::Hovered,
            UiAppearanceAxisClass::PressedIdle,
            20,
        ),
        (
            UiAppearanceAxisClass::HoverOutside,
            UiAppearanceAxisClass::PressedArmedInside,
            30,
        ),
        (
            UiAppearanceAxisClass::Hovered,
            UiAppearanceAxisClass::PressedArmedInside,
            30,
        ),
        (
            UiAppearanceAxisClass::HoverOutside,
            UiAppearanceAxisClass::PressedCapturedOutside,
            40,
        ),
        (
            UiAppearanceAxisClass::Hovered,
            UiAppearanceAxisClass::PressedCapturedOutside,
            40,
        ),
    ] {
        table = table.with_cell(
            UiAppearanceCell::when([
                UiAppearanceAxisPredicate::exact(hover),
                UiAppearanceAxisPredicate::exact(pressed),
            ])
            .literal(UiThemeValue::Color(UiThemeColor::from_channels([
                red, 0, 0, 255,
            ]))),
        );
    }
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new("test.pointer-mounted-background").unwrap(),
        UiAppearanceRoleRevision::new(1).unwrap(),
        UiAppearanceRoleApplicability::AnyComponent,
        &UiAppearanceAspectContract::component([UiAppearanceAspect::Background], []).unwrap(),
        [(
            UiAppearanceAspect::Background,
            table.compile(UiAppearanceAspect::Background).unwrap(),
        )],
    )
    .unwrap()
}
