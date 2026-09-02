use super::super::{
    UiOverlayCompositionOwnerLifecycle, UiOverlayOwnerBridgeDenial, UiOverlayStackSnapshot,
};
use super::gate2_owner_integration_support::{
    assert_overlay, close_topmost, extent_owner, OwnerIntegrationWorld,
};
use super::owner_support::{close_request, idempotency, prepared_generation_variant};
use crate::runtime::portal::{
    UiPortalExitRetentionReceipt, UiPortalLifecyclePosture, UiPortalOverlayBindingDenial,
    UiPortalOverlayBindingRow,
};

#[test]
fn gate2_owner_boundaries_retain_one_truthful_order_through_portal_lifecycle() {
    let mut world = OwnerIntegrationWorld::new();
    let mut lifecycle = admit(&mut world);
    let initial = lifecycle.current().cloned().unwrap();
    assert_initial(&world, &lifecycle, &initial);

    let topmost_snapshot = settle_topmost_close(&mut world, &mut lifecycle);
    reject_incoherent_extent(&world, &mut lifecycle, &topmost_snapshot);
    let retention = settle_exit_retention(&mut world, &mut lifecycle, &topmost_snapshot);
    settle_terminal(&mut world, &mut lifecycle, retention);
    settle_parent_close(&mut world, &mut lifecycle);
}

fn admit(world: &mut OwnerIntegrationWorld) -> UiOverlayCompositionOwnerLifecycle {
    UiOverlayCompositionOwnerLifecycle::admit_from_owners(
        [world.take_backdrop()],
        1,
        world.sources(),
    )
    .unwrap()
}

fn assert_initial(
    world: &OwnerIntegrationWorld,
    lifecycle: &UiOverlayCompositionOwnerLifecycle,
    initial: &UiOverlayStackSnapshot,
) {
    let binding_export = world
        .bindings
        .export(&world.portal_owner.stack_snapshot())
        .unwrap();
    assert_eq!(binding_export.rows().len(), 3);
    assert!(binding_export
        .rows()
        .iter()
        .all(|row| row.declaration() == world.portal_declaration));
    assert_overlay(
        initial,
        world.backdrop_identity,
        &[
            (world.parent, None, 1, UiPortalLifecyclePosture::Visible),
            (
                world.child,
                Some(world.parent),
                2,
                UiPortalLifecyclePosture::Visible,
            ),
            (world.sibling, None, 3, UiPortalLifecyclePosture::Visible),
        ],
    );
    assert_eq!(initial.portal_revision(), world.portal_owner.revision());
    assert_eq!(initial.motion_revision(), None);
    assert!(world.motion.overlay_owner_export().rows().is_empty());
    assert_eq!(lifecycle.current(), Some(initial));
}

fn settle_topmost_close(
    world: &mut OwnerIntegrationWorld,
    lifecycle: &mut UiOverlayCompositionOwnerLifecycle,
) -> UiOverlayStackSnapshot {
    let predecessor = lifecycle.current().cloned().unwrap();
    let changes = world.changes();
    assert_eq!(close_topmost(&mut world.portal_owner, 31625), world.sibling);
    let stale = lifecycle.prepare_successor_from_owners(world.sources(), &changes);
    assert_eq!(
        stale,
        Err(UiOverlayOwnerBridgeDenial::PortalBindings(
            UiPortalOverlayBindingDenial::MissingPortal,
        ))
    );
    assert_eq!(lifecycle.current(), Some(&predecessor));

    world
        .bindings
        .replace([
            UiPortalOverlayBindingRow::new(world.portal_declaration, world.parent),
            UiPortalOverlayBindingRow::new(world.portal_declaration, world.child),
        ])
        .unwrap();
    let prepared = lifecycle
        .prepare_successor_from_owners(world.sources(), &changes)
        .unwrap();
    assert_eq!(lifecycle.current(), Some(&predecessor));
    lifecycle.retain_prepared(prepared).unwrap();
    let snapshot = lifecycle.current().cloned().unwrap();
    assert_overlay(
        &snapshot,
        world.backdrop_identity,
        &[
            (world.parent, None, 1, UiPortalLifecyclePosture::Visible),
            (
                world.child,
                Some(world.parent),
                2,
                UiPortalLifecyclePosture::Visible,
            ),
        ],
    );
    assert_ne!(snapshot.portal_revision(), predecessor.portal_revision());
    assert_eq!(snapshot.portal_revision(), world.portal_owner.revision());
    snapshot
}

fn reject_incoherent_extent(
    world: &OwnerIntegrationWorld,
    lifecycle: &mut UiOverlayCompositionOwnerLifecycle,
    predecessor: &UiOverlayStackSnapshot,
) {
    let foreign_extent = extent_owner(
        prepared_generation_variant(),
        world.surface,
        world.runtime_surface,
        2,
    );
    let denied = lifecycle.prepare_successor_from_owners(
        world.sources_with_extent(&foreign_extent),
        &world.changes(),
    );
    assert!(matches!(
        denied,
        Err(UiOverlayOwnerBridgeDenial::OwnerExports(
            super::super::UiOverlayOwnerExportDenial::ExtentGenerationMismatch
        ))
    ));
    assert_eq!(lifecycle.current(), Some(predecessor));
}

fn settle_exit_retention(
    world: &mut OwnerIntegrationWorld,
    lifecycle: &mut UiOverlayCompositionOwnerLifecycle,
    predecessor: &UiOverlayStackSnapshot,
) -> UiPortalExitRetentionReceipt {
    let child_close = world
        .portal_owner
        .prepare(close_request(world.child, world.runtime_surface, 31626))
        .unwrap();
    assert!(!child_close.is_idempotent());
    let (_, retention) = world
        .portal_owner
        .commit_published_with_exit_retention(child_close, true)
        .unwrap();
    let retention = retention.unwrap();
    assert_eq!(retention.portal(), world.child);
    assert_eq!(world.portal_owner.active_count(), 2);
    assert_eq!(
        world.portal_owner.stack_snapshot().rows()[1].lifecycle(),
        UiPortalLifecyclePosture::Closing
    );

    let closing = lifecycle
        .prepare_successor_from_owners(world.sources(), &world.changes())
        .unwrap();
    assert_eq!(lifecycle.current(), Some(predecessor));
    lifecycle.retain_prepared(closing).unwrap();
    let snapshot = lifecycle.current().cloned().unwrap();
    assert_overlay(
        &snapshot,
        world.backdrop_identity,
        &[
            (world.parent, None, 1, UiPortalLifecyclePosture::Visible),
            (
                world.child,
                Some(world.parent),
                2,
                UiPortalLifecyclePosture::Closing,
            ),
        ],
    );
    assert_ne!(snapshot.portal_revision(), predecessor.portal_revision());
    assert_eq!(snapshot.portal_revision(), world.portal_owner.revision());

    let reconstructed = lifecycle.reconstruct_from_owners(world.sources()).unwrap();
    assert_eq!(reconstructed.snapshot(), &snapshot);
    lifecycle.retain_prepared(reconstructed).unwrap();
    assert_eq!(lifecycle.current(), Some(&snapshot));
    retention
}

fn settle_terminal(
    world: &mut OwnerIntegrationWorld,
    lifecycle: &mut UiOverlayCompositionOwnerLifecycle,
    retention: UiPortalExitRetentionReceipt,
) {
    let predecessor = lifecycle.current().cloned().unwrap();
    let terminal_transition = world
        .portal_owner
        .prepare_exit_terminal(retention, idempotency(31627))
        .unwrap();
    world
        .portal_owner
        .commit_published(terminal_transition)
        .unwrap();
    assert_eq!(world.portal_owner.active_count(), 1);
    world
        .bindings
        .replace([UiPortalOverlayBindingRow::new(
            world.portal_declaration,
            world.parent,
        )])
        .unwrap();

    let terminal = lifecycle
        .prepare_successor_from_owners(world.sources(), &world.changes())
        .unwrap();
    assert_eq!(lifecycle.current(), Some(&predecessor));
    lifecycle.retain_prepared(terminal).unwrap();
    let snapshot = lifecycle.current().cloned().unwrap();
    assert_overlay(
        &snapshot,
        world.backdrop_identity,
        &[(world.parent, None, 1, UiPortalLifecyclePosture::Visible)],
    );
    assert_ne!(snapshot.portal_revision(), predecessor.portal_revision());
    assert_eq!(snapshot.portal_revision(), world.portal_owner.revision());
}

fn settle_parent_close(
    world: &mut OwnerIntegrationWorld,
    lifecycle: &mut UiOverlayCompositionOwnerLifecycle,
) {
    let predecessor = lifecycle.current().cloned().unwrap();
    let parent_close = world
        .portal_owner
        .prepare(close_request(world.parent, world.runtime_surface, 31628))
        .unwrap();
    world.portal_owner.commit_published(parent_close).unwrap();
    world.bindings.replace([]).unwrap();

    let empty = lifecycle
        .prepare_successor_from_owners(world.sources(), &world.changes())
        .unwrap();
    assert_eq!(lifecycle.current(), Some(&predecessor));
    lifecycle.retain_prepared(empty).unwrap();
    let snapshot = lifecycle.current().cloned().unwrap();
    assert_overlay(&snapshot, world.backdrop_identity, &[]);
    assert_ne!(snapshot.portal_revision(), predecessor.portal_revision());
    assert_eq!(snapshot.portal_revision(), world.portal_owner.revision());
    assert_eq!(world.portal_owner.active_count(), 0);
}
