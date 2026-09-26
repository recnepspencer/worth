//! Leading the hit rows a publication commits while a settle is owed.
//!
//! A publication commits fresh hit rows where the frame lays its content
//! out. Rows the frame carries unchanged keep the lead the frame they
//! replace held, but a row the frame republished reads committed geometry.
//! While a settle is owed the host shows each replaced command where its
//! groups' displayed samples stand, so the new frame's rows lead there too:
//! a pointer reaches content where the host draws it, whichever frame
//! committed the row. The publication committed its rows where mounted
//! geometry stands, so this lead measures from there. A group the frame
//! placed stands where the frame publishes it, whatever its sample says, so
//! its rows lead nowhere.

use super::scroll_accepted_sample_settlement::UiScrollSettlementReading;
use super::scroll_settle_hit_lead::UiScrollHitLeadParts;
use super::scroll_settle_retry::UiOwedScrollSettles;
use crate::mounting::UiMountedFrameOutcome;

impl UiScrollHitLeadParts<'_> {
    /// Lead the rows `outcome` published on each surface whose settle is
    /// owed to where the surface's displayed samples stand.
    pub(super) fn lead_published(
        &mut self,
        settles: &UiOwedScrollSettles,
        outcome: &UiMountedFrameOutcome,
    ) {
        let (UiMountedFrameOutcome::Published(publication)
        | UiMountedFrameOutcome::Reconciled(publication)) = outcome
        else {
            return;
        };
        let mut owed = Vec::new();
        publication.with_surface_presentations(|surfaces| {
            owed.extend(
                surfaces
                    .iter()
                    .map(|surface| surface.semantic_surface())
                    .filter(|surface| settles.owes(*surface)),
            );
        });
        for surface in owed {
            self.lead_owed_surface(surface);
        }
    }

    fn lead_owed_surface(&mut self, surface: worth_ui_host_contract::UiSemanticSurfaceIdentity) {
        let Some(displayed) = self.mounted.current_presentation_for_surface(surface) else {
            return;
        };
        let reading = UiScrollSettlementReading {
            mounted: self.mounted,
            scroll: self.scroll,
        };
        let settlements = self
            .mounted
            .accepted_scroll_group_samples()
            .into_iter()
            .filter(|(target, _)| !self.mounted.scroll_group_placed_on_screen(*target))
            .filter_map(|(target, _)| match reading.owner(target, surface) {
                Ok(None) => None,
                Ok(Some(owner)) => Some(
                    reading
                        .displayed_sample(target, surface)
                        .and_then(|sample| owner.settle(sample).ok()),
                ),
                Err(_) => Some(None),
            })
            .collect::<Option<Vec<_>>>();
        let Some(settlements) = settlements else {
            self.retire(surface, displayed);
            return;
        };
        let poses = settlements
            .iter()
            .map(|settlement| {
                (
                    settlement.owner.surface,
                    settlement.owner.owner_instance,
                    settlement.offset,
                )
            })
            .collect::<Vec<_>>();
        // The published rows committed each owner where mounted geometry
        // stands, save those direct input has staged past since.
        let shown = self.staged_under_direct(surface, &[]);
        self.lead(&poses, Some(&shown), surface, displayed);
    }
}
