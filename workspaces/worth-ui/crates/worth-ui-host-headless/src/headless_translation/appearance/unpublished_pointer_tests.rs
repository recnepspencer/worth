use super::{context, order, Context};
use crate::{
    UiHeadlessAppearanceMechanic as Mechanic, UiHeadlessAppearanceMechanicChange as Change,
};
use worth_ui_host_contract::*;

#[test]
fn pointer_handoff_and_removal_translate_without_node_or_paint_work() {
    // Contract fixture: mounted identities are issued here. This proves the
    // actual unpublished contract-to-headless boundary, not runtime targeting.
    let context = context();
    let old = pointer(&context, 41);
    let new = pointer(&context, 42);
    let old_identity = UiMountedAppearanceMechanic::Pointer(old).identity();
    for successor in [Some(new), None] {
        let fragment = transition(&context, old, successor);
        let projection = UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
            context.frame,
            context.presentation,
            [fragment],
        )
        .unwrap();
        let transcript = crate::translate_unpublished_appearance_for_certification(&projection)
            .expect("a sealed primary handoff reaches the headless boundary");
        assert_eq!(transcript.fragments().len(), 1);
        let fragment = &transcript.fragments()[0];
        assert_eq!(
            fragment.identity(),
            UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
                surface: context.surface,
                pointer: successor.unwrap_or(old).pointer(),
            },
        );
        assert!(fragment.text_candidates().is_empty());
        assert!(fragment
            .presentation_affinity()
            .receipt_affinity()
            .is_none());
        let work = fragment.work();
        assert_eq!(work.predecessor(), Some(context.predecessor));
        assert_eq!(work.successor().frame(), context.frame);
        assert!(work.damage().is_empty());
        assert!(!work.order_changed());
        match successor {
            Some(new) => {
                assert_eq!(work.successor().mechanics(), &[Mechanic::Pointer(new)]);
                assert_eq!(
                    work.changes(),
                    &[
                        Change::Remove(old_identity.clone()),
                        Change::Insert(Mechanic::Pointer(new)),
                    ]
                );
            }
            None => {
                assert!(work.successor().mechanics().is_empty());
                assert_eq!(work.changes(), &[Change::Remove(old_identity.clone())]);
            }
        }
    }
}

fn pointer(context: &Context, id: u64) -> UiMountedPointerAffordanceMechanic {
    UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
        UiHostPointerIdentity::new(id),
        context.surface,
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        UiPointerAffordanceFamily::Activation,
    )
}

fn transition(
    context: &Context,
    old: UiMountedPointerAffordanceMechanic,
    new: Option<UiMountedPointerAffordanceMechanic>,
) -> UiUnpublishedAppearanceFragment {
    let old_identity = UiMountedAppearanceMechanic::Pointer(old).identity();
    let manifest =
        UiMountedAppearancePredecessorManifest::from_runtime_mounting([old_identity.clone()], [])
            .unwrap();
    let frame = UiMountedAppearanceFrame::from_runtime_mounting(
        context.frame,
        context.surface,
        new.map(UiMountedAppearanceMechanic::Pointer),
        order(context),
    )
    .unwrap();
    let mut changes = vec![UiMountedAppearanceMechanicChange::Remove(old_identity)];
    if let Some(new) = new {
        changes.push(UiMountedAppearanceMechanicChange::Insert(
            UiMountedAppearanceMechanic::Pointer(new),
        ));
    }
    let work = UiMountedAppearanceWork::from_runtime_mounting(
        UiMountedAppearanceWorkPosture::Delta,
        Some(context.predecessor),
        Some(manifest),
        frame,
        changes,
        [],
        false,
    )
    .unwrap();
    let affinity = UiMountedPresentationAffinity::from_runtime_mounting(
        Some(context.predecessor),
        context.frame,
        context.requirement,
        context.content,
        None,
    );
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
            surface: context.surface,
            pointer: new.unwrap_or(old).pointer(),
        },
        work,
        [],
        context.requirement,
        affinity,
    )
    .unwrap()
}
