#[cfg(test)]
mod accessibility_focus_hook;
mod active_descendant;
#[allow(
    dead_code,
    reason = "milestone 3.16 Gate 0 seals the focus appearance export before it is consumed"
)]
mod appearance_export;
mod container_navigation;
mod identity;
mod inspection;
mod modality;
mod participant;
mod portal_transition;
mod proposal;
mod rebind;
mod receipt;
mod request;
mod restoration;
mod routing;
#[cfg(feature = "certification-support")]
mod scale_certification;
mod semantic_focus;
mod state;
#[cfg(test)]
mod state_tests;

#[cfg(test)]
pub(in crate::runtime) use accessibility_focus_hook::{
    UiAccessibilityFocusHook, UiAccessibilityFocusHookSupport,
};
pub(in crate::runtime) use active_descendant::UiActiveDescendant;
#[allow(
    unused_imports,
    reason = "milestone 3.16 Gate 0 exposes the sealed focus appearance contract internally"
)]
pub(crate) use appearance_export::{
    UiFocusAppearanceClass, UiFocusAppearancePosture, UiFocusAppearanceTarget,
};
pub(crate) use container_navigation::{
    UiFocusContainerNavigationKey, UiFocusContainerNavigationReceipt,
};
pub(in crate::runtime) use identity::UiFocusParticipantIdentity;
pub(crate) use identity::UiFocusScopeIdentity;
pub(in crate::runtime) use inspection::UiFocusInspectionSnapshot;
pub(in crate::runtime) use modality::{UiFocusVisibleModality, UiWindowFocus};
pub(in crate::runtime) use participant::UiFocusParticipant;
pub(crate) use portal_transition::UiPortalFocusTransitionDenial;
pub(in crate::runtime) use proposal::{
    UiPortalFocusBoundaryIdentity, UiPortalFocusRequirement, UiStagedFocusServiceProposal,
};
pub(crate) use rebind::UiPreparedFocusMountedReconciliation;
pub(crate) use receipt::UiFocusOutcome;
pub(crate) use receipt::UiFocusReconciliationReceipt;
pub(crate) use receipt::UiFocusTransitionReceipt;
pub(crate) use request::UiFocusCause;
#[cfg(any(test, feature = "certification-support"))]
pub(in crate::runtime) use request::UiFocusRequest;
pub(in crate::runtime) use request::UiFocusTraversalDirection;
pub(in crate::runtime) use restoration::UiFocusRestorationToken;
pub(in crate::runtime) use routing::UiFocusPlan;
pub(crate) use routing::UiFocusRoutingDenial;
pub(crate) use routing::UiHostFocusTraversalDirection;
#[cfg(feature = "certification-support")]
pub(crate) use scale_certification::focus_scale_evidence;
pub(crate) use semantic_focus::UiSemanticKeyboardFocus;
pub(crate) use state::UiFocusRuntimeState;
