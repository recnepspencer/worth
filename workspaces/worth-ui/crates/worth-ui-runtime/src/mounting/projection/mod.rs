mod appearance;
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
mod prepared_projection;
mod semantic_text;
mod static_paint;

pub(crate) use appearance::{
    UiMountedAppearanceLoweringDenial, UiMountedAppearanceLoweringInput,
    UiMountedAppearanceNodeInput,
};
pub(crate) use appearance_selection::{
    UiMountedAppearanceProjectionSelection, UiMountedAppearanceSelectionCostReport,
};
pub use denial::UiMountedProjectionDenial;
pub(crate) use focus_scope::UiMountedFocusScope;
pub(in crate::mounting) use frame_storage::diagnostic_source::UiMountedDiagnosticSource;
pub(crate) use frame_storage::presentation_sources::compile as compile_presentation_sources;
pub(crate) use frame_storage::UiMountedAppearanceNodeInputContext;
pub use frame_storage::UiMountedProjectionFrame;
pub(in crate::mounting) use frame_storage::UiMountedSemanticMechanicSource;
pub(in crate::mounting) use frame_storage::UiMountedSemanticProjection;
pub(crate) use frame_storage::{
    UiAppearanceStateCapacityExceeded, UiMountedAppearanceFrameState,
    UiMountedAppearanceStateMutationDenial, UiMountedProjectionFrameOwner,
};
pub(in crate::mounting) use hit_test::reattribute_hit_test;
pub use node_receipt::UiMountedNodeReceipt;
pub use prepared_projection::UiProjectedMountedFrameCandidate;
pub(in crate::mounting) use static_paint::reattribute_filled_rect;

pub(crate) use intent_posture::{
    UiIntentPostureCommit, UiIntentPostureObservation, UiIntentPostureTable,
};
pub(crate) use lowering::{
    prepare_projection, UiMountedPreviewProjectionInput, UiMountedProjectionInput,
};
pub(crate) use prepared_projection::{
    UiMountedPresentationDeltaSource, UiPreparedMountedProjection,
};

#[cfg(test)]
pub(in crate::mounting) fn prove_paint_only_mechanic_locality() {
    frame_storage::mechanic_source_tests::mechanic_source_routes_paint_only_work_through_current_mounted_authority();
}
