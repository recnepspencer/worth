//! Leading the hit rows of a surface whose settle is owed to where the host
//! shows the accepted sample, and returning them once nothing is owed.
//!
//! A sample's lead reads the rows of the frame the host shows, which
//! committed each owner at its accepted offset. Mounted geometry can be
//! staged past that frame, by direct input or a layout no witness has shown,
//! so that lead measures from the accepted offsets, never from the staged
//! poses.

use super::scroll_accepted_sample_settlement::UiAcceptedScrollSettlement;
use crate::mounting::presentation::motion_sampling::UiPresentationMotionPresentedSurface;
use crate::mounting::presentation::{UiDisplayedScrollOffset, UiDisplayedSurfaceBasis};
use crate::mounting::WorthUiMountedSessionState;
use crate::runtime::interaction::UiInteractionRuntimeState;
use crate::runtime::scroll::UiScrollRuntimeState;
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

/// Where the rows of a frame committed one owner's offset.
pub(super) type UiShownScrollOffset = (
    UiSemanticSurfaceIdentity,
    UiMountedInstanceIdentity,
    crate::runtime::scroll::UiScrollOffset,
);

/// What leading hit rows borrows: mounted geometry to lead, Scroll to
/// measure from, and interaction to observe each crossing. A publication
/// path that lends the session out in parts can lead as well as the session
/// can.
pub(super) struct UiScrollHitLeadParts<'a> {
    pub(super) mounted: &'a mut WorthUiMountedSessionState,
    pub(super) scroll: Option<&'a UiScrollRuntimeState>,
    pub(super) interaction: &'a mut UiInteractionRuntimeState,
}

impl UiScrollHitLeadParts<'_> {
    /// Lead the hit rows of `surface`, displayed as `displayed`, to the
    /// offsets `poses` settle at, where the host shows the content before the
    /// owed settle is paid. Each row moves from where `shown` says its frame
    /// committed the owner, or from mounted geometry for an owner `shown`
    /// does not list; `None` when that is unknown. Rows no pose can lead
    /// stand where committed geometry put them.
    pub(super) fn lead(
        &mut self,
        poses: &[(
            UiSemanticSurfaceIdentity,
            UiMountedInstanceIdentity,
            UiDisplayedScrollOffset,
        )],
        shown: Option<&[UiShownScrollOffset]>,
        surface: UiSemanticSurfaceIdentity,
        displayed: UiDisplayedSurfaceBasis,
    ) {
        let led = shown.map(|shown| self.mounted.lead_presented_scroll_hits(poses, shown));
        let led = match led {
            Some(Ok(transitions)) if !poses.is_empty() => transitions,
            Some(Ok(_) | Err(_)) | None => {
                self.retire(surface, displayed);
                return;
            }
        };
        for transition in led.iter() {
            self.interaction
                .observe_presented_hit_transition(transition, self.mounted);
        }
    }

    /// Where the rows of the frame the host shows `surface` in committed
    /// each owner `settlements` moves, and each owner direct input has
    /// staged past: their accepted offsets. `None` when an owner a sample
    /// moves has no accepted offset for a lead to measure from.
    fn accepted_scroll_offsets(
        &self,
        settlements: &[UiAcceptedScrollSettlement],
        surface: UiSemanticSurfaceIdentity,
    ) -> Option<Vec<UiShownScrollOffset>> {
        let scroll = self.scroll?;
        let mut shown = settlements
            .iter()
            .map(|settlement| {
                let owner = settlement.owner;
                scroll
                    .offset(owner.entry.owner(), owner.entry.incarnation())
                    .ok()
                    .map(|offset| (owner.surface, owner.owner_instance, offset))
            })
            .collect::<Option<Vec<_>>>()?;
        shown.extend(self.staged_under_direct(surface, &shown));
        Some(shown)
    }

    /// Where the rows of the frame the host shows `surface` in committed
    /// each owner direct input has staged past, other than those `listed`:
    /// their accepted offsets.
    pub(super) fn staged_under_direct(
        &self,
        surface: UiSemanticSurfaceIdentity,
        listed: &[UiShownScrollOffset],
    ) -> Vec<UiShownScrollOffset> {
        self.scroll
            .into_iter()
            .flat_map(|scroll| scroll.accepted_offsets_under_direct(surface))
            .filter(|(owner, _)| {
                !listed
                    .iter()
                    .any(|(sampled, moved, _)| *sampled == surface && moved == owner)
            })
            .map(|(owner, offset)| (surface, owner, offset))
            .collect()
    }

    /// Return the hit rows of `surface`, displayed as `displayed`, to
    /// committed geometry.
    pub(super) fn retire(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
        displayed: UiDisplayedSurfaceBasis,
    ) {
        let retired = self
            .mounted
            .retire_presented_scroll_leads(surface, displayed);
        for transition in retired.iter() {
            self.interaction
                .observe_presented_hit_transition(transition, self.mounted);
        }
    }
}

impl super::super::WorthUiActiveApplicationSession {
    fn scroll_hit_lead_parts(&mut self) -> UiScrollHitLeadParts<'_> {
        UiScrollHitLeadParts {
            mounted: &mut self.mounted,
            scroll: self.scroll.as_ref(),
            interaction: &mut self.interaction,
        }
    }

    /// Lead the presented surface's hit rows to the offsets `poses` settle
    /// at, where the host shows the content before the owed settle is paid.
    pub(super) fn lead_scroll_hits(
        &mut self,
        settlements: &[UiAcceptedScrollSettlement],
        poses: &[(
            UiSemanticSurfaceIdentity,
            UiMountedInstanceIdentity,
            UiDisplayedScrollOffset,
        )],
        presented: UiPresentationMotionPresentedSurface,
    ) {
        let mut parts = self.scroll_hit_lead_parts();
        let surface = presented.semantic_surface();
        let shown = parts.accepted_scroll_offsets(settlements, surface);
        parts.lead(poses, shown.as_deref(), surface, presented.displayed());
    }

    /// Return the presented surface's hit rows to committed geometry.
    pub(super) fn retire_scroll_hit_leads(
        &mut self,
        presented: UiPresentationMotionPresentedSurface,
    ) {
        self.scroll_hit_lead_parts()
            .retire(presented.semantic_surface(), presented.displayed());
    }
}
