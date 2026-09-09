use super::super::Context;
use super::support::{pointer_fragment, surface_affinity, work_with_manifest};
use crate::*;

fn pointer(
    context: &Context,
    pointer: UiHostPointerIdentity,
    target: UiMountedInstanceIdentity,
) -> UiMountedPointerAffordanceMechanic {
    UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
        pointer,
        context.surface,
        target,
        UiPointerAffordanceFamily::Activation,
    )
}

#[test]
fn pointer_removal_accepts_predecessor_only_attribution() {
    let context = super::super::context();
    let pointer_id = UiHostPointerIdentity::new(17);
    let old = pointer(
        &context,
        pointer_id,
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
    );
    let identity = UiMountedAppearanceMechanic::Pointer(old).identity();
    let work = work_with_manifest(
        &context,
        UiMountedAppearanceWorkPosture::Delta,
        [identity.clone()],
        [],
        [UiMountedAppearanceMechanicChange::Remove(identity)],
        [],
    );
    assert_eq!(
        UiUnpublishedAppearanceFragment::from_runtime_mounting(
            UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
                surface: context.surface,
                pointer: UiHostPointerIdentity::new(18),
            },
            work.clone(),
            [],
            context.requirement,
            surface_affinity(&context),
        ),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch),
        "removal-only work must identify the departing pointer",
    );
    let fragment = UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
            surface: context.surface,
            pointer: pointer_id,
        },
        work,
        [],
        context.requirement,
        surface_affinity(&context),
    )
    .unwrap();

    assert!(fragment.work().successor().mechanics().is_empty());
}

#[test]
fn pointer_retarget_uses_remove_and_insert_with_one_current_pointer() {
    let context = super::super::context();
    let pointer_id = UiHostPointerIdentity::new(23);
    let old = pointer(
        &context,
        pointer_id,
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
    );
    let new = pointer(
        &context,
        pointer_id,
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
    );
    let old_identity = UiMountedAppearanceMechanic::Pointer(old).identity();
    let new_mechanic = UiMountedAppearanceMechanic::Pointer(new);
    let work = work_with_manifest(
        &context,
        UiMountedAppearanceWorkPosture::Delta,
        [old_identity.clone()],
        [new_mechanic.clone()],
        [
            UiMountedAppearanceMechanicChange::Remove(old_identity),
            UiMountedAppearanceMechanicChange::Insert(new_mechanic),
        ],
        [],
    );
    let fragment = UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
            surface: context.surface,
            pointer: pointer_id,
        },
        work,
        [],
        context.requirement,
        surface_affinity(&context),
    )
    .unwrap();

    assert_eq!(fragment.work().successor().mechanics().len(), 1);
    assert_eq!(fragment.work().changes().len(), 2);
}

#[test]
fn pointer_primary_handoff_keeps_one_surface_fragment() {
    let context = super::super::context();
    let old = pointer(
        &context,
        UiHostPointerIdentity::new(101),
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
    );
    let new = pointer(&context, UiHostPointerIdentity::new(102), old.target());
    let fragment = handoff(
        &context,
        &[UiMountedAppearanceMechanic::Pointer(old).identity()],
        new,
        new.pointer(),
    )
    .expect("a surface can atomically replace its primary pointer");
    assert_eq!(fragment.work().changes().len(), 2);
    assert_eq!(
        fragment.work().successor().mechanics(),
        &[UiMountedAppearanceMechanic::Pointer(new)],
    );
    assert!(fragment.work().damage().is_empty());
    let frame = UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
        context.frame,
        context.attempt,
        [fragment],
    )
    .unwrap();
    assert_eq!(frame.fragments().len(), 1);
}

#[test]
fn pointer_primary_handoff_rejects_foreign_or_ambiguous_predecessors() {
    let context = super::super::context();
    let old = pointer(
        &context,
        UiHostPointerIdentity::new(103),
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
    );
    let new = pointer(&context, UiHostPointerIdentity::new(104), old.target());
    let old_identity = UiMountedAppearanceMechanic::Pointer(old).identity();
    let foreign = UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
        old.pointer(),
        UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        old.target(),
        old.family(),
    );
    let second = pointer(&context, UiHostPointerIdentity::new(105), old.target());
    for predecessors in [
        vec![UiMountedAppearanceMechanic::Pointer(foreign).identity()],
        vec![UiMountedAppearanceMechanicIdentity::Surface(old.target())],
        vec![
            old_identity.clone(),
            UiMountedAppearanceMechanic::Pointer(second).identity(),
        ],
        vec![
            old_identity.clone(),
            UiMountedAppearanceMechanicIdentity::Surface(old.target()),
        ],
    ] {
        assert!(handoff(&context, &predecessors, new, new.pointer()).is_err());
    }
    assert_eq!(
        handoff(&context, &[old_identity], new, old.pointer()),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch),
        "a fragment with a successor must identify the arriving pointer",
    );
}

#[test]
fn pointer_fragment_rejects_empty_predecessor_and_successor() {
    let context = super::super::context();
    let work = work_with_manifest(
        &context,
        UiMountedAppearanceWorkPosture::Unchanged,
        [],
        [],
        [],
        [],
    );
    assert_eq!(
        UiUnpublishedAppearanceFragment::from_runtime_mounting(
            UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
                surface: context.surface,
                pointer: UiHostPointerIdentity::new(106),
            },
            work,
            [],
            context.requirement,
            surface_affinity(&context),
        ),
        Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch),
    );
}

#[test]
fn pointer_handoff_cannot_carry_overlay_order_changes() {
    let context = super::super::context();
    let old = pointer(
        &context,
        UiHostPointerIdentity::new(107),
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
    );
    let new = pointer(&context, UiHostPointerIdentity::new(108), old.target());
    let old_identity = UiMountedAppearanceMechanic::Pointer(old).identity();
    let valid = handoff(&context, &[old_identity.clone()], new, new.pointer()).unwrap();
    for order_in_predecessor in [true, false] {
        let participant = UiOverlayParticipantIdentity::Portal(old.target());
        let manifest = UiMountedAppearancePredecessorManifest::from_runtime_mounting(
            [old_identity.clone()],
            order_in_predecessor.then_some(participant.clone()),
        )
        .unwrap();
        let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
            context.surface,
            context.attempt,
            1,
            1,
            (!order_in_predecessor).then_some(participant),
        )
        .unwrap();
        let successor = UiMountedAppearanceFrame::from_runtime_mounting(
            context.frame,
            context.surface,
            valid.work().successor().mechanics().iter().cloned(),
            order,
        )
        .unwrap();
        let work = UiMountedAppearanceWork::from_runtime_mounting(
            UiMountedAppearanceWorkPosture::Delta,
            Some(context.predecessor),
            Some(manifest),
            successor,
            valid.work().changes().iter().cloned(),
            [],
            true,
        )
        .unwrap();
        assert_eq!(
            UiUnpublishedAppearanceFragment::from_runtime_mounting(
                valid.identity(),
                work,
                [],
                context.requirement,
                surface_affinity(&context),
            ),
            Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch)
        );
    }
}

fn handoff(
    context: &Context,
    predecessors: &[UiMountedAppearanceMechanicIdentity],
    successor: UiMountedPointerAffordanceMechanic,
    fragment_pointer: UiHostPointerIdentity,
) -> Result<UiUnpublishedAppearanceFragment, UiUnpublishedAppearanceFrameProjectionDenial> {
    let successor = UiMountedAppearanceMechanic::Pointer(successor);
    let changes = predecessors
        .iter()
        .cloned()
        .map(UiMountedAppearanceMechanicChange::Remove)
        .chain([UiMountedAppearanceMechanicChange::Insert(successor.clone())]);
    let work = work_with_manifest(
        context,
        UiMountedAppearanceWorkPosture::Delta,
        predecessors.iter().cloned(),
        [successor],
        changes,
        [],
    );
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
            surface: context.surface,
            pointer: fragment_pointer,
        },
        work,
        [],
        context.requirement,
        surface_affinity(context),
    )
}

#[test]
fn pointer_fragment_denies_two_current_pointers_on_one_surface() {
    let context = super::super::context();
    let first_id = UiHostPointerIdentity::new(29);
    let second_id = UiHostPointerIdentity::new(31);
    let first_target = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let second_target = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let first = pointer(&context, first_id, first_target);
    let second = pointer(&context, second_id, second_target);
    let first_identity = UiMountedAppearanceMechanic::Pointer(first).identity();
    let second_identity = UiMountedAppearanceMechanic::Pointer(second).identity();
    let work = work_with_manifest(
        &context,
        UiMountedAppearanceWorkPosture::Unchanged,
        [first_identity, second_identity],
        [
            UiMountedAppearanceMechanic::Pointer(pointer(&context, first_id, first_target)),
            UiMountedAppearanceMechanic::Pointer(pointer(&context, second_id, second_target)),
        ],
        [],
        [],
    );

    assert_eq!(
        UiUnpublishedAppearanceFragment::from_runtime_mounting(
            UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
                surface: context.surface,
                pointer: first_id,
            },
            work,
            [],
            context.requirement,
            surface_affinity(&context),
        ),
        Err(
            UiUnpublishedAppearanceFrameProjectionDenial::MultiplePointerMechanicsForSurface(
                context.surface
            )
        )
    );
}

#[test]
fn pointer_fragments_still_have_one_fragment_per_surface() {
    let context = super::super::context();
    let first = pointer_fragment(&context, UiHostPointerIdentity::new(37));
    let second = pointer_fragment(&context, UiHostPointerIdentity::new(41));

    assert_eq!(
        UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
            context.frame,
            context.attempt,
            [first, second],
        ),
        Err(
            UiUnpublishedAppearanceFrameProjectionDenial::MultiplePointerFragmentsForSurface(
                context.surface
            )
        )
    );
}
