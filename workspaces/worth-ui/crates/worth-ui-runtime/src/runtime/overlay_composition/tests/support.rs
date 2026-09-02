use super::super::*;

pub(crate) fn role() -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    let contract = worth_ui_dsl::UiAppearanceAspectContract::backdrop();
    let partition = |aspect: worth_ui_dsl::UiAppearanceAspect| {
        worth_ui_dsl::UiAppearanceDecisionPartition::compile(
            [],
            [worth_ui_dsl::UiAppearanceDecisionRule::new(
                [],
                worth_ui_dsl::UiAppearanceDecisionResult::theme_slot(
                    worth_ui_dsl::UiThemeSlotIdentity::new(format!("test.backdrop.{aspect:?}"))
                        .unwrap(),
                    aspect.value_kind(),
                ),
            )],
        )
        .unwrap()
    };
    worth_ui_dsl::UiAppearanceRoleDeclaration::admit(
        worth_ui_dsl::UiAppearanceRoleIdentity::new("test.overlay.backdrop").unwrap(),
        worth_ui_dsl::UiAppearanceRoleRevision::new(1).unwrap(),
        worth_ui_dsl::UiAppearanceRoleApplicability::Backdrop,
        &contract,
        [
            (
                worth_ui_dsl::UiAppearanceAspect::Background,
                partition(worth_ui_dsl::UiAppearanceAspect::Background),
            ),
            (
                worth_ui_dsl::UiAppearanceAspect::Opacity,
                partition(worth_ui_dsl::UiAppearanceAspect::Opacity),
            ),
        ],
    )
    .unwrap()
}

pub(crate) fn declaration(
    identity: u64,
    surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    scope: worth_ui_dsl::UiBackdropScope,
    presence: worth_ui_dsl::UiBackdropPresenceBasis,
    motion: worth_ui_dsl::UiBackdropMotionBasis,
    placement: worth_ui_dsl::UiBackdropPlacement,
) -> worth_ui_dsl::UiBackdropDeclaration {
    declaration_with_extent(
        identity,
        surface,
        scope,
        worth_ui_dsl::UiBackdropExtentBasis::SurfaceViewport(surface),
        presence,
        motion,
        placement,
    )
}

pub(crate) fn declaration_with_extent(
    identity: u64,
    surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    scope: worth_ui_dsl::UiBackdropScope,
    extent: worth_ui_dsl::UiBackdropExtentBasis,
    presence: worth_ui_dsl::UiBackdropPresenceBasis,
    motion: worth_ui_dsl::UiBackdropMotionBasis,
    placement: worth_ui_dsl::UiBackdropPlacement,
) -> worth_ui_dsl::UiBackdropDeclaration {
    worth_ui_dsl::UiBackdropDeclaration::admit(
        worth_ui_dsl::UiBackdropIdentity::new(identity).unwrap(),
        surface,
        scope,
        extent,
        presence,
        motion,
        placement,
        &role(),
    )
    .unwrap()
}

pub(crate) fn box_at(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> worth_ui_host_contract::UiMountedCanonicalBox {
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface,
        },
    )
    .unwrap()
}

pub(crate) fn surface_extent(
    surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    revision: u64,
) -> UiOverlaySurfaceExtentSnapshot {
    surface_extent_with_viewport(
        surface,
        runtime_surface,
        revision,
        box_at(0.0, 0.0, 800.0, 600.0),
        [],
    )
}

pub(crate) fn surface_extent_with_viewport(
    surface: worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity,
    runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    revision: u64,
    viewport: worth_ui_host_contract::UiMountedCanonicalBox,
    regions: impl IntoIterator<Item = UiOverlayRegionExtent>,
) -> UiOverlaySurfaceExtentSnapshot {
    UiOverlaySurfaceExtentSnapshot::seal(surface, runtime_surface, revision, viewport, regions)
        .unwrap()
}

pub(crate) fn portal(graph: u64) -> crate::runtime::portal::UiPortalIdentity {
    crate::runtime::portal::UiPortalIdentity::for_test(graph)
}

pub(crate) fn portal_snapshot(
    runtime_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    rows: impl IntoIterator<Item = (crate::runtime::portal::UiPortalIdentity, u64)>,
) -> crate::runtime::portal::UiPortalStackSnapshot {
    portal_snapshot_with_rows(
        7,
        rows.into_iter().map(|(portal, ordinal)| {
            (
                portal,
                None,
                runtime_surface,
                ordinal,
                crate::runtime::portal::UiPortalLifecyclePosture::Visible,
            )
        }),
    )
}

pub(crate) fn portal_snapshot_with_rows(
    owner_revision: u64,
    rows: impl IntoIterator<
        Item = (
            crate::runtime::portal::UiPortalIdentity,
            Option<crate::runtime::portal::UiPortalIdentity>,
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            u64,
            crate::runtime::portal::UiPortalLifecyclePosture,
        ),
    >,
) -> crate::runtime::portal::UiPortalStackSnapshot {
    crate::runtime::portal::UiPortalStackSnapshot::for_test(owner_revision, rows)
}

pub(crate) fn input<'a>(
    extent: &'a UiOverlaySurfaceExtentSnapshot,
    snapshot: &'a crate::runtime::portal::UiPortalStackSnapshot,
    bindings: &'a [UiOverlayPortalBinding],
    motion: Option<&'a UiOverlayMotionSnapshot>,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
) -> UiOverlayCompositionInput<'a> {
    UiOverlayCompositionInput::new(
        UiOverlayApplicationGeneration::for_test(1),
        presentation,
        extent,
        snapshot,
        bindings,
        motion,
    )
}

pub(crate) fn input_with_generation<'a>(
    generation: u64,
    extent: &'a UiOverlaySurfaceExtentSnapshot,
    snapshot: &'a crate::runtime::portal::UiPortalStackSnapshot,
    bindings: &'a [UiOverlayPortalBinding],
    motion: Option<&'a UiOverlayMotionSnapshot>,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
) -> UiOverlayCompositionInput<'a> {
    UiOverlayCompositionInput::new(
        UiOverlayApplicationGeneration::for_test(generation),
        presentation,
        extent,
        snapshot,
        bindings,
        motion,
    )
}

pub(crate) fn presentation() -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
    worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap()
}
