mod authority;
mod consumption_view;
pub(crate) mod coordinator;
mod effect_requirements;
mod focus_placement;
pub(crate) mod motion_sampling;
mod opacity_composition;
mod outcome;
mod preflight;
mod reconciliation;
mod shutdown;
mod state;
mod terminal;
pub(crate) mod work_producer;
#[cfg(test)]
mod work_producer_tests;

pub(crate) use authority::{
    UiMountedPresentationLease, UiMountedPresentationLeaseDenial, UiMountedPresentationLeaseGate,
    UiMountedPresentationWork,
};
pub(crate) use consumption_view::UiMountedHostPresentationAuthority;
pub(crate) use coordinator::{
    UiAcceptedAppearanceMotion, UiMotionSamplePresentationOutcome, UiMountedPresentationCoordinator,
};
pub use focus_placement::{
    UiFocusHostPlacementReconciliationDenial, UiFocusHostPlacementReconciliationOutcome,
    UiFocusHostPlacementReconciliationReceipt, UiFocusHostPlacementShutdownReport,
};
pub(crate) use focus_placement::{
    UiFocusHostPlacementSettlementDenial, UiMountedFocusPlacementDenial,
    UiMountedFocusPlacementRequestBasis,
};
pub(in crate::mounting) use opacity_composition::compose_opacity;
pub use outcome::{
    UiMountedIndeterminateFrame, UiMountedPresentationOutcome, UiMountedPresentationReceipt,
    UiMountedPresentationWitness, UiMountedPresentedFrame, UiMountedRejectedFrame,
    UiMountedSupersededFrame, UiMountedSurfacePresentationReceipt,
    UiMountedSurfacePresentationRejection, UiPresentationIndeterminateReport,
};
pub use reconciliation::{UiHostPresentationReconciliation, UiMountedSurfaceReconciliationBinding};
pub(super) use shutdown::{UiMountedPresentationQueryShutdown, UiMountedPresentationTextShutdown};
pub use shutdown::{
    UiMountedPresentationShutdownAttempt, UiMountedPresentationShutdownDisposition,
    UiMountedPresentationShutdownReport,
};
pub(crate) use state::UiMountedSupersedingPresentationBasis;
pub use state::{
    UiMountedPresentationAdmission, UiMountedPresentationAdmissionDenial,
    UiMountedPresentationAdmissionRejection, UiMountedPresentationAttempt,
    UiMountedPresentationCompletionDenial, UiMountedPresentationInFlight,
};
