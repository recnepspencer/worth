use super::{
    test_support::{
        idempotency, open_request, portal, presented_geometry, semantic_surface, state,
        viewport_bounds,
    },
    UiPortalDismissalPreparation, UiPortalDismissalTrigger, UiPortalIdentity,
    UiPortalInputShielding, UiPortalRuntimeState, UiPortalServiceDisposition,
    UiPortalServiceRequest, UiPortalServiceTransitionDenial, UiPortalStackOrdinalIssuer,
};

#[test]
fn idempotent_open_preserves_identity_and_stack_ordinal() {
    let mut state = state();
    let portal = portal(701, 801);
    let request = open_request(portal, 901);
    let opened = state.prepare(request).unwrap();
    let ordinal = opened.stack_ordinal().expect("open carries an ordinal");
    state.commit_published(opened).unwrap();
    let before_duplicate = state.stack_snapshot();

    let duplicate = state.prepare(request).unwrap();
    let receipt = state.commit_published(duplicate).unwrap();

    assert_eq!(
        receipt.disposition(),
        UiPortalServiceDisposition::Idempotent
    );
    assert_eq!(state.active_count(), 1);
    assert_eq!(state.stack_snapshot().rows()[0].portal(), portal);
    assert_eq!(state.stack_snapshot().rows()[0].ordinal(), ordinal);
    assert_eq!(
        state.stack_snapshot().rows()[0].ordinal(),
        before_duplicate.rows()[0].ordinal()
    );
}

#[test]
fn lawful_topmost_replacement_keeps_identity_and_mints_a_new_ordinal() {
    let mut state = state();
    let portal = portal(702, 802);
    let first = open_request(portal, 902);
    let surface = first.semantic_surface();
    let opened = state.prepare(first).unwrap();
    let original_ordinal = opened.stack_ordinal().unwrap();
    state.commit_published(opened).unwrap();

    let replacement = state
        .prepare(moved_open(portal, 903, surface, 2))
        .expect("the current topmost portal may replace its placement");
    let replacement_ordinal = replacement.stack_ordinal().unwrap();
    assert!(replacement_ordinal > original_ordinal);
    let receipt = state.commit_published(replacement).unwrap();

    assert_eq!(receipt.disposition(), UiPortalServiceDisposition::Opened);
    assert_eq!(state.active_count(), 1);
    assert_eq!(state.stack_snapshot().rows()[0].portal(), portal);
    assert_eq!(
        state.stack_snapshot().rows()[0].ordinal(),
        replacement_ordinal
    );
}

#[test]
fn replacement_below_the_topmost_row_is_denied_without_mutation() {
    let mut state = state();
    let lower = portal(703, 803);
    let lower_request = open_request(lower, 904);
    let lower_surface = lower_request.semantic_surface();
    state
        .commit_published(state.prepare(lower_request).unwrap())
        .unwrap();
    let upper = portal(704, 804);
    state
        .commit_published(state.prepare(open_request(upper, 905)).unwrap())
        .unwrap();
    let before = state.stack_snapshot();

    assert!(matches!(
        state.prepare(moved_open(lower, 906, lower_surface, 3)),
        Err(UiPortalServiceTransitionDenial::ReplacementNotTopmost)
    ));
    assert_eq!(state.stack_snapshot(), before);
    assert_eq!(state.revision(), before.owner_revision());
}

#[test]
fn topmost_dismissal_uses_activation_order_for_same_depth_siblings() {
    let mut state = state();
    let first = portal(705, 805);
    let second = portal(706, 806);
    open_live(&mut state, first, 907);
    open_live(&mut state, second, 908);

    let UiPortalDismissalPreparation::Prepared(dismissal) = state
        .prepare_dismissal(UiPortalDismissalTrigger::Escape, None, idempotency(909))
        .unwrap()
    else {
        panic!("Escape must select a live topmost portal")
    };

    assert_eq!(dismissal.portal(), second);
}

#[test]
fn nested_and_sibling_rows_are_sealed_in_one_total_order() {
    let mut state = state();
    let parent = portal(707, 807);
    let child = portal(708, 808);
    let sibling = portal(709, 809);
    open_live(&mut state, parent, 910);
    let child_geometry = presented_geometry(11);
    let child_open = UiPortalServiceRequest::open_nested(
        child,
        idempotency(911),
        child_geometry,
        viewport_bounds(child_geometry),
        semantic_surface(),
        parent,
        UiPortalInputShielding::ModalSurface,
    );
    state
        .commit_published(state.prepare(child_open).unwrap())
        .unwrap();
    open_live(&mut state, sibling, 912);

    let snapshot = state.stack_snapshot();
    assert_eq!(
        snapshot
            .rows()
            .iter()
            .map(|row| row.portal())
            .collect::<Vec<_>>(),
        [parent, child, sibling]
    );
    assert_eq!(snapshot.rows()[0].parent(), None);
    assert_eq!(snapshot.rows()[1].parent(), Some(parent));
    assert_eq!(snapshot.rows()[2].parent(), None);
    assert!(snapshot
        .rows()
        .windows(2)
        .all(|rows| rows[0].ordinal() < rows[1].ordinal()));
}

#[test]
fn reconstructed_order_index_produces_the_same_sealed_snapshot() {
    let mut state = state();
    let first = portal(710, 810);
    let second = portal(711, 811);
    open_live(&mut state, first, 913);
    open_live(&mut state, second, 914);
    let before = state.stack_snapshot();

    state.reconstruct_stack_order_for_test();

    assert_eq!(state.stack_snapshot(), before);
}

#[test]
fn live_row_capacity_denies_the_next_open_before_any_effect() {
    let mut state = state();
    let limit = super::capacity::UI_PORTAL_LIVE_ROW_CAPACITY;
    for offset in 0..limit {
        let offset = u64::try_from(offset).unwrap();
        let identity = portal(20_000 + offset, 30_000 + offset);
        let transition = state
            .prepare(open_request(identity, 40_000 + offset))
            .expect("every row below the live capacity prepares");
        state.commit_published(transition).unwrap();
    }
    let before = state.stack_snapshot();
    let rejected = portal(21_024, 31_024);

    assert!(matches!(
        state.prepare(open_request(rejected, 41_024)),
        Err(UiPortalServiceTransitionDenial::LiveRowCapacityExceeded { limit: 1_024 })
    ));
    assert_eq!(state.active_count(), limit);
    assert_eq!(state.stack_snapshot(), before);
}

#[test]
fn stale_replacement_cannot_consume_a_second_ordinal() {
    let mut state = state();
    let portal_identity = portal(712, 812);
    let first = open_request(portal_identity, 915);
    let surface = first.semantic_surface();
    state
        .commit_published(state.prepare(first).unwrap())
        .unwrap();
    let prepared = state
        .prepare(moved_open(portal_identity, 916, surface, 12))
        .unwrap();
    let stale = state
        .prepare(moved_open(portal_identity, 917, surface, 13))
        .unwrap();
    let replacement_ordinal = prepared.stack_ordinal().unwrap();
    state.commit_published(prepared).unwrap();
    let before_stale = state.stack_snapshot();

    assert_eq!(
        state.commit_published(stale),
        Err(UiPortalServiceTransitionDenial::StalePlan)
    );
    assert_eq!(state.stack_snapshot(), before_stale);

    let sibling = portal(713, 813);
    let sibling_open = state.prepare(open_request(sibling, 918)).unwrap();
    assert_eq!(
        sibling_open.stack_ordinal().unwrap().value(),
        replacement_ordinal.value() + 1
    );
}

#[test]
fn ordinal_issuer_survives_portal_installation_replacement() {
    let persistence = crate::runtime::UiServiceStatePersistencePosture::SessionRestoreCandidate;
    let policy = crate::declaration::UiPortalPolicy::dropdown();
    let mut predecessor = UiPortalRuntimeState::new_with_policy_and_ordinal_issuer(
        persistence,
        policy,
        UiPortalStackOrdinalIssuer::new(),
    );
    let first = open_request(portal(714, 814), 919);
    let first_transition = predecessor.prepare(first).unwrap();
    assert_eq!(first_transition.stack_ordinal().unwrap().value(), 1);
    predecessor.commit_published(first_transition).unwrap();
    predecessor.shutdown();
    let issuer = predecessor.take_stack_ordinal_issuer();

    let successor =
        UiPortalRuntimeState::new_with_policy_and_ordinal_issuer(persistence, policy, issuer);
    let transition = successor
        .prepare(open_request(portal(715, 815), 920))
        .unwrap();

    assert_eq!(transition.stack_ordinal().unwrap().value(), 2);
}

#[test]
fn rebind_removes_missing_owners_and_their_portal_owned_descendants() {
    let mut state = state();
    let parent = portal(716, 816);
    let child = portal(717, 817);
    let sibling = portal(718, 818);
    open_live(&mut state, parent, 921);
    let geometry = presented_geometry(14);
    let child_request = UiPortalServiceRequest::open_nested(
        child,
        idempotency(922),
        geometry,
        viewport_bounds(geometry),
        semantic_surface(),
        parent,
        UiPortalInputShielding::ContentBounds,
    );
    state
        .commit_published(state.prepare(child_request).unwrap())
        .unwrap();
    open_live(&mut state, sibling, 923);
    let before_revision = state.revision();

    let removed = state.remove_rebound_portals(&successor_view(&[sibling]));

    assert!(removed.contains(&parent));
    assert!(removed.contains(&child));
    assert!(!removed.contains(&sibling));
    assert_eq!(state.active_count(), 1);
    assert_eq!(state.stack_snapshot().rows()[0].portal(), sibling);
    assert_eq!(state.revision(), before_revision + 1);

    let revision = state.revision();
    assert!(state
        .remove_rebound_portals(&successor_view(&[sibling]))
        .is_empty());
    assert_eq!(state.revision(), revision);
}

#[test]
fn wrong_surface_close_is_denied_without_touching_current_portal_truth() {
    let mut state = state();
    let portal = portal(719, 819);
    let open = open_request(portal, 924);
    state
        .commit_published(state.prepare(open).unwrap())
        .unwrap();
    let before = state.stack_snapshot();

    assert!(matches!(
        state.prepare(UiPortalServiceRequest::close(
            portal,
            idempotency(925),
            super::UiPortalDismissalCause::Escape,
            semantic_surface(),
        )),
        Err(UiPortalServiceTransitionDenial::PortalSurfaceMismatch)
    ));
    assert_eq!(state.stack_snapshot(), before);
}

fn open_live(
    state: &mut UiPortalRuntimeState,
    portal: UiPortalIdentity,
    lineage: u64,
) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
    let request = open_request(portal, lineage);
    let surface = request.semantic_surface();
    state
        .commit_published(state.prepare(request).unwrap())
        .unwrap();
    surface
}

fn moved_open(
    portal: UiPortalIdentity,
    lineage: u64,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    epoch: u64,
) -> UiPortalServiceRequest {
    let geometry = presented_geometry(epoch);
    UiPortalServiceRequest::open(
        portal,
        idempotency(lineage),
        geometry,
        Some(viewport_bounds(geometry)),
        surface,
    )
}

fn successor_view(portals: &[UiPortalIdentity]) -> crate::mounting::UiMountedIdentityView {
    let instances = portals
        .iter()
        .map(|portal| {
            let basis = crate::mounting::UiMountedIdentityBasis::new(
                portal.owner().graph_node(),
                crate::graph::UiRepeatedInstanceBasis::unavailable(),
                semantic_surface(),
                worth_ui_host_contract::UiMountIncarnation::mint_unbound()
                    .expect("rebind fixture mount incarnation"),
            );
            crate::mounting::UiMountedInstanceIdentityView::new(
                portal.owner().mounted_instance_identity(),
                basis,
            )
        })
        .collect();
    crate::mounting::UiMountedIdentityView::new(instances, Vec::new(), None, Vec::new())
}
