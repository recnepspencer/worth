use super::super::test_support::{
    idempotency, portal, presented_geometry, semantic_surface, state, viewport_bounds,
};
use super::super::UiPortalServiceRequest;

#[test]
fn reopening_a_visible_portal_prepares_exactly_its_committed_stack() {
    let mut owner = state();
    let portal = portal(851, 861);
    let surface = semantic_surface();
    let geometry = presented_geometry(1);
    let request = |lineage| {
        UiPortalServiceRequest::open(
            portal,
            idempotency(lineage),
            geometry,
            Some(viewport_bounds(geometry)),
            surface,
        )
    };
    let first = owner.prepare(request(871)).unwrap();
    owner.commit_published(first).unwrap();
    let predecessor = owner.surface_stack_snapshot(surface);
    let reopen = owner.prepare(request(872)).unwrap();
    let prepared = predecessor
        .clone()
        .for_transition(surface, &reopen, false)
        .unwrap();
    assert_eq!(prepared.rows.len(), 1);
    assert_eq!(
        prepared.order.len(),
        1,
        "one live portal has exactly one stack position"
    );
    assert_eq!(prepared.topmost_portal(), Some(portal));
    assert_eq!(
        owner
            .surface_stack_snapshot(surface)
            .order
            .iter()
            .collect::<Vec<_>>(),
        predecessor.order.iter().collect::<Vec<_>>(),
        "preparation preserves the predecessor"
    );
    owner.commit_published(reopen).unwrap();
    let accepted = owner.surface_stack_snapshot(surface);
    assert_eq!(
        prepared.order.iter().collect::<Vec<_>>(),
        accepted.order.iter().collect::<Vec<_>>()
    );
    assert_eq!(
        prepared.rows.iter().collect::<Vec<_>>(),
        accepted.rows.iter().collect::<Vec<_>>()
    );
}
