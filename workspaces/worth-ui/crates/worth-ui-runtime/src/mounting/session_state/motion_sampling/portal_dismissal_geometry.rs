//! What a Portal looks like to a press that may dismiss it: the
//! presentation the press is read at, and the body the host shows there.
use super::super::WorthUiMountedSessionState;

impl WorthUiMountedSessionState {
    /// The presentation at which a press on `admitted` may be classified
    /// against one Portal. A later publication on the same physical surface
    /// can supersede `admitted` while the host still holds the press, and the
    /// press still names what the reader saw. It is read at the current
    /// presentation only when that Portal looked the same in both: the
    /// admitted frame showed the same overlay geometry, and no Motion sample
    /// of the Portal reached the screen after the press. Otherwise what the
    /// reader saw is unknown and the press stays stale.
    pub(crate) fn portal_dismissal_presentation(
        &self,
        admitted: worth_ui_host_contract::UiHostObservationPresentationBasis,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> Result<
        worth_ui_host_contract::UiHostObservationPresentationBasis,
        crate::mounting::UiPresentedFrameBasisDenial,
    > {
        use crate::mounting::UiPresentedFrameBasisDenial as Denial;
        if self
            .current_semantic_surface_for_presentation(admitted)
            .is_ok()
        {
            return Ok(admitted);
        }
        if self
            .presentation
            .binding_requires_reconstruction(admitted.binding())
        {
            return Err(Denial::PresentationTruthUnavailable);
        }
        let current = self
            .current_surface_for_binding(admitted.binding())
            .and_then(|surface| {
                self.current_presentation_for_surface(surface)
                    .map(|displayed| displayed.basis())
            })
            .filter(|current| {
                current.host_surface() == admitted.host_surface()
                    && current.epoch() > admitted.epoch()
            })
            .ok_or(Denial::Expired)?;
        let seen = self.retention.presented_portal_overlay(admitted, target)?;
        let shown = self.retention.presented_portal_overlay(current, target)?;
        let unchanged = match (seen, shown) {
            (Some(seen), Some(shown)) => same_portal_geometry(seen, shown),
            _ => false,
        };
        if !unchanged
            || self
                .motion_sampling
                .target_presented_after(target, admitted)
        {
            return Err(Denial::Expired);
        }
        Ok(current)
    }

    /// The Portal body `target` names as the host shows it at
    /// `presentation`: the committed body moved by its on-screen Motion
    /// sample. `None` when no sample on screen moves it, and so the committed
    /// placement stands. That includes a sample whose base has no area or
    /// that carries the body out of finite geometry, where hit testing places
    /// the Portal's rows nowhere.
    pub(crate) fn committed_motion_geometry_for_target(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<
        Option<crate::mounting::presentation::UiDisplayedRect>,
        crate::mounting::UiPresentedFrameBasisDenial,
    > {
        if self
            .presentation
            .binding_requires_reconstruction(presentation.binding())
        {
            return Err(crate::mounting::UiPresentedFrameBasisDenial::PresentationTruthUnavailable);
        }
        self.current_semantic_surface_for_presentation(presentation)?;
        let hit_test = self
            .retention
            .interaction_hit_test_basis(presentation, None)?;
        if !hit_test
            .rows()
            .iter()
            .any(|row| row.mounted_instance() == target.mounted_instance())
        {
            return Err(crate::mounting::UiPresentedFrameBasisDenial::Unknown);
        }
        let Some(map) = self
            .motion_sampling
            .current_sample_for_target(target, presentation)
            .and_then(|sample| sample.base_map())
        else {
            return Ok(None);
        };
        // The sample moves whatever the frame laid out in its base, as it
        // moves the Portal's paint and hit rows. A frame that moved the Portal
        // since the sample was taken committed a body elsewhere, and the
        // sample carries that body, not the one it was taken of.
        let Some(body) = self
            .retention
            .presented_portal_body(presentation, target)?
            .and_then(|body| map.apply(body))
        else {
            return Ok(None);
        };
        crate::mounting::presentation::UiDisplayedRect::displayed(body, hit_test.displayed())
            .map(Some)
            .map_err(|_| crate::mounting::UiPresentedFrameBasisDenial::Unknown)
    }
}

/// Everything a Portal dismissal reads from a presented overlay. Frame and
/// receipt identities differ across publications that leave it in place.
fn same_portal_geometry(
    seen: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
    shown: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
) -> bool {
    seen.surface() == shown.surface()
        && seen.owner() == shown.owner()
        && seen.portal_identity() == shown.portal_identity()
        && seen.anchor_bounds() == shown.anchor_bounds()
        && seen.bounds() == shown.bounds()
        && seen.clip_bounds() == shown.clip_bounds()
        && seen.lifecycle() == shown.lifecycle()
        && seen.shielding() == shown.shielding()
}
