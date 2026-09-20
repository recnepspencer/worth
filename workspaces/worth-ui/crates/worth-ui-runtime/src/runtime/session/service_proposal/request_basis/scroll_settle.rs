//! The authority a Scroll owner publishes one settle transition under.
//!
//! A settle is not an admitted intent and not a Portal transition: it is the
//! consequence of a host wheel observation that the Scroll owner already routed.
//! Its request identity is the revision of that route, so every notch carries
//! its own proposal identity and no two settles of one owner can be mistaken
//! for the same request.

use core::num::NonZeroU64;

#[derive(Debug)]
pub(in crate::runtime) struct UiScrollSettleServiceRequestAuthority;

impl super::sealed::Sealed for UiScrollSettleServiceRequestAuthority {}

impl super::UiServiceRequestOriginAuthority for UiScrollSettleServiceRequestAuthority {
    fn service_request_origin(&self) -> super::UiServiceRequestOrigin {
        super::UiServiceRequestOrigin::HostObservation
    }
}

impl super::UiServiceRequestBasis<UiScrollSettleServiceRequestAuthority> {
    /// The request one prepared Scroll settle transition is published under.
    /// `presentation` is the basis of the frame that is currently published, so
    /// the surface axes of the request are the ones the settle will submit into.
    pub(in crate::runtime) fn from_scroll_settle(
        transition: &crate::runtime::scroll::UiPreparedScrollSettleTransition,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        application: crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<Self, super::UiServiceRequestBasisDenial> {
        let issued: NonZeroU64 = transition.request_lineage();
        let identity = super::UiServiceRequestIdentity(issued);
        Self::seal(super::UiServiceRequestBasisInput {
            identity,
            causal_parent: None,
            causal_root: identity,
            application,
            surface: super::UiServiceSurfaceBasis {
                semantic_surface: transition.semantic_surface(),
                host_surface: presentation.host_surface(),
                binding: presentation.binding(),
            },
            presentation: Some(presentation),
            source_order: super::UiServiceSourceOrder(issued),
            cancellation: super::UiServiceCancellationIdentity(issued),
            resource_budget: super::UiServiceResourceBudgetIdentity(issued),
            authority: UiScrollSettleServiceRequestAuthority,
        })
    }
}
