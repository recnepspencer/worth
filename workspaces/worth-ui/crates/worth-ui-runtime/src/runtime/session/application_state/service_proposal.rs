use super::WorthUiApplicationSessionState;

#[path = "service_proposal/focus_reveal.rs"]
mod focus_reveal;
#[path = "service_proposal/portal.rs"]
mod portal;
#[path = "service_proposal/portal_cancellation.rs"]
mod portal_cancellation;
#[path = "service_proposal/portal_frame_binding.rs"]
mod portal_frame_binding;
#[path = "service_proposal/portal_types.rs"]
mod portal_types;
#[path = "service_proposal/scroll_settle.rs"]
mod scroll_settle;
#[path = "service_proposal/scroll_settle_settlement.rs"]
mod scroll_settle_settlement;
#[path = "service_proposal/scroll_settle_types.rs"]
mod scroll_settle_types;
#[path = "service_proposal/settlement.rs"]
mod settlement;
#[path = "service_proposal/terminal.rs"]
mod terminal;

pub(crate) use focus_reveal::{UiFocusRevealStagingDenial, UiStagedFocusReveal};
use portal_types::UiPortalProposalSettlement;
pub(crate) use portal_types::{
    UiIndeterminatePortalProposalTransaction, UiPortalProposalPreparation,
    UiPortalProposalPreparationDenial, UiStagedPortalProposalTransaction,
};
pub(crate) use scroll_settle_types::UiScrollSettlePublicationDenial;

impl WorthUiApplicationSessionState {
    pub(crate) fn service_proposal_resource_counts(&self) -> [u16; 4] {
        let entries = self.runtime.service_proposals.census().entries();
        [entries[0].1, entries[1].1, entries[2].1, entries[3].1]
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn inspect_service_proposal_resources_for_certification(
        &self,
    ) -> ([u16; 4], usize, usize) {
        (
            self.service_proposal_resource_counts(),
            self.runtime.service_proposals.live_occupancy_count(),
            self.runtime.service_proposals.live_cancellation_count(),
        )
    }
}
