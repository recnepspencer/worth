mod command_target;
mod continuity;
mod execution_affinity;
mod pointer_position;
mod pointer_target_refresh;
pub(crate) use pointer_target_refresh::refresh_pointer_target;
mod presented_frame;
mod presented_geometry;
mod presented_target;

pub(crate) use pointer_position::{current_pointer_surface, UiPresentedPointerPosition};

pub(crate) use command_target::resolve_presented_command_target;
pub use continuity::UiPointerGestureContinuityKind;
pub use presented_frame::UiInteractionTargetingDenial;
pub(crate) use presented_frame::{
    admit_current_target, admit_current_target_incarnation, map_current_affinity_denial,
    require_current_presentation, require_current_target, resolve_presented_focus_target,
    resolve_presented_surface_target,
};
#[cfg(test)]
pub(crate) use presented_target::interaction_target_view_for_test;
pub use presented_target::{
    UiPresentedInteractionTarget, UiPresentedInteractionTargetView, UiPresentedTargetFrameRelation,
};

pub(crate) use continuity::{issue_continuity, UiPointerGestureContinuityDenial};
pub(crate) use execution_affinity::{
    admit_continued_intent_execution_affinity, admit_presented_intent_execution_affinity,
    UiIntentExecutionTargetAffinity,
};
pub(crate) use presented_frame::resolve_presented_target;
pub(crate) use presented_geometry::{UiPresentedInteractionGeometry, UiPresentedViewportGeometry};

pub(crate) use presented_frame::map_hit_query_denial;
