use super::test_support::{idempotency, open_request, portal, semantic_surface, state};
use super::{UiPortalInputShielding, UiPortalServiceRequest};

#[test]
fn rebind_removes_missing_owners_and_their_portal_owned_descendants() {
    let mut state = state();
    let parent = portal(716, 816);
    let child = portal(717, 817);
    let sibling = portal(718, 818);
    open_live(&mut state, parent, 921);
    let geometry = super::test_support::presented_geometry(14);
    let child_request = UiPortalServiceRequest::open_nested(
        child,
        idempotency(922),
        geometry,
        super::test_support::viewport_bounds(geometry),
        semantic_surface(),
        parent,
        UiPortalInputShielding::ContentBounds,
    );
    state
        .commit_published(state.prepare(child_request).unwrap())
        .unwrap();
    open_live(&mut state, sibling, 923);
    let before_revision = state.revision();

    let removal = state.prepare_rebound_portal_removal(&successor_view(&[sibling]), false);
    let removed = state
        .commit_rebound_portal_removal(removal)
        .expect("prepared Portal removal remains current");

    assert!(removed.contains(&parent));
    assert!(removed.contains(&child));
    assert!(!removed.contains(&sibling));
    assert_eq!(state.active_count(), 1);
    assert_eq!(state.stack_snapshot().rows()[0].portal(), sibling);
    assert_eq!(state.revision(), before_revision + 1);

    let revision = state.revision();
    let removal = state.prepare_rebound_portal_removal(&successor_view(&[sibling]), false);
    assert!(state
        .commit_rebound_portal_removal(removal)
        .expect("empty Portal removal remains current")
        .is_empty());
    assert_eq!(state.revision(), revision);
}

#[test]
fn prepared_rebind_removal_is_revision_current_before_it_removes_rows() {
    let mut state = state();
    let retained = portal(719, 819);
    open_live(&mut state, retained, 924);
    let removal = state.prepare_rebound_portal_removal(&successor_view(&[]), false);
    let before_stale_commit = state.stack_snapshot();

    open_live(&mut state, portal(720, 820), 925);

    assert!(state.commit_rebound_portal_removal(removal).is_err());
    assert_eq!(state.stack_snapshot().rows()[0].portal(), retained);
    assert_eq!(state.active_count(), before_stale_commit.rows().len() + 1);
}

#[test]
fn accepted_motion_presentation_rebinds_only_same_frame_records_on_its_surface() {
    let mut state = state();
    let surface = semantic_surface();
    let rebound = portal(721, 821);
    let foreign_frame = portal(722, 822);
    open_live_on_surface(&mut state, rebound, 926, surface);
    open_live_on_surface(&mut state, foreign_frame, 927, surface);
    let committed = state
        .committed_presentation_for(rebound)
        .expect("live Portal carries its accepted placement");
    let foreign = state
        .committed_presentation_for(foreign_frame)
        .expect("live Portal carries its accepted placement");
    assert_ne!(committed.frame(), foreign.frame());
    let advanced = worth_ui_host_contract::UiHostObservationPresentationBasis::new(
        committed.host_surface(),
        committed.frame(),
        committed.binding(),
        worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(7),
    );

    state.rebind_presented_motion_presentation(surface, advanced);

    assert_eq!(state.committed_presentation_for(rebound), Some(advanced));
    assert_eq!(
        state.committed_presentation_for(foreign_frame),
        Some(foreign)
    );
    assert_eq!(state.topmost_presentation(), Some(foreign));
}

fn open_live_on_surface(
    state: &mut super::UiPortalRuntimeState,
    portal: super::UiPortalIdentity,
    lineage: u64,
    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) {
    let geometry = super::test_support::presented_geometry(1);
    let request = UiPortalServiceRequest::open(
        portal,
        idempotency(lineage),
        geometry,
        Some(super::test_support::viewport_bounds(geometry)),
        surface,
    );
    state
        .commit_published(state.prepare(request).unwrap())
        .unwrap();
}

fn open_live(
    state: &mut super::UiPortalRuntimeState,
    portal: super::UiPortalIdentity,
    lineage: u64,
) {
    let request = open_request(portal, lineage);
    state
        .commit_published(state.prepare(request).unwrap())
        .unwrap();
}

fn successor_view(portals: &[super::UiPortalIdentity]) -> crate::mounting::UiMountedIdentityView {
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
