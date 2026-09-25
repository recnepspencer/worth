mod authority;
mod consumption_view;
pub(crate) mod coordinator;
mod effect_requirements;
mod focus_placement;
pub(crate) mod motion_sampling;
mod opacity_composition;
mod outcome;
mod preflight;
mod presented_surface;
mod reconciliation;
mod shutdown;
mod state;
mod terminal;
mod truth_geometry;
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
    UiFocusHostPlacementReconciliationReceipt, UiFocusHostPlacementSettlementDenial,
    UiFocusHostPlacementShutdownReport,
};
pub(crate) use focus_placement::{
    UiMountedFocusPlacementDenial, UiMountedFocusPlacementRequestBasis,
};
pub(in crate::mounting) use opacity_composition::compose_opacity;
pub use outcome::{
    UiMountedIndeterminateFrame, UiMountedPresentationOutcome, UiMountedPresentationReceipt,
    UiMountedPresentationWitness, UiMountedPresentedFrame, UiMountedRejectedFrame,
    UiMountedSupersededFrame, UiMountedSurfacePresentationReceipt,
    UiMountedSurfacePresentationRejection, UiPresentationIndeterminateReport,
};
#[cfg(any(test, feature = "certification-support"))]
pub(crate) use presented_surface::presented_surface_witness_for_certification;
pub(crate) use presented_surface::{UiDisplayedSurfaceBasis, UiPresentedSurfaceWitness};
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
#[cfg(test)]
pub(crate) use truth_geometry::{displayed_rect_for_test, displayed_scroll_offset_for_test};
pub(crate) use truth_geometry::{
    UiAcceptedRect, UiDisplayedRect, UiDisplayedScrollOffset, UiPublishedMap, UiPublishedRect,
    UiPublishedToAcceptedMap, UiRebaseDenial, UiScrollPoseShift, UiScrollStandingDenial,
    UiTruthGeometryDenial,
};
