//! Which mounted occurrence a host scroll delta addresses.
//!
//! The host reports where the gesture landed, not what owns scrolling there.
//! Turning one into the other is a read against the current presentation: the
//! frame the reader aimed at has to still be the frame on screen, or the
//! coordinate names a layout nobody is looking at any more.
//!
//! Only the first event of a gesture asks this question. Once an owner has the
//! gesture, [`super::scroll_gesture_latching`] answers instead.

use super::super::WorthUiActiveApplicationSession;

use crate::runtime::scroll::UiHostScrollObservationDenial;

impl WorthUiActiveApplicationSession {
    pub(super) fn resolve_scroll_target(
        &self,
        target: worth_ui_host_contract::UiHostScrollDeltaTargetAffinity,
        work: &mut crate::mounting::UiHitTestSpatialWork,
    ) -> Result<
        (
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::mounting::UiMountedIdentityBasis,
        ),
        UiHostScrollObservationDenial,
    > {
        match target {
            worth_ui_host_contract::UiHostScrollDeltaTargetAffinity::ExactCoordinate {
                presentation,
                position,
            } => {
                crate::runtime::interaction::targeting::require_current_presentation(
                    &self.mounted,
                    presentation,
                )
                .map_err(UiHostScrollObservationDenial::Targeting)?;
                let target = crate::runtime::interaction::targeting::resolve_presented_target(
                    &self.mounted,
                    presentation,
                    position,
                    work,
                )
                .map_err(UiHostScrollObservationDenial::Targeting)?;
                // Content that travels with a Scroll region is laid out relative
                // to the region owner, not mounted beneath it, so a wheel over
                // that content addresses the region that scrolls it.
                let hit = target.view().mounted_instance();
                let mounted = self.mounted.scrolled_content_owner(hit).unwrap_or(hit);
                self.mounted
                    .current_mounted_identity_basis(mounted)
                    .map(|basis| (mounted, basis))
                    .ok_or(UiHostScrollObservationDenial::MountedBasisUnavailable)
            }
            worth_ui_host_contract::UiHostScrollDeltaTargetAffinity::ExactMountedTarget {
                presentation,
                mounted,
            } => {
                crate::runtime::interaction::targeting::require_current_presentation(
                    &self.mounted,
                    presentation,
                )
                .map_err(UiHostScrollObservationDenial::Targeting)?;
                let surface = self
                    .mounted
                    .current_surface_for_binding(presentation.binding())
                    .ok_or(UiHostScrollObservationDenial::MountedBasisUnavailable)?;
                self.mounted
                    .admit_current_interaction_affinity(
                        crate::mounting::UiMountedInteractionAffinityInput {
                            surface,
                            binding: presentation.binding(),
                            mounted_instance: mounted.instance(),
                            node_receipt: mounted.node_receipt(),
                        },
                    )
                    .map_err(|denial| {
                        UiHostScrollObservationDenial::Targeting(
                            crate::runtime::interaction::targeting::map_current_affinity_denial(
                                denial,
                            ),
                        )
                    })?;
                self.mounted
                    .current_mounted_identity_basis(mounted.instance())
                    .map(|basis| (mounted.instance(), basis))
                    .ok_or(UiHostScrollObservationDenial::MountedBasisUnavailable)
            }
            worth_ui_host_contract::UiHostScrollDeltaTargetAffinity::PresentedSurfaceFallback {
                ..
            } => Err(UiHostScrollObservationDenial::PresentedSurfaceFallbackIsAmbiguous),
        }
    }
}
