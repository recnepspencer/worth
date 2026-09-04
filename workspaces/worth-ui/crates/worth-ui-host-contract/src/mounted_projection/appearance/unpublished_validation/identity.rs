use super::super::{
    UiUnpublishedAppearanceFragment, UiUnpublishedAppearanceFragmentIdentity,
    UiUnpublishedAppearanceFrameProjectionDenial,
};
use crate::{
    UiMountedAppearanceMechanic, UiMountedAppearanceMechanicChange,
    UiMountedAppearanceMechanicIdentity,
};

pub(super) fn validate(
    fragment: &UiUnpublishedAppearanceFragment,
    frame: crate::UiMountedFrameIdentity,
    surface: crate::UiSemanticSurfaceIdentity,
) -> Result<(), UiUnpublishedAppearanceFrameProjectionDenial> {
    let mechanics = fragment.work.successor().mechanics();
    let changes = fragment.work.changes();
    match fragment.identity {
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            predecessor,
            successor,
        } => validate_node(fragment, frame, predecessor, successor, mechanics, changes),
        UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(identity_surface) => {
            validate_overlay(fragment, surface, identity_surface, mechanics, changes)
        }
        UiUnpublishedAppearanceFragmentIdentity::SurfacePointer {
            surface: identity_surface,
            pointer,
        } => validate_pointer(surface, identity_surface, pointer, mechanics, changes),
    }
}

fn validate_node(
    fragment: &UiUnpublishedAppearanceFragment,
    frame: crate::UiMountedFrameIdentity,
    predecessor: Option<crate::UiMountedNodeReceiptIdentity>,
    successor: Option<crate::UiMountedNodeReceiptIdentity>,
    mechanics: &[UiMountedAppearanceMechanic],
    changes: &[UiMountedAppearanceMechanicChange],
) -> Result<(), UiUnpublishedAppearanceFrameProjectionDenial> {
    if predecessor.is_none() && successor.is_none() {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::NodeReceiptAttributionMissing);
    }
    if predecessor
        .zip(successor)
        .is_some_and(|(before, after)| before.mounted_instance() != after.mounted_instance())
    {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::NodeReceiptInstanceMismatch);
    }
    if let Some(receipt) = predecessor {
        if fragment.work.predecessor() != Some(receipt.frame()) {
            return Err(
                UiUnpublishedAppearanceFrameProjectionDenial::NodeReceiptPredecessorFrameMismatch,
            );
        }
        let Some(manifest) = fragment.work.predecessor_manifest() else {
            return Err(
                UiUnpublishedAppearanceFrameProjectionDenial::NodeReceiptPredecessorMismatch,
            );
        };
        if !manifest
            .mechanic_identities()
            .iter()
            .any(|identity| identity_belongs_to_node(identity, receipt))
        {
            return Err(
                UiUnpublishedAppearanceFrameProjectionDenial::NodeReceiptPredecessorMismatch,
            );
        }
    }
    if let Some(receipt) = successor {
        if receipt.frame() != frame {
            return Err(
                UiUnpublishedAppearanceFrameProjectionDenial::NodeReceiptSuccessorFrameMismatch,
            );
        }
        if !mechanics
            .iter()
            .any(|mechanic| mechanic_belongs_to_node(mechanic, receipt))
        {
            return Err(UiUnpublishedAppearanceFrameProjectionDenial::NodeReceiptSuccessorMismatch);
        }
        if fragment.presentation_affinity.receipt_affinity()
            != Some(crate::UiMountedNodeReceiptAffinity::from_receipt(receipt))
        {
            return Err(
                UiUnpublishedAppearanceFrameProjectionDenial::PresentationReceiptAffinityMismatch,
            );
        }
    } else if !mechanics.is_empty() {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::NodeReceiptSuccessorMismatch);
    }
    if mechanics.iter().any(|mechanic| {
        !successor.is_some_and(|receipt| mechanic_belongs_to_node(mechanic, receipt))
    }) {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch);
    }
    for change in changes {
        if !node_change_matches(fragment, change, predecessor, successor) {
            return Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch);
        }
    }
    if successor.is_none()
        && (changes.is_empty()
            || changes
                .iter()
                .any(|change| !matches!(change, UiMountedAppearanceMechanicChange::Remove(_))))
    {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::NodeReceiptPredecessorMismatch);
    }
    Ok(())
}

fn node_change_matches(
    fragment: &UiUnpublishedAppearanceFragment,
    change: &UiMountedAppearanceMechanicChange,
    predecessor: Option<crate::UiMountedNodeReceiptIdentity>,
    successor: Option<crate::UiMountedNodeReceiptIdentity>,
) -> bool {
    let predecessor_identity_is_present = |identity: &UiMountedAppearanceMechanicIdentity| {
        predecessor.is_some_and(|receipt| identity_belongs_to_node(identity, receipt))
            && fragment
                .work
                .predecessor_manifest()
                .is_some_and(|manifest| manifest.mechanic_identities().contains(identity))
    };
    match change {
        UiMountedAppearanceMechanicChange::Insert(mechanic) => {
            successor.is_some_and(|receipt| mechanic_belongs_to_node(mechanic, receipt))
        }
        UiMountedAppearanceMechanicChange::Replace {
            predecessor: old,
            successor: new,
        } => {
            predecessor_identity_is_present(old)
                && successor.is_some_and(|receipt| mechanic_belongs_to_node(new, receipt))
        }
        UiMountedAppearanceMechanicChange::Remove(old) => predecessor_identity_is_present(old),
    }
}

fn validate_overlay(
    fragment: &UiUnpublishedAppearanceFragment,
    surface: crate::UiSemanticSurfaceIdentity,
    identity_surface: crate::UiSemanticSurfaceIdentity,
    mechanics: &[UiMountedAppearanceMechanic],
    changes: &[UiMountedAppearanceMechanicChange],
) -> Result<(), UiUnpublishedAppearanceFrameProjectionDenial> {
    if identity_surface != surface {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch);
    }
    if !mechanics.iter().any(is_overlay_mechanic) && !changes.iter().any(change_belongs_to_overlay)
    {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch);
    }
    if mechanics
        .iter()
        .any(|mechanic| !is_overlay_mechanic(mechanic))
        || changes
            .iter()
            .any(|change| !change_belongs_to_overlay(change))
        || !overlay_participants_match(fragment)
    {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch);
    }
    Ok(())
}

fn validate_pointer(
    surface: crate::UiSemanticSurfaceIdentity,
    identity_surface: crate::UiSemanticSurfaceIdentity,
    pointer: crate::UiHostPointerIdentity,
    mechanics: &[UiMountedAppearanceMechanic],
    changes: &[UiMountedAppearanceMechanicChange],
) -> Result<(), UiUnpublishedAppearanceFrameProjectionDenial> {
    if identity_surface != surface {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch);
    }
    if !mechanics
        .iter()
        .any(|mechanic| pointer_matches(mechanic, surface, pointer))
        && !changes
            .iter()
            .any(|change| change_belongs_to_pointer(change, surface, pointer))
    {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch);
    }
    if mechanics
        .iter()
        .filter(|mechanic| matches!(mechanic, UiMountedAppearanceMechanic::Pointer(candidate) if candidate.surface() == surface))
        .count()
        > 1
    {
        return Err(
            UiUnpublishedAppearanceFrameProjectionDenial::MultiplePointerMechanicsForSurface(
                surface,
            ),
        );
    }
    if mechanics
        .iter()
        .any(|mechanic| !pointer_matches(mechanic, surface, pointer))
        || changes
            .iter()
            .any(|change| !change_belongs_to_pointer(change, surface, pointer))
    {
        return Err(UiUnpublishedAppearanceFrameProjectionDenial::FragmentIdentityMismatch);
    }
    Ok(())
}

fn mechanic_belongs_to_node(
    mechanic: &UiMountedAppearanceMechanic,
    receipt: crate::UiMountedNodeReceiptIdentity,
) -> bool {
    match mechanic {
        UiMountedAppearanceMechanic::Surface(mechanic) => mechanic.node_receipt() == receipt,
        UiMountedAppearanceMechanic::PortalSurface(_) => false,
        UiMountedAppearanceMechanic::Outline(mechanic) => mechanic.node_receipt() == receipt,
        UiMountedAppearanceMechanic::TextForeground(mechanic) => mechanic.node_receipt() == receipt,
        UiMountedAppearanceMechanic::Pointer(_) | UiMountedAppearanceMechanic::Backdrop(_) => false,
    }
}

fn identity_belongs_to_node(
    identity: &UiMountedAppearanceMechanicIdentity,
    receipt: crate::UiMountedNodeReceiptIdentity,
) -> bool {
    match identity {
        UiMountedAppearanceMechanicIdentity::Surface(instance)
        | UiMountedAppearanceMechanicIdentity::Outline(instance) => {
            *instance == receipt.mounted_instance()
        }
        UiMountedAppearanceMechanicIdentity::TextForeground { target, .. } => {
            *target == receipt.mounted_instance()
        }
        UiMountedAppearanceMechanicIdentity::PortalSurface(_)
        | UiMountedAppearanceMechanicIdentity::Pointer { .. }
        | UiMountedAppearanceMechanicIdentity::Backdrop(_) => false,
    }
}

fn is_overlay_mechanic(mechanic: &UiMountedAppearanceMechanic) -> bool {
    matches!(
        mechanic,
        UiMountedAppearanceMechanic::PortalSurface(_) | UiMountedAppearanceMechanic::Backdrop(_)
    )
}

fn change_belongs_to_overlay(change: &UiMountedAppearanceMechanicChange) -> bool {
    change.successor().is_none_or(is_overlay_mechanic)
        && change.identity().is_none_or(|identity| {
            matches!(
                identity,
                UiMountedAppearanceMechanicIdentity::PortalSurface(_)
                    | UiMountedAppearanceMechanicIdentity::Backdrop(_)
            )
        })
}

fn overlay_participants_match(fragment: &UiUnpublishedAppearanceFragment) -> bool {
    fragment
        .work
        .successor()
        .overlay_order()
        .bottom_to_top()
        .iter()
        .all(|participant| match participant {
            crate::UiOverlayParticipantIdentity::Portal(instance) => fragment
                .work
                .successor()
                .mechanics()
                .iter()
                .any(|mechanic| matches!(mechanic, UiMountedAppearanceMechanic::PortalSurface(portal) if portal.portal_instance() == *instance)),
            crate::UiOverlayParticipantIdentity::Backdrop(identity) => fragment
                .work
                .successor()
                .mechanics()
                .iter()
                .any(|mechanic| matches!(mechanic, UiMountedAppearanceMechanic::Backdrop(backdrop) if backdrop.identity() == identity)),
        })
}

fn pointer_matches(
    mechanic: &UiMountedAppearanceMechanic,
    surface: crate::UiSemanticSurfaceIdentity,
    pointer: crate::UiHostPointerIdentity,
) -> bool {
    matches!(mechanic, UiMountedAppearanceMechanic::Pointer(candidate)
        if candidate.surface() == surface && candidate.pointer() == pointer)
}

fn change_belongs_to_pointer(
    change: &UiMountedAppearanceMechanicChange,
    surface: crate::UiSemanticSurfaceIdentity,
    pointer: crate::UiHostPointerIdentity,
) -> bool {
    change
        .successor()
        .is_none_or(|mechanic| pointer_matches(mechanic, surface, pointer))
        && change.identity().is_none_or(|identity| {
            matches!(identity, UiMountedAppearanceMechanicIdentity::Pointer {
                pointer: candidate,
                surface: candidate_surface,
                ..
            } if *candidate == pointer && *candidate_surface == surface)
        })
}
