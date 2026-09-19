use super::super::{context, node_affinity, node_fragment, surface_mechanic, surface_work};
use super::support::{removed_surface_fragment, surface_affinity, surface_at, work_with_manifest};
use crate::*;

#[test]
fn node_removal_keeps_predecessor_manifest_and_damage_truth() {
    let context = context();
    let fragment = removed_surface_fragment(&context);

    assert_eq!(fragment.work().successor().mechanics(), &[]);
    assert_eq!(fragment.work().changes().len(), 1);
    assert_eq!(fragment.work().damage().len(), 1);
    assert!(
        UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
            context.frame,
            context.attempt,
            [fragment],
        )
        .is_ok()
    );
}

#[test]
fn node_both_receipts_accept_unchanged_work() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let predecessor_mechanic = surface_at(context.predecessor, instance);
    let predecessor = predecessor_mechanic.node_receipt();
    let predecessor_identity =
        UiMountedAppearanceMechanic::Surface(predecessor_mechanic).identity();
    let successor_mechanic = surface_at(context.frame, instance);
    let successor = successor_mechanic.node_receipt();
    let work = work_with_manifest(
        &context,
        UiMountedAppearanceWorkPosture::Unchanged,
        [predecessor_identity],
        [UiMountedAppearanceMechanic::Surface(successor_mechanic)],
        [],
        [],
    );
    let fragment = UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: Some(predecessor),
            successor: Some(successor),
        },
        work,
        [],
        context.requirement,
        node_affinity(&context, successor),
    )
    .unwrap();

    assert!(
        UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
            context.frame,
            context.attempt,
            [fragment],
        )
        .is_ok()
    );
}

#[test]
fn node_both_receipts_accept_replace_against_each_exact_basis() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let predecessor_mechanic = surface_at(context.predecessor, instance);
    let predecessor = predecessor_mechanic.node_receipt();
    let predecessor_identity =
        UiMountedAppearanceMechanic::Surface(predecessor_mechanic).identity();
    let successor_mechanic = surface_at(context.frame, instance);
    let successor = successor_mechanic.node_receipt();
    let successor_mechanic = UiMountedAppearanceMechanic::Surface(successor_mechanic);
    let change = UiMountedAppearanceMechanicChange::replacement(
        predecessor_identity.clone(),
        successor_mechanic.clone(),
    )
    .unwrap();
    let work = work_with_manifest(
        &context,
        UiMountedAppearanceWorkPosture::Delta,
        [predecessor_identity],
        [successor_mechanic],
        [change],
        [UiAppearanceDamageRegion::new(0, 0, 1, 1).unwrap()],
    );
    let fragment = UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: Some(predecessor),
            successor: Some(successor),
        },
        work,
        [],
        context.requirement,
        node_affinity(&context, successor),
    )
    .unwrap();

    assert_eq!(fragment.work().changes().len(), 1);
}

#[test]
fn node_both_receipts_accept_insert_against_successor_receipt() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let predecessor_mechanic = surface_at(context.predecessor, instance);
    let predecessor = predecessor_mechanic.node_receipt();
    let predecessor_identity =
        UiMountedAppearanceMechanic::Surface(predecessor_mechanic).identity();
    let successor_surface = surface_at(context.frame, instance);
    let successor = successor_surface.node_receipt();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(context.frame).unwrap();
    let span = UiMountedTextPaintSpanIdentity::from_runtime_mounting([31; 32]);
    let candidate = super::super::text_row(&context, instance, successor, span);
    let foreground = UiMountedTextForegroundAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedTextForegroundAppearanceCompletionInput {
            issuer,
            node_receipt: successor,
            command: UiMountedPaintCommandIdentity::semantic_text(&candidate),
            paint_span: span,
            foreground: UiMountedAppearanceColor::from_straight_srgba([1, 2, 3, 255]),
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap();
    let inserted = UiMountedAppearanceMechanic::TextForeground(foreground);
    let work = work_with_manifest(
        &context,
        UiMountedAppearanceWorkPosture::Delta,
        [predecessor_identity],
        [
            UiMountedAppearanceMechanic::Surface(successor_surface),
            inserted.clone(),
        ],
        [UiMountedAppearanceMechanicChange::Insert(inserted)],
        [],
    );
    let fragment = UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor: Some(predecessor),
            successor: Some(successor),
        },
        work,
        [candidate],
        context.requirement,
        node_affinity(&context, successor),
    )
    .unwrap();

    assert_eq!(fragment.text_candidates().len(), 1);
}

#[test]
fn overlay_removal_accepts_predecessor_only_attribution() {
    let context = context();
    let portal_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let predecessor_surface = surface_at(context.predecessor, portal_instance);
    let portal = UiMountedPortalSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        portal_instance,
        predecessor_surface,
    )
    .unwrap();
    let identity = UiMountedAppearanceMechanic::PortalSurface(portal).identity();
    let work = work_with_manifest(
        &context,
        UiMountedAppearanceWorkPosture::Delta,
        [identity.clone()],
        [],
        [UiMountedAppearanceMechanicChange::Remove(identity)],
        [UiAppearanceDamageRegion::new(0, 0, 2, 2).unwrap()],
    );
    let fragment = UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(context.surface),
        work,
        [],
        context.requirement,
        surface_affinity(&context),
    )
    .unwrap();

    assert!(fragment.work().successor().mechanics().is_empty());
}

#[test]
fn node_affinity_must_name_the_exact_successor_receipt() {
    let context = context();
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mechanic = surface_mechanic(&context, instance);
    let (receipt, work) = surface_work(&context, mechanic);
    let wrong_receipt = UiMountedNodeReceiptIssuer::mint_for(context.frame)
        .unwrap()
        .receipt_for(UiMountedInstanceIdentity::mint_unbound().unwrap());

    assert_eq!(
        UiUnpublishedAppearanceFragment::from_runtime_mounting(
            UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                predecessor: None,
                successor: Some(receipt),
            },
            work,
            [],
            context.requirement,
            node_affinity(&context, wrong_receipt),
        ),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::PresentationReceiptAffinityMismatch)
    );
}

#[test]
fn projection_stops_at_fragment_capacity_plus_one() {
    struct BoundedFragments {
        yielded: usize,
        fragment: UiUnpublishedAppearanceFragment,
    }

    impl Iterator for BoundedFragments {
        type Item = UiUnpublishedAppearanceFragment;

        fn next(&mut self) -> Option<Self::Item> {
            if self.yielded == UI_UNPUBLISHED_APPEARANCE_FRAGMENT_CAPACITY + 1 {
                panic!("projection admission drained beyond capacity plus one");
            }
            self.yielded += 1;
            Some(self.fragment.clone())
        }
    }

    let context = context();
    let fragment = node_fragment(&context, UiMountedInstanceIdentity::mint_unbound().unwrap());
    assert_eq!(
        UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
            context.frame,
            context.attempt,
            BoundedFragments {
                yielded: 0,
                fragment,
            },
        ),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentCapacityExceeded)
    );
}
