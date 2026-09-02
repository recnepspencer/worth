pub(crate) mod anchored_allocation;
mod capacity;
mod dismissal;
mod identity;
mod inspection;
mod lifecycle;
mod overlay_binding_export;
mod placement;
#[cfg(test)]
mod placement_tests;
mod planning;
mod proposal;
mod rebind;
mod receipt;
mod request;
#[cfg(feature = "certification-support")]
mod scale_certification;
mod stack_ordinal;
mod stack_snapshot;
mod state;
mod transition;

#[cfg(test)]
mod state_dismissal_tests;
#[cfg(test)]
mod state_lifecycle_tests;
#[cfg(test)]
mod state_rebind_tests;
#[cfg(test)]
mod state_retention_tests;
#[cfg(test)]
mod state_tests;
#[cfg(test)]
mod test_support;

pub(crate) use dismissal::UiPortalDismissalIgnoreReason;
pub(crate) use dismissal::{UiPortalDismissalPreparation, UiPortalDismissalTrigger};
pub(crate) use identity::{UiPortalIdentity, UiPortalOwnerIdentity};
pub(crate) use inspection::UiPortalClosedInspectionRecord;
pub(crate) use lifecycle::{
    UiPortalDismissalCause, UiPortalInputShielding, UiPortalLifecyclePosture,
};
pub(crate) use overlay_binding_export::{
    UiPortalOverlayBindingDenial, UiPortalOverlayBindingOwner, UiPortalOverlayBindingOwnerExport,
    UiPortalOverlayBindingRow,
};
pub(crate) use placement::{
    UiCommittedPortalPlacement, UiPortalLayerIdentity, UiPortalPlacementDenial,
    UiPortalPlacementSide, UiPreparedPortalPlacement, UiPresentedPortalBounds,
};
pub(crate) use proposal::UiStagedPortalServiceProposal;
pub(crate) use rebind::UiPreparedPortalRebindRemoval;
pub(crate) use receipt::{
    UiPortalExitRetentionReceipt, UiPortalServiceDisposition, UiPortalServiceReceipt,
};
pub(crate) use request::UiPortalServiceRequest;
#[cfg(feature = "certification-support")]
pub(crate) use scale_certification::portal_scale_evidence;
pub(crate) use stack_ordinal::UiPortalStackOrdinal;
pub(crate) use stack_ordinal::UiPortalStackOrdinalIssuer;
#[allow(
    unused_imports,
    reason = "the sealed Portal stack contract is consumed by the later overlay lane"
)]
pub(crate) use stack_snapshot::UiPortalStackSnapshot;
pub(crate) use state::{UiPortalRuntimeState, UiPortalShutdownReport};
pub(crate) use transition::{
    UiPortalExitTerminalDenial, UiPortalServiceTransitionDenial, UiPreparedPortalServiceTransition,
};
