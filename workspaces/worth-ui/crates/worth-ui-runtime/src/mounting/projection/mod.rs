mod appearance;
#[cfg(test)]
pub(crate) use appearance::derive_unbound_ancestry;
mod appearance_selection;
mod cost_accounting;
mod denial;
mod focus_scope;
mod frame_storage;
mod geometry;
mod hit_test;
mod intent_posture;
mod lowering;
mod mechanical_role;
mod node_receipt;
mod participation;
mod pointer_affordance;
mod pointer_affordance_work;
mod prepared_projection;
/// Scroll chrome lowers beside node lowering rather than through it: chrome has
/// no graph node and no mounted instance of its own.
#[path = "lowering/scroll_chrome_lowering.rs"]
mod scroll_chrome_lowering;
#[cfg(test)]
#[path = "lowering/scroll_chrome_lowering_tests.rs"]
mod scroll_chrome_lowering_tests;
mod semantic_text;

pub(crate) use appearance::UiMountedAppearanceClip;
#[cfg(test)]
pub(crate) use appearance::UiMountedAppearanceClipDenial;
pub(crate) use appearance::{
    UiMountedAppearanceDerivedInput, UiMountedAppearanceGeometryInput,
    UiMountedAppearanceLoweringDenial, UiMountedAppearanceLoweringInput,
    UiMountedAppearanceNodeInput, UiMountedAppearanceScrollChromeInput,
    UiMountedAppearanceSurfaceOverlayInput, UiMountedAppearanceTextSpanInput,
    UiResolvedAppearanceNodeSource,
};
pub(crate) use appearance_selection::UiMountedAppearanceProjectionSelection;
pub use appearance_selection::UiMountedAppearanceSelectionCostReport;
pub use denial::UiMountedProjectionDenial;
pub(crate) use focus_scope::UiMountedFocusScope;
pub(in crate::mounting) use frame_storage::diagnostic_source::UiMountedDiagnosticSource;
pub(crate) use frame_storage::presentation_sources::compile as compile_presentation_sources;
pub(crate) use frame_storage::UiMountedAppearanceNodeInputContext;
#[cfg(test)]
pub(crate) use frame_storage::UiMountedAppearanceOrderDenial;
pub(crate) use frame_storage::UiMountedAppearanceOutputDenial;
pub(crate) use frame_storage::UiMountedAppearanceSurfaceSampleGeometry;
pub(in crate::mounting) use frame_storage::UiMountedHitMechanicSource;
pub use frame_storage::UiMountedProjectionFrame;
pub(in crate::mounting) use frame_storage::UiMountedSemanticMechanicSource;
pub(in crate::mounting) use frame_storage::UiMountedSemanticProjection;
pub(crate) use frame_storage::{
    UiAppearanceStateCapacityExceeded, UiMountedAppearanceFrameState,
    UiMountedAppearanceStateMutationDenial, UiMountedProjectionFrameOwner,
};
pub(in crate::mounting) use hit_test::{reattribute_hit_test, reattribute_hit_test_with_probes};
pub use node_receipt::UiMountedNodeReceipt;
pub(crate) use pointer_affordance::UiMountedPointerAffordanceState;
pub use pointer_affordance::UiMountedPointerAffordanceWork;
pub use prepared_projection::UiProjectedMountedFrameCandidate;

pub(crate) use intent_posture::{
    UiIntentPostureCommit, UiIntentPostureObservation, UiIntentPostureTable,
};
pub(crate) use lowering::{
    prepare_projection, UiMountedPreviewProjectionInput, UiMountedProjectionInput,
};
pub(crate) use prepared_projection::{
    UiMountedPresentationDeltaSource, UiPreparedMountedProjection,
};
pub(crate) use scroll_chrome_lowering::{
    lower_scroll_chrome, UiMountedScrollChromeNode, UiScrollChromeLoweringDenial,
    UiScrollChromeLoweringInput,
};

#[cfg(test)]
pub(in crate::mounting) fn prove_paint_only_mechanic_locality() {
    frame_storage::mechanic_source_tests::mechanic_source_routes_paint_only_work_through_current_mounted_authority();
}
