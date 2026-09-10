use super::test_support::authored_overlay_builder_with_appearance_contract_and_region;
use crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission;
use crate::runtime::{WorthUiSourceProvider, WorthUiWatcherEvent};

pub(super) const SOURCE: &str = r#"
surface workspace.surface.overlay {}
surface workspace.surface.secondary {}
appearance role overlay.content applies_to workspace.component.overlay {
    background use token(overlay.content.background)
}
appearance role overlay.scrim applies_to backdrop {
    background use token(overlay.scrim.background)
    opacity use token(overlay.scrim.opacity)
}
portal overlay.menu {
    surface workspace.surface.overlay
    anchor workspace.anchor
    layer transient
    dismiss escape
    focus first_enabled
    motion system_popover
}
portal overlay.secondary {
    surface workspace.surface.secondary
    anchor workspace.anchor
    layer transient
    dismiss escape
    focus first_enabled
    motion system_popover
}
backdrop overlay.scrim {
    scope surface_singleton
    extent surface_viewport workspace.surface.overlay
    presence while portal overlay.menu presented
    motion none
    place immediately_before portal overlay.menu
    appearance { role overlay.scrim }
}
component workspace.component.overlay {
    appearance { role overlay.content }
    region workspace.region.primary {
        sizing workspace.sizing.mosaic_support;
    }
    interaction activate routes workspace.intent.open opens portal overlay.menu;
}
intent workspace.intent.open {
    definition workspace.intent.open
    interaction activate
    operability workspace.intent.open.operability
        mutability-application-boolean workspace.intent.open.mutable
        readiness-application-boolean workspace.intent.open.ready
        policy-application-boolean workspace.intent.open.policy
    confirmation workspace.intent.open.confirmation not-required
    concurrency target-route-single-flight
    consequences mounted-posture
}
"#;

pub(super) fn close_source(session: &mut crate::facade::WorthUiActiveApplicationSession) {
    close_source_with(session, SOURCE);
}

pub(super) fn close_source_with(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    source: &str,
) {
    let candidate = lower_file_submission(
        WorthUiSourceProvider::in_memory("overlay-appearance-runtime")
            .with_file("app/main.wui", source),
        [WorthUiWatcherEvent::provider_revision(
            "overlay-appearance-runtime",
        )],
        session.capabilities(),
    );
    let mut turn = session.begin_observation_turn().unwrap();
    turn.admit_source(candidate).unwrap();
    let observations = turn.seal().unwrap();
    session.classify_observations(observations).unwrap();
}

pub(super) fn appearance_overlay_session() -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    appearance_overlay_session_with_source(SOURCE)
}

pub(super) fn appearance_overlay_session_with_source(
    source: &str,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    appearance_overlay_session_with_component_role(source, component_role())
}

pub(super) fn appearance_overlay_session_with_component_role(
    source: &str,
    role: worth_ui_dsl::UiAppearanceRoleDeclaration,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    appearance_overlay_session_with_component_role_and_region(
        source,
        role,
        crate::runtime::tests::source_ingress_boundary_test_support::source_backed_package_region(),
    )
}

pub(super) fn appearance_overlay_session_with_region_clipping(
    source: &str,
    clipping: crate::capability::MosaicClippingPosture,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    let region =
        crate::runtime::tests::source_ingress_boundary_test_support::source_backed_package_region()
            .with_clipping(clipping)
            .with_scroll_ownership(crate::capability::MosaicScrollOwnership::no_scrolling());
    appearance_overlay_session_with_component_role_and_region(source, component_role(), region)
}

pub(super) fn appearance_overlay_session_with_region(
    region: crate::capability::MosaicRegionKindDescriptor,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    appearance_overlay_session_with_component_role_and_region(SOURCE, component_role(), region)
}

fn appearance_overlay_session_with_component_role_and_region(
    source: &str,
    role: worth_ui_dsl::UiAppearanceRoleDeclaration,
    region: crate::capability::MosaicRegionKindDescriptor,
) -> (
    crate::facade::WorthUiActiveApplicationSession,
    crate::certification_support::ScriptedPresentationHost,
) {
    let builder = || {
        authored_overlay_builder_with_appearance_contract_and_region(
            role.aspect_contract().clone(),
            region.clone(),
        )
        .register_surface(crate::capability::SurfaceDescriptor::new(
            crate::capability::SurfaceId::new("workspace.surface.secondary").unwrap(),
            crate::capability::SurfaceKind::overlay_content(),
            crate::capability::ComponentId::new("workspace.component.overlay").unwrap(),
            crate::capability::SurfacePlacementClass::overlay_layer(),
            crate::capability::SurfaceStateClass::restorable(),
        ))
        .register_appearance_role(role.clone())
        .unwrap()
        .register_appearance_role(backdrop_role())
        .unwrap()
        .register_appearance_theme_bundle(theme_bundle())
        .unwrap()
    };
    let capability_app = builder()
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("overlay appearance capabilities should prepare");
    let submission = lower_file_submission(
        WorthUiSourceProvider::in_memory("overlay-appearance-runtime")
            .with_file("app/main.wui", source),
        [WorthUiWatcherEvent::provider_revision(
            "overlay-appearance-runtime",
        )],
        capability_app.capabilities(),
    );
    let host = crate::certification_support::ScriptedPresentationHost::native_display();
    host.set_capabilities(worth_ui_host_native::appearance_capability_report());
    let observer = host.clone();
    let session = builder()
        .with_candidate_submission(submission)
        .freeze()
        .map(|app| {
            crate::facade::entry::WorthUiCertificationApplicationTransition::activate_test_host(
                app, host,
            )
        })
        .expect("overlay appearance source should prepare")
        .launch()
        .expect("overlay appearance source should launch");
    (session, observer)
}

fn component_role() -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    use worth_ui_dsl::*;
    UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new("overlay.content").unwrap())
        .applies_to_component(UiDslComponentReference::new("workspace.component.overlay").unwrap())
        .cover(
            UiAppearanceAspect::Background,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([]).uses_slot(
                    UiThemeSlotIdentity::new("overlay.content.background").unwrap(),
                    UiThemeValueKind::Color,
                ),
            ),
        )
        .unwrap()
        .build()
        .unwrap()
}

pub(super) fn backdrop_role() -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    use worth_ui_dsl::*;
    let slot = |identity, kind| {
        UiAppearancePartitionAuthoring::new([]).with_cell(
            UiAppearanceCell::when([]).uses_slot(UiThemeSlotIdentity::new(identity).unwrap(), kind),
        )
    };
    UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new("overlay.scrim").unwrap())
        .applies_to_backdrop()
        .cover(
            UiAppearanceAspect::Background,
            slot("overlay.scrim.background", UiThemeValueKind::Color),
        )
        .unwrap()
        .cover(
            UiAppearanceAspect::Opacity,
            slot("overlay.scrim.opacity", UiThemeValueKind::Opacity),
        )
        .unwrap()
        .build()
        .unwrap()
}

pub(super) fn establish_geometry(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) {
    establish_allocation(session);
    crate::facade::entry::mounted_occurrence_geometry_test_support::
        install_nonoverlapping_surface_geometry(session, surface, 1, &[]);
}

pub(super) fn establish_allocation(session: &mut crate::facade::WorthUiActiveApplicationSession) {
    let assumptions = crate::host::UiHostMeasurementAssumptionProfile::from_capability_report(
        session.host_measurement_capability().capability_report(),
        1,
        2,
        3,
        4,
    );
    let request = crate::facade::entry::UiMountedAllocationMeasurementRequest::new(
        worth_ui_host_contract::UiMeasurementEvidenceFamily::ViewportExtent,
        crate::host::UiHostMeasurementNeed::ViewportExtent(
            worth_ui_host_contract::UiViewportExtentRequest,
        ),
        crate::host::UiHostMeasurementNormalizationContext::viewport_logical_exact(assumptions),
    );
    session
        .establish_mounted_allocation_catalog(1, [request])
        .unwrap();
}

pub(super) fn expected_overlay_color() -> [u8; 4] {
    let to_linear = |channel: u8| {
        let value = f64::from(channel) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    let to_srgb = |linear: f64| {
        let value = if linear <= 0.0031308 {
            linear * 12.92
        } else {
            1.055 * linear.powf(1.0 / 2.4) - 0.055
        };
        (value * 255.0).round() as u8
    };
    let lower_alpha = 32_768.0 / 65_535.0;
    let upper_alpha = 128.0 / 255.0;
    let alpha = upper_alpha + lower_alpha * (1.0 - upper_alpha);
    let mut result = [0; 4];
    for (index, (lower, upper)) in [4, 8, 12].into_iter().zip([32, 64, 96]).enumerate() {
        let premultiplied =
            to_linear(upper) * upper_alpha + to_linear(lower) * lower_alpha * (1.0 - upper_alpha);
        result[index] = to_srgb(premultiplied / alpha);
    }
    result[3] = (alpha * 255.0).round() as u8;
    result
}

pub(super) fn open_portal(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    graph: crate::graph::UiGraphNodeIdentity,
    mounted: worth_ui_host_contract::UiMountedInstanceIdentity,
    declaration: worth_ui_dsl::UiPortalDeclarationId,
    sequence: u64,
) -> crate::runtime::portal::UiPortalIdentity {
    let portal = crate::runtime::portal::UiPortalIdentity::for_owner(
        crate::runtime::portal::UiPortalOwnerIdentity::from_mounted_owner(graph, mounted),
    );
    let geometry = crate::runtime::interaction::UiPresentedInteractionGeometry::for_test(
        session
            .mounted
            .current_presentation_for_surface(surface)
            .unwrap(),
    );
    let request = crate::runtime::portal::UiPortalServiceRequest::open(
        portal,
        crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
            session.session_identity().as_u64(),
            sequence,
        ),
        geometry,
        Some(
            crate::runtime::interaction::UiPresentedViewportGeometry::for_test(
                geometry.clip_bounds(),
                geometry.presentation(),
            ),
        ),
        surface,
    )
    .with_declared_portal(Some(declaration));
    let stage = session
        .admit_authored_portal_open(declaration, portal, surface)
        .unwrap();
    let transition = session.portal.as_ref().unwrap().prepare(request).unwrap();
    let binding = crate::runtime::portal::UiPortalOverlayBindingCommit::from_transition(
        &transition,
        Some(stage),
    );
    session
        .portal
        .as_mut()
        .unwrap()
        .commit_published(transition)
        .unwrap();
    session.commit_authored_overlay_binding(binding).unwrap();
    portal
}

fn theme_bundle() -> crate::capability::FrozenAppearanceThemeCapabilities {
    use crate::capability::*;
    let slots = [
        (
            "overlay.content.background",
            worth_ui_dsl::UiThemeValueKind::Color,
        ),
        (
            "overlay.scrim.background",
            worth_ui_dsl::UiThemeValueKind::Color,
        ),
        (
            "overlay.scrim.opacity",
            worth_ui_dsl::UiThemeValueKind::Opacity,
        ),
    ];
    let catalog = UiThemeSlotCatalog::admit(
        1,
        slots.iter().map(|(identity, kind)| {
            UiThemeSlotDeclaration::new(
                ThemeTokenId::new(*identity).unwrap(),
                ThemeTokenFamily::surface(),
                *kind,
                ThemeTokenSource::application(),
                UiThemeSlotDisclosure::Public,
                UiThemeSlotSuccessorCompatibility::ExactMeaning,
                None,
            )
        }),
    )
    .unwrap();
    let identity = UiThemeDefinitionIdentity::new("theme.overlay.appearance").unwrap();
    let definition = UiThemeDefinition::admit(
        identity.clone(),
        1,
        &catalog,
        [
            (
                ThemeTokenId::new("overlay.content.background").unwrap(),
                worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                    32, 64, 96, 128,
                ])),
            ),
            (
                ThemeTokenId::new("overlay.scrim.background").unwrap(),
                worth_ui_dsl::UiThemeValue::Color(worth_ui_dsl::UiThemeColor::from_channels([
                    4, 8, 12, 255,
                ])),
            ),
            (
                ThemeTokenId::new("overlay.scrim.opacity").unwrap(),
                worth_ui_dsl::UiThemeValue::Opacity(
                    worth_ui_dsl::UiThemeOpacity::from_ratio(1, 2).unwrap(),
                ),
            ),
        ],
    )
    .unwrap();
    FrozenAppearanceThemeCapabilities::admit(catalog, identity, vec![definition]).unwrap()
}
