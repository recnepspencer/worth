use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostSurfacePosition, UiMountedNodeReceiptIdentity,
};

use crate::runtime::interaction::targeting::{
    issue_continuity, map_current_affinity_denial, resolve_presented_target,
    UiInteractionTargetingDenial, UiPresentedInteractionTarget,
};

/// Current visual posture of the captured incarnation, separate from the
/// original presented target retained for activation continuity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct UiActivePressedAppearance {
    presentation: UiHostObservationPresentationBasis,
    node_receipt: UiMountedNodeReceiptIdentity,
    inside: bool,
}

impl UiActivePressedAppearance {
    pub(super) fn from_press(target: &UiPresentedInteractionTarget) -> Self {
        Self {
            presentation: target.presentation(),
            node_receipt: target.node_receipt(),
            inside: true,
        }
    }

    pub(super) fn refresh(
        &mut self,
        original: &UiPresentedInteractionTarget,
        presentation: UiHostObservationPresentationBasis,
        position: UiHostSurfacePosition,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Result<bool, UiInteractionTargetingDenial> {
        let node_receipt = mounted
            .current_presented_incarnation_receipt(
                crate::mounting::UiMountedIncarnationAffinityInput {
                    surface: original.surface(),
                    binding: original.binding(),
                    mounted_instance: original.mounted_instance(),
                },
                presentation,
            )
            .map_err(map_current_affinity_denial)?;
        let inside = match resolve_presented_target(mounted, presentation, position) {
            Ok(current) => issue_continuity(original, &current).is_ok(),
            Err(UiInteractionTargetingDenial::NoTarget { .. }) => false,
            Err(denial) => return Err(denial),
        };
        let successor = Self {
            presentation,
            node_receipt,
            inside,
        };
        let changed = *self != successor;
        *self = successor;
        Ok(changed)
    }

    pub(super) fn refresh_evidence(
        &mut self,
        original: &UiPresentedInteractionTarget,
        presentation: UiHostObservationPresentationBasis,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Result<(), UiInteractionTargetingDenial> {
        self.node_receipt = mounted
            .current_presented_incarnation_receipt(
                crate::mounting::UiMountedIncarnationAffinityInput {
                    surface: original.surface(),
                    binding: original.binding(),
                    mounted_instance: original.mounted_instance(),
                },
                presentation,
            )
            .map_err(map_current_affinity_denial)?;
        self.presentation = presentation;
        Ok(())
    }

    pub(super) const fn presentation(self) -> UiHostObservationPresentationBasis {
        self.presentation
    }

    pub(super) const fn node_receipt(self) -> UiMountedNodeReceiptIdentity {
        self.node_receipt
    }

    pub(super) const fn inside(self) -> bool {
        self.inside
    }
}
