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
        [UiAppearanceDamageRegion::new(0, 0, 2, 2).unwrap()],
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
        [UiAppearanceDamageRegion::new(0, 0, 2, 2).unwrap()],
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
