pub(crate) mod anchored_allocation;
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
#[allow(
    dead_code,
    reason = "milestone 3.16 Gate 0 seals Portal total-order snapshots before appearance consumes them"
)]
mod stack_snapshot;
mod state;
mod transition;

#[cfg(test)]
mod state_dismissal_tests;
#[cfg(test)]
mod state_retention_tests;
#[cfg(test)]
mod state_tests;

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
pub(crate) use receipt::{
    UiPortalExitRetentionReceipt, UiPortalServiceDisposition, UiPortalServiceReceipt,
};
pub(crate) use request::UiPortalServiceRequest;
#[cfg(feature = "certification-support")]
pub(crate) use scale_certification::portal_scale_evidence;
#[allow(
    unused_imports,
    reason = "milestone 3.16 Gate 0 exposes the sealed Portal stack contract internally"
)]
pub(crate) use stack_snapshot::{UiPortalStackOrdinal, UiPortalStackSnapshot};
pub(crate) use state::{UiPortalRuntimeState, UiPortalShutdownReport};
pub(crate) use transition::{
    UiPortalExitTerminalDenial, UiPortalServiceTransitionDenial, UiPreparedPortalServiceTransition,
};
