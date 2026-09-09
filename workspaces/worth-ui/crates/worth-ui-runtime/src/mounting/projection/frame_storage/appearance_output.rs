use worth_ui_host_contract::{
    UiMountedAppearanceWork, UiMountedNodeReceiptIdentity, UiMountedPresentationAffinity,
    UiMountedPresentationAttemptIdentity, UiMountedSurfaceBindingRequirement,
    UiUnpublishedAppearanceFragment, UiUnpublishedAppearanceFragmentIdentity,
    UiUnpublishedAppearanceFrameProjection, UiUnpublishedAppearanceFrameProjectionDenial,
};

use super::UiMountedProjectionFrame;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedAppearanceOutputDenial {
    NodeLowering,
    Order(super::UiMountedAppearanceOrderDenial),
    PointerLowering,
    HostGeometryProfileUnavailable,
    HostGeometrySurfaceUnavailable,
    HostGeometryScale(worth_ui_host_contract::UiHostAppearanceScaleDenial),
    AncestorClip(super::super::appearance::UiMountedAppearanceClipDenial),
    CurrentProjectionUnavailable,
    TextCandidate(super::super::UiMountedProjectionDenial),
    Transport(UiUnpublishedAppearanceFrameProjectionDenial),
}

#[derive(Clone)]
pub(super) struct UiMountedAppearanceNodeWork {
    pub(super) predecessor: Option<UiMountedNodeReceiptIdentity>,
    pub(super) successor: Option<UiMountedNodeReceiptIdentity>,
    pub(super) work: UiMountedAppearanceWork,
}

#[derive(Clone)]
pub(super) struct UiMountedAppearanceOverlayWork {
    pub(super) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(super) work: UiMountedAppearanceWork,
}

pub(super) fn assemble(
    frame: &UiMountedProjectionFrame,
    presentation: UiMountedPresentationAttemptIdentity,
    bindings: &[UiMountedSurfaceBindingRequirement],
    nodes: Vec<UiMountedAppearanceNodeWork>,
    overlays: Vec<UiMountedAppearanceOverlayWork>,
    pointers: &super::super::UiMountedPointerAffordanceState,
) -> Result<
    (
        Option<UiUnpublishedAppearanceFrameProjection>,
        super::super::UiMountedPointerAffordanceState,
    ),
    UiMountedAppearanceOutputDenial,
> {
    let bindings = bindings
        .iter()
        .map(|binding| (binding.semantic_surface(), *binding))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut fragments = Vec::with_capacity(nodes.len());
    for node in nodes {
        if node.work.successor().mechanics().is_empty() && node.work.changes().is_empty() {
            continue;
        }
        let binding = bindings
            .get(&node.work.successor().semantic_surface())
            .copied()
            .ok_or(UiMountedAppearanceOutputDenial::Transport(
                UiUnpublishedAppearanceFrameProjectionDenial::SurfaceBindingMismatch,
            ))?;
        let affinity = UiMountedPresentationAffinity::from_runtime_mounting(
            node.work.predecessor(),
            frame.frame_identity(),
            binding,
            frame.content_generation(),
            frame.node_receipt_affinity(),
        );
        let text_candidates = if node.work.successor().mechanics().iter().any(|mechanic| {
            matches!(
                mechanic,
                worth_ui_host_contract::UiMountedAppearanceMechanic::TextForeground(_)
            )
        }) {
            let receipt = node
                .successor
                .ok_or(UiMountedAppearanceOutputDenial::CurrentProjectionUnavailable)?;
            frame.appearance_text_candidates(receipt.mounted_instance())?
        } else {
            Vec::new()
        };
        fragments.push(
            UiUnpublishedAppearanceFragment::from_runtime_mounting(
                UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                    predecessor: node.predecessor,
                    successor: node
                        .successor
                        .filter(|_| !node.work.successor().mechanics().is_empty()),
                },
                node.work,
                text_candidates,
                binding,
                affinity,
            )
            .map_err(UiMountedAppearanceOutputDenial::Transport)?,
        );
    }
    for overlay in overlays {
        if overlay.work.successor().mechanics().is_empty()
            && overlay.work.changes().is_empty()
            && !overlay.work.order_changed()
        {
            continue;
        }
        let binding = bindings.get(&overlay.surface).copied().ok_or(
            UiMountedAppearanceOutputDenial::Transport(
                UiUnpublishedAppearanceFrameProjectionDenial::SurfaceBindingMismatch,
            ),
        )?;
        let affinity = UiMountedPresentationAffinity::from_runtime_mounting(
            overlay.work.predecessor(),
            frame.frame_identity(),
            binding,
            frame.content_generation(),
            None,
        );
        fragments.push(
            UiUnpublishedAppearanceFragment::from_runtime_mounting(
                UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(overlay.surface),
                overlay.work,
                [],
                binding,
                affinity,
            )
            .map_err(UiMountedAppearanceOutputDenial::Transport)?,
        );
    }
    let (pointers, pointer_fragments) = pointers.lower(frame, presentation, &bindings)?;
    fragments.extend(pointer_fragments);
    if fragments.is_empty() {
        return Ok((None, pointers));
    }
    UiUnpublishedAppearanceFrameProjection::from_runtime_mounting(
        frame.frame_identity(),
        presentation,
        fragments,
    )
    .map(|projection| (Some(projection), pointers))
    .map_err(UiMountedAppearanceOutputDenial::Transport)
}
