// Gate 1 deliberately keeps this authority unpublished until a later cutover.

mod backdrop;
mod counters;
mod damage;
mod delta;
mod fact;
mod input;
mod lowering;
mod mechanic_equivalence;
mod opacity_composition;
mod outline;
mod overlay_order;
mod pointer_affordance;
mod reconstruction;
mod surface;
mod text_foreground;

pub(crate) use delta::UiMountedAppearanceDeltaSummary;
pub(crate) use fact::{
    UiMountedAppearanceBackdropInput, UiMountedAppearanceFacts, UiMountedAppearanceLoweringInput,
    UiMountedAppearanceNodeInput, UiMountedAppearanceOutlineInput, UiMountedAppearanceOverlayInput,
    UiMountedAppearancePointerInput, UiMountedAppearanceTextForegroundInput,
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiMountedAppearanceLoweringDenial {
    NodeReceiptFrameMismatch,
    NodeProjectionIssuerMismatch,
    NodeSurfaceMismatch,
    PortalSurfaceMissing,
    PointerTargetMismatch,
    PointerSurfaceMismatch,
    OutlineAllocationMismatch,
    BackdropPlacementMismatch,
    OverlayRevisionMissing,
    OrderParticipantMissing,
    Surface(worth_ui_host_contract::UiMountedSurfaceAppearanceCompletionDenial),
    PortalSurface(worth_ui_host_contract::UiMountedPortalSurfaceAppearanceCompletionDenial),
    Outline(worth_ui_host_contract::UiMountedOutlineAppearanceCompletionDenial),
    TextForeground(worth_ui_host_contract::UiMountedTextForegroundAppearanceCompletionDenial),
    Backdrop(worth_ui_host_contract::UiMountedBackdropCompletionDenial),
    OverlayOrder(worth_ui_host_contract::UiMountedOverlayOrderMechanicDenial),
    Frame(worth_ui_host_contract::UiMountedAppearanceFrameDenial),
    WorkConstruction,
}

#[derive(Clone, Default)]
pub(crate) struct UiMountedAppearanceSidecar {
    current: Option<UiMountedAppearanceFacts>,
    counters: counters::UiMountedAppearanceCounters,
    last_delta: Option<UiMountedAppearanceDeltaSummary>,
}

impl UiMountedAppearanceSidecar {
    pub(crate) fn mount(
        &mut self,
        input: UiMountedAppearanceLoweringInput,
    ) -> Result<worth_ui_host_contract::UiMountedAppearanceWork, UiMountedAppearanceLoweringDenial>
    {
        let successor = lowering::lower(input)?;
        let delta = delta::work(self.current.as_ref(), &successor)?;
        self.counters.observe(&delta.work, delta.summary);
        self.last_delta = Some(delta.summary);
        self.current = Some(successor);
        Ok(delta.work)
    }

    pub(crate) fn reconstruct(
        &mut self,
        input: UiMountedAppearanceLoweringInput,
    ) -> Result<worth_ui_host_contract::UiMountedAppearanceWork, UiMountedAppearanceLoweringDenial>
    {
        reconstruction::rebuild(self.current.as_ref(), input).map(|(delta, facts)| {
            self.counters.observe(&delta.work, delta.summary);
            self.last_delta = Some(delta.summary);
            self.current = Some(facts);
            delta.work
        })
    }

    #[cfg(test)]
    pub(crate) fn current(&self) -> Option<&UiMountedAppearanceFacts> {
        self.current.as_ref()
    }

    #[cfg(test)]
    #[allow(
        dead_code,
        reason = "Gate 1 retains appearance counter evidence for later mounting proofs"
    )]
    pub(crate) const fn counters(&self) -> counters::UiMountedAppearanceCounters {
        self.counters
    }

    #[cfg(test)]
    pub(crate) const fn last_delta(&self) -> Option<UiMountedAppearanceDeltaSummary> {
        self.last_delta
    }
}

#[cfg(test)]
mod tests;
