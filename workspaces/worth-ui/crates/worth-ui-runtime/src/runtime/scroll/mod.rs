pub(crate) mod allocation;
mod anchor;
pub(crate) mod chrome;
mod host_observation;
mod identity;
mod inspection;
mod model;
mod ownership_chain;
mod programmatic_reveal;
mod proposal;
mod receipt;
mod request;
mod routing;
#[cfg(feature = "certification-support")]
mod scale_certification;
mod settle_proposal;
mod settle_stop;
mod settle_transition;
mod shared_owner_reconciliation;
mod state;
pub(crate) mod transition;

pub(crate) use anchor::{
    UiScrollAnchor, UiScrollAnchorIdentity, UiScrollAnchorPolicy,
    UiScrollAnchorReconciliationOutcome, UiScrollAnchorReconciliationReceipt,
    UiScrollRebindRequest,
};
pub(crate) use host_observation::{
    UiHostScrollObservationDenial, UiHostScrollObservationOutcome, UiScrollBoundsResolutionDenial,
};
pub(crate) use identity::{
    UiScrollOwnerIdentity, UiScrollOwnerIncarnation, UiScrollOwnerRegistration,
};
pub(crate) use inspection::UiScrollOwnerInspectionRecord;
pub(crate) use model::{UiScrollAxes, UiScrollBounds, UiScrollDelta, UiScrollOffset};
pub(crate) use ownership_chain::{
    UiResolvedScrollOwnershipChain, UiScrollOwnershipResolutionDenial,
};
pub(crate) use programmatic_reveal::{
    UiScrollProgrammaticRevealRequest, UiScrollRevealInterval, UiScrollRevealTarget,
    UiScrollViewportExtent,
};
pub(in crate::runtime) use proposal::UiStagedScrollServiceProposal;
pub(crate) use receipt::{
    UiScrollChainTransition, UiScrollCounters, UiScrollRouteDenial, UiScrollRouteReceipt,
};
pub(crate) use request::{UiScrollChainEntry, UiScrollDeltaCause, UiScrollDeltaRequest};
#[cfg(feature = "certification-support")]
pub(crate) use scale_certification::scroll_scale_evidence;
pub(in crate::runtime) use settle_proposal::UiStagedScrollSettleProposal;
pub use settle_stop::UiScrollSettleStop;
pub(crate) use settle_transition::{
    UiPreparedScrollSettleTransition, UiScrollSettleTransitionDenial,
};
pub(crate) use shared_owner_reconciliation::UiSharedScrollOwnerReconciliation;
pub(crate) use state::UiScrollRuntimeState;

#[cfg(test)]
mod state_rebind_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod state_transition_tests;
