use super::super::support;
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
    session_with_second_order(role, 65_537)
}

pub(super) fn session_with_second_order(
    role: &UiAppearanceRoleDeclaration,
    second_order: u32,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    session_with_text(role, second_order, None)
}

pub(super) fn session_with_text(
    role: &UiAppearanceRoleDeclaration,
    second_order: u32,
    text: Option<crate::capability::ComponentSemanticTextContract>,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    session_with_services(role, second_order, text, false)
}

pub(super) fn session_with_motion(
    role: &UiAppearanceRoleDeclaration,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    session_with_services(role, 65_537, None, true)
}

fn session_with_services(
    role: &UiAppearanceRoleDeclaration,
    second_order: u32,
    text: Option<crate::capability::ComponentSemanticTextContract>,
    motion: bool,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let observer = host.clone();
    let requires_text_presentation = text.is_some();
    let with_services = |builder: crate::facade::entry::WorthUiApplicationBuilder| {
        if motion {
            builder.with_motion_policy_defaults(crate::declaration::UiMotionPolicy::system_respecting())
        } else {
            builder
        }
    };
    let snapshot = with_services(builder(role, second_order, text.clone()))
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("focus capabilities must prepare");
    let session = with_services(builder(role, second_order, text))
        .with_candidate_submission(source(
            snapshot.capabilities(),
            role,
            "focus-initial-source",
            motion,
        ))
        .freeze()
        .map(|application| {
            let mut application =
                crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                    application,
                    host,
                );
            if requires_text_presentation {
                let installation =
                    worth_ui_query_binding::WorthUiPresentationAsyncHostPlan::prepare()
                        .unwrap()
                        .install_for_certification()
                        .unwrap();
                application
                    .install_presentation_async(installation)
                    .unwrap();
            }
            application
        })
        .expect("focus appearance source must prepare")
        .launch()
        .expect("focus appearance owner must launch");
    (session, observer)
}

fn builder(
    role: &UiAppearanceRoleDeclaration,
    second_order: u32,
    text: Option<crate::capability::ComponentSemanticTextContract>,
) -> crate::facade::entry::WorthUiApplicationBuilder {
    let (_, _, world_profile) =
        crate::evidence::measurement::projection::fact_test_support::display_field_projection_context(
            "appearance-focus-neighborhoods",
        );
    let token = crate::capability::ThemeTokenId::new(support::LEGACY_STATIC_PAINT_TOKEN).unwrap();
    let component = |name, paint_order| {
        support::static_paint_component(name, token.clone())
            .with_surface_paint_order(paint_order)
            .with_focus(crate::capability::ComponentFocusSupport::focusable())
    };
    let mut first = component(support::APPEARANCE_NODE_A, 65_536);
    if let Some(text) = text {
        first = first
            .with_semantic_text(text)
            .with_appearance_aspect_contract(
                UiAppearanceAspectContract::component([UiAppearanceAspect::Foreground], [])
                    .unwrap(),
            )
            .unwrap();
    }
    crate::facade::WorthUi::app()
        .with_focus_policy_defaults(crate::declaration::UiFocusPolicy::workbench())
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .with_graph_world_profile(world_profile)
        .register_component(first)
        .register_component(component(support::APPEARANCE_NODE_B, second_order))
        .register_theme_token(support::appearance_theme_token(token))
        .register_mosaic_region_kind(source_backed_package_region())
        .register_mosaic_sizing_contract(source_backed_package_sizing())
        .register_appearance_role(role.clone())
        .unwrap()
        .register_appearance_role(component_role("test.focus.b", support::APPEARANCE_NODE_B))
        .unwrap()
        .register_appearance_theme_bundle(theme_bundle())
        .unwrap()
}

pub(super) fn role() -> UiAppearanceRoleDeclaration {
    component_role("test.focus.a", support::APPEARANCE_NODE_A)
}

pub(super) fn foreground_role() -> UiAppearanceRoleDeclaration {
    role_for_aspect(
        "test.focus.a",
        support::APPEARANCE_NODE_A,
        UiAppearanceAspect::Foreground,
    )
}

fn component_role(name: &str, component: &str) -> UiAppearanceRoleDeclaration {
    role_for_aspect(name, component, UiAppearanceAspect::Background)
}

fn role_for_aspect(
    name: &str,
    component: &str,
    aspect: UiAppearanceAspect,
) -> UiAppearanceRoleDeclaration {
    let mut table = UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
        UiAppearanceStateAxis::Focus,
    )]);
    for (class, label, red) in states() {
        table = table.with_cell(
            UiAppearanceCell::named(label)
                .when([UiAppearanceAxisPredicate::exact(class)])
                .uses_slot(
                    UiThemeSlotIdentity::new(format!("focus.color.c{red}")).unwrap(),
                    UiThemeValueKind::Color,
                ),
        );
    }
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new(name).unwrap(),
        UiAppearanceRoleRevision::new(1).unwrap(),
        UiAppearanceRoleApplicability::Component(UiDslComponentReference::new(component).unwrap()),
        &UiAppearanceAspectContract::component([aspect], []).unwrap(),
        [(aspect, table.compile(aspect).unwrap())],
    )
    .unwrap()
}

fn states() -> [(UiAppearanceAxisClass, &'static str, u8); 4] {
    [
        (UiAppearanceAxisClass::FocusUnfocused, "unfocused", 10),
        (UiAppearanceAxisClass::FocusFocused, "focused", 20),
        (UiAppearanceAxisClass::FocusVisible, "focus-visible", 30),
        (
            UiAppearanceAxisClass::FocusedWindowInactive,
            "window-inactive",
            40,
        ),
    ]
}

fn theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    use crate::capability::*;
    let slots =
        [10, 20, 30, 40].map(|red| ThemeTokenId::new(format!("focus.color.c{red}")).unwrap());
    let catalog = UiThemeSlotCatalog::admit(
        1,
        slots.iter().cloned().map(|slot| {
            UiThemeSlotDeclaration::new(
                slot,
                ThemeTokenFamily::surface(),
                UiThemeValueKind::Color,
                ThemeTokenSource::application(),
                UiThemeSlotDisclosure::Public,
                UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            )
        }),
    )
    .unwrap();
    let identity = UiThemeDefinitionIdentity::new("theme.focus-neighborhoods").unwrap();
    let definition = UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        slots.into_iter().zip([10, 20, 30, 40]).map(|(slot, red)| {
            (
                slot,
                UiThemeValue::Color(UiThemeColor::from_channels([red, 0, 0, 255])),
            )
        }),
    )
    .unwrap();
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition]).unwrap()
}

pub(super) fn close_source(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &UiAppearanceRoleDeclaration,
    source_name: &str,
) {
    close_source_with_services(session, role, source_name, false);
}

pub(super) fn close_source_with_motion(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &UiAppearanceRoleDeclaration,
    source_name: &str,
) {
    close_source_with_services(session, role, source_name, true);
}

fn close_source_with_services(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &UiAppearanceRoleDeclaration,
    source_name: &str,
    motion: bool,
) {
    let candidate = source(session.capabilities(), role, source_name, motion);
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    session.classify_observations(observations).unwrap();
}

fn source(
    capabilities: &crate::capability::CapabilitySnapshot,
    role: &UiAppearanceRoleDeclaration,
    source_name: &str,
    motion: bool,
) -> crate::runtime::WorthUiWatchedCandidateSubmission {
    let mut text = String::from("focus appearance.focus { scope workbench; restore; reveal; }\n");
    if motion {
        text.push_str("motion appearance.motion { reduced system_respecting }\n");
    }
    for (component, role_name) in [
        (support::APPEARANCE_NODE_A, role.role().as_str()),
        (support::APPEARANCE_NODE_B, "test.focus.b"),
    ] {
        let aspect = if component == support::APPEARANCE_NODE_A
            && role
                .partitions()
                .iter()
                .any(|(aspect, _)| *aspect == UiAppearanceAspect::Foreground)
        {
            "foreground"
        } else {
            "background"
        };
        text.push_str(&format!(
            "appearance role {role_name} applies_to {component} {{ {aspect} over [focus] {{\n"
        ));
        for (_, label, red) in states() {
            text.push_str(&format!(
                "cell {label} when focus = {label} use token(focus.color.c{red})\n"
            ));
        }
        text.push_str("} }\n");
        text.push_str(&format!(
            "component {component} {{ appearance {{ role {role_name} }} region workspace.region.primary {{ sizing workspace.sizing.mosaic_support; }} }}\n",
        ));
    }
    crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission(
        crate::runtime::WorthUiSourceProvider::in_memory(source_name)
            .with_file("appearance/consumer.wui", text),
        [crate::runtime::WorthUiWatcherEvent::provider_revision(
            source_name,
        )],
        capabilities,
    )
}

pub(super) fn publish(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    host: &crate::certification_support::ScriptedPresentationHost,
    frame: crate::mounting::UiPreparedMountedFrame,
    now: u64,
) {
    for _ in frame.surfaces() {
        if now == 1 {
            host.push_native_display_presented();
        } else {
            host.push_native_display_settled_without_effects();
        }
    }
    match session.present_prepared_mounted_frame_internal(
        frame,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(100),
        now,
    ) {
        crate::mounting::UiMountedFrameOutcome::Published(_)
        | crate::mounting::UiMountedFrameOutcome::Unchanged(_) => {}
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(denial) => {
            panic!("publication {now}: {:?}", denial.denial())
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
            panic!("publication {now}: {:?}", rejected.rejections())
        }
        other => panic!("publication {now}: {:?}", std::mem::discriminant(&other)),
    }
}
