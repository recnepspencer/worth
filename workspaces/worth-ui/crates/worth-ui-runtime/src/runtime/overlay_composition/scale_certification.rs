use super::planner::UiOverlayPlanCounters;
use super::*;
use std::collections::BTreeMap;
use worth_ui_dsl::{
    UiAppearanceAspect, UiAppearanceCell, UiAppearancePartitionAuthoring, UiAppearanceRole,
    UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity, UiBackdropDeclaration,
    UiBackdropExtentBasis, UiBackdropIdentity, UiBackdropMotionBasis, UiBackdropPlacement,
    UiBackdropPresenceBasis, UiBackdropScope, UiPortalDeclarationId,
    UiSemanticSurfaceDeclarationIdentity, UiThemeColor, UiThemeOpacity, UiThemeValue,
};

const PORTAL_COUNT: usize = 32;
const BACKDROP_COUNT: usize = 48;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiOverlayScaleEvidence {
    pub(crate) portal_rows: usize,
    pub(crate) portal_depth: usize,
    pub(crate) backdrop_rows: usize,
    pub(crate) dark_backdrops: usize,
    pub(crate) colored_backdrops: usize,
    pub(crate) transparent_backdrops: usize,
    pub(crate) initial: UiOverlayPlanCounters,
    pub(crate) successor: UiOverlayPlanCounters,
    pub(crate) released_rows: usize,
}

pub(crate) fn overlay_scale_evidence() -> UiOverlayScaleEvidence {
    let application = crate::facade::WorthUi::app()
        .with_change_profile(crate::runtime::rebind::UiChangeProfile::platform_pulse())
        .freeze()
        .map(crate::facade::entry::WorthUiCertificationApplicationTransition::activate_builder_host)
        .expect("AP10 overlay generation prepares");
    let generation =
        UiOverlayApplicationGeneration::from_prepared(application.generation_identity().clone());
    let declaration_surface = UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let portals = (0..PORTAL_COUNT)
        .map(|index| crate::runtime::portal::UiPortalIdentity::for_test(index as u64 + 1))
        .collect::<Vec<_>>();
    let portal_declarations = (0..PORTAL_COUNT)
        .map(|index| UiPortalDeclarationId::new(index as u64 + 1).unwrap())
        .collect::<Vec<_>>();
    let portal_snapshot = crate::runtime::portal::UiPortalStackSnapshot::for_test(
        1,
        portals.iter().enumerate().map(|(index, portal)| {
            (
                *portal,
                index.checked_sub(1).map(|parent| portals[parent]),
                runtime_surface,
                index as u64 + 1,
                crate::runtime::portal::UiPortalLifecyclePosture::Visible,
            )
        }),
    );
    let bindings = portals
        .iter()
        .zip(&portal_declarations)
        .map(|(portal, declaration)| UiOverlayPortalBinding::new(*declaration, *portal))
        .collect::<Vec<_>>();
    let declarations = (0..BACKDROP_COUNT)
        .map(|index| backdrop(index, declaration_surface, &portal_declarations))
        .collect::<Vec<_>>();
    let extent = UiOverlaySurfaceExtentSnapshot::seal(
        declaration_surface,
        runtime_surface,
        1,
        viewport(),
        [],
    )
    .unwrap();
    let motion = motion_snapshot(1, &portals, &portal_declarations);
    let presentation =
        worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
    let mut state =
        UiOverlayCompositionState::admit(declarations, 1, UiOverlayCapacityProfile::qualified())
            .unwrap();
    let initial = state
        .prepare_initial(input(
            &generation,
            &extent,
            &portal_snapshot,
            &bindings,
            &motion,
            presentation,
        ))
        .unwrap();
    let reservation = initial.reservation();
    let initial_counters = initial.counters();
    let categories = backdrop_categories(initial.snapshot());
    state.publish(initial).unwrap();

    let successor_motion = motion_snapshot(2, &portals, &portal_declarations);
    let successor = state
        .prepare_successor(
            input(
                &generation,
                &extent,
                &portal_snapshot,
                &bindings,
                &successor_motion,
                presentation,
            ),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::PortalMotion(
                portal_declarations[0],
            )]),
        )
        .unwrap();
    let successor_counters = successor.counters();
    state.publish(successor).unwrap();
    let released_rows = state.current.take().unwrap().participants().len();
    assert!(state.current().is_none());
    drop(state);

    UiOverlayScaleEvidence {
        portal_rows: reservation.portal_rows,
        portal_depth: portal_depth(&portal_snapshot),
        backdrop_rows: reservation.backdrop_rows,
        dark_backdrops: categories[0],
        colored_backdrops: categories[1],
        transparent_backdrops: categories[2],
        initial: initial_counters,
        successor: successor_counters,
        released_rows,
    }
}

fn portal_depth(snapshot: &crate::runtime::portal::UiPortalStackSnapshot) -> usize {
    let parents = snapshot
        .rows()
        .iter()
        .map(|row| (row.portal(), row.parent()))
        .collect::<BTreeMap<_, _>>();
    snapshot
        .rows()
        .iter()
        .map(|row| {
            let mut depth = 1;
            let mut parent = row.parent();
            while let Some(identity) = parent {
                depth += 1;
                parent = parents[&identity];
            }
            depth
        })
        .max()
        .unwrap_or(0)
}

fn input<'a>(
    generation: &'a UiOverlayApplicationGeneration,
    extent: &'a UiOverlaySurfaceExtentSnapshot,
    portals: &'a crate::runtime::portal::UiPortalStackSnapshot,
    bindings: &'a [UiOverlayPortalBinding],
    motion: &'a UiOverlayMotionSnapshot,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
) -> UiOverlayCompositionInput<'a> {
    UiOverlayCompositionInput {
        generation: generation.clone(),
        presentation,
        extent,
        portal_snapshot: portals,
        portal_bindings: bindings,
        motion: Some(motion),
    }
}

fn backdrop(
    index: usize,
    surface: UiSemanticSurfaceDeclarationIdentity,
    portals: &[UiPortalDeclarationId],
) -> UiBackdropDeclaration {
    let portal = portals[index % PORTAL_COUNT];
    let placement = if index < PORTAL_COUNT {
        UiBackdropPlacement::ImmediatelyBeforePortal(portal)
    } else {
        UiBackdropPlacement::ImmediatelyAfterPortal(portal)
    };
    UiBackdropDeclaration::admit(
        UiBackdropIdentity::new(index as u64 + 1).unwrap(),
        surface,
        UiBackdropScope::PerPortalInstance(portal),
        UiBackdropExtentBasis::SurfaceViewport(surface),
        UiBackdropPresenceBasis::WhilePortalPresented(portal),
        UiBackdropMotionBasis::PortalPresentation(portal),
        placement,
        &backdrop_role(index % 3),
    )
    .unwrap()
}

fn backdrop_role(category: usize) -> UiAppearanceRoleDeclaration {
    let (name, color, opacity) = match category {
        0 => ("scale.backdrop.dark", [8, 12, 16, 255], UiThemeOpacity::ONE),
        1 => (
            "scale.backdrop.colored",
            [32, 96, 192, 255],
            UiThemeOpacity::ONE,
        ),
        _ => (
            "scale.backdrop.transparent",
            [0, 0, 0, 0],
            UiThemeOpacity::ZERO,
        ),
    };
    UiAppearanceRole::authoring(UiAppearanceRoleIdentity::new(name).unwrap())
        .applies_to_backdrop()
        .cover(
            UiAppearanceAspect::Background,
            UiAppearancePartitionAuthoring::new([]).with_cell(
                UiAppearanceCell::when([])
                    .literal(UiThemeValue::Color(UiThemeColor::from_channels(color))),
            ),
        )
        .unwrap()
        .cover(
            UiAppearanceAspect::Opacity,
            UiAppearancePartitionAuthoring::new([])
                .with_cell(UiAppearanceCell::when([]).literal(UiThemeValue::Opacity(opacity))),
        )
        .unwrap()
        .build()
        .unwrap()
}

fn motion_snapshot(
    revision: u64,
    portals: &[crate::runtime::portal::UiPortalIdentity],
    declarations: &[UiPortalDeclarationId],
) -> UiOverlayMotionSnapshot {
    UiOverlayMotionSnapshot::seal(
        revision,
        portals
            .iter()
            .zip(declarations)
            .map(|(portal, declaration)| {
                UiOverlayMotionBinding::new(*declaration, *portal, revision)
            }),
    )
    .unwrap()
}

fn backdrop_categories(snapshot: &UiOverlayStackSnapshot) -> [usize; 3] {
    let mut result = [0; 3];
    for participant in snapshot.participants() {
        let UiOverlayStackParticipant::Backdrop(row) = participant else {
            continue;
        };
        match row.role().as_str() {
            "scale.backdrop.dark" => result[0] += 1,
            "scale.backdrop.colored" => result[1] += 1,
            "scale.backdrop.transparent" => result[2] += 1,
            role => panic!("unexpected AP10 backdrop role {role}"),
        }
    }
    result
}

fn viewport() -> worth_ui_host_contract::UiMountedCanonicalBox {
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: 0.0,
            y: 0.0,
            width: 1_280.0,
            height: 720.0,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::HostSurface,
        },
    )
    .unwrap()
}
