use super::super::*;
use super::support::{declaration, input, portal, portal_snapshot, presentation, surface_extent};
use worth_ui_dsl::UiPortalDeclarationId;

struct MotionFixture {
    state: UiOverlayCompositionState,
    extent: UiOverlaySurfaceExtentSnapshot,
    portals: crate::runtime::portal::UiPortalStackSnapshot,
    bindings: [UiOverlayPortalBinding; 2],
    motion: UiOverlayMotionSnapshot,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    dependent: UiPortalDeclarationId,
    first: crate::runtime::portal::UiPortalIdentity,
    second: crate::runtime::portal::UiPortalIdentity,
}

fn fixture() -> MotionFixture {
    let surface = worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity::new(1).unwrap();
    let dependent = UiPortalDeclarationId::new(10).unwrap();
    let unrelated = UiPortalDeclarationId::new(11).unwrap();
    let runtime_surface =
        worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let first = portal(20);
    let second = portal(21);
    let declarations = [
        declaration(
            1,
            surface,
            worth_ui_dsl::UiBackdropScope::PerPortalInstance(dependent),
            worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(dependent),
            worth_ui_dsl::UiBackdropMotionBasis::PortalPresentation(dependent),
            worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(dependent),
        ),
        declaration(
            2,
            surface,
            worth_ui_dsl::UiBackdropScope::PerPortalInstance(unrelated),
            worth_ui_dsl::UiBackdropPresenceBasis::WhilePortalPresented(unrelated),
            worth_ui_dsl::UiBackdropMotionBasis::None,
            worth_ui_dsl::UiBackdropPlacement::ImmediatelyBeforePortal(unrelated),
        ),
    ];
    let portals = portal_snapshot(runtime_surface, [(first, 1), (second, 2)]);
    let bindings = [
        UiOverlayPortalBinding::new(dependent, first),
        UiOverlayPortalBinding::new(unrelated, second),
    ];
    let motion =
        UiOverlayMotionSnapshot::seal(9, [UiOverlayMotionBinding::new(dependent, first, 1)])
            .unwrap();
    MotionFixture {
        state: UiOverlayCompositionState::admit(declarations, 3, Default::default()).unwrap(),
        extent: surface_extent(surface, runtime_surface, 2),
        portals,
        bindings,
        motion,
        presentation: presentation(),
        dependent,
        first,
        second,
    }
}

fn backdrop_for(
    snapshot: &UiOverlayStackSnapshot,
    declaration: u64,
    portal: crate::runtime::portal::UiPortalIdentity,
) -> &UiOverlayBackdropRow {
    snapshot
        .participants()
        .iter()
        .find_map(|participant| match participant {
            UiOverlayStackParticipant::Backdrop(row)
                if row.declaration().value() == declaration
                    && row.identity().scope() == UiOverlayBackdropInstanceScope::Portal(portal) =>
            {
                Some(row)
            }
            _ => None,
        })
        .expect("fixture backdrop remains materialized")
}

#[test]
fn motion_successor_rematerializes_only_indexed_dependents() {
    let mut fixture = fixture();
    let initial = fixture
        .state
        .prepare_initial(input(
            &fixture.extent,
            &fixture.portals,
            &fixture.bindings,
            Some(&fixture.motion),
            fixture.presentation,
        ))
        .unwrap();
    fixture.state.publish(initial).unwrap();
    let previous = fixture.state.current().unwrap().clone();
    let successor_motion = UiOverlayMotionSnapshot::seal(
        10,
        [UiOverlayMotionBinding::new(
            fixture.dependent,
            fixture.first,
            2,
        )],
    )
    .unwrap();
    let successor = fixture
        .state
        .prepare_successor(
            input(
                &fixture.extent,
                &fixture.portals,
                &fixture.bindings,
                Some(&successor_motion),
                fixture.presentation,
            ),
            &UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::PortalMotion(
                fixture.dependent,
            )]),
        )
        .unwrap();

    assert_eq!(successor.snapshot().motion_revision(), Some(10));
    assert_eq!(successor.counters().backdrop_declarations_selected(), 1);
    assert_eq!(successor.counters().backdrop_mechanics_changed(), 1);
    assert_eq!(
        backdrop_for(successor.snapshot(), 2, fixture.second),
        backdrop_for(&previous, 2, fixture.second)
    );
    assert_eq!(
        backdrop_for(successor.snapshot(), 1, fixture.first)
            .motion()
            .unwrap()
            .revision(),
        2
    );
    assert_eq!(fixture.state.current(), Some(&previous));
}

#[test]
fn missing_motion_input_denies_and_preserves_the_predecessor() {
    let mut fixture = fixture();
    let initial = fixture
        .state
        .prepare_initial(input(
            &fixture.extent,
            &fixture.portals,
            &fixture.bindings,
            Some(&fixture.motion),
            fixture.presentation,
        ))
        .unwrap();
    fixture.state.publish(initial).unwrap();
    let previous = fixture.state.current().unwrap().clone();
    let changes =
        UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::PortalMotion(fixture.dependent)]);

    assert_eq!(
        fixture.state.prepare_successor(
            input(
                &fixture.extent,
                &fixture.portals,
                &fixture.bindings,
                None,
                fixture.presentation,
            ),
            &changes,
        ),
        Err(UiOverlayCompositionDenial::MissingMotionBasis {
            backdrop: worth_ui_dsl::UiBackdropIdentity::new(1).unwrap(),
            portal: fixture.dependent,
        })
    );
    assert_eq!(fixture.state.current(), Some(&previous));
}

#[test]
fn missing_changed_motion_binding_is_denied_without_rejecting_the_revision() {
    let mut fixture = fixture();
    let initial = fixture
        .state
        .prepare_initial(input(
            &fixture.extent,
            &fixture.portals,
            &fixture.bindings,
            Some(&fixture.motion),
            fixture.presentation,
        ))
        .unwrap();
    fixture.state.publish(initial).unwrap();
    let previous = fixture.state.current().unwrap().clone();
    let missing = UiOverlayMotionSnapshot::seal(10, []).unwrap();
    let changes =
        UiOverlayChangeSet::from_changes([UiOverlayChangedBasis::PortalMotion(fixture.dependent)]);

    assert_eq!(
        fixture.state.prepare_successor(
            input(
                &fixture.extent,
                &fixture.portals,
                &fixture.bindings,
                Some(&missing),
                fixture.presentation,
            ),
            &changes,
        ),
        Err(UiOverlayCompositionDenial::MissingMotionBinding {
            backdrop: worth_ui_dsl::UiBackdropIdentity::new(1).unwrap(),
            portal: fixture.dependent,
        })
    );
    assert_eq!(fixture.state.current(), Some(&previous));
}
