mod command_routing;
mod focus;
mod motion;
mod normalized_plan;
mod policy_defaults;
mod policy_normalization_denial;
mod portal;
mod scroll;
mod scroll_wheel_behavior;
mod selection;

pub use command_routing::UiCommandRoutingPolicy;
pub(super) use command_routing::UiDeclaredCommandRoutingContract;
pub(super) use focus::UiDeclaredFocusOwnershipContract;
pub use focus::{UiFocusPolicy, UiFocusScopePolicy};
pub(super) use motion::UiDeclaredMotionPolicyContract;
pub use motion::{UiMotionPolicy, UiReducedMotionBehavior};
pub use normalized_plan::UiNormalizedServicePolicyPlan;
pub(crate) use policy_defaults::UiServicePolicyDefaults;
pub use policy_normalization_denial::UiServicePolicyNormalizationDenial;
pub(crate) use portal::UiDeclaredPortalPlacementGeometry;
pub(super) use portal::UiDeclaredPortalSurfaceContract;
pub use portal::{UiPortalPolicy, UiPortalPolicyKind};
pub(super) use scroll::UiDeclaredScrollOwnershipContract;
pub use scroll::{UiScrollAnchorBehavior, UiScrollPolicy, UiScrollRevealAlignment};
pub use scroll_wheel_behavior::{
    UiScrollWheelBehavior, UiScrollWheelBehaviorDenial, UI_SCROLL_WHEEL_SETTLE_TICK_CEILING,
};
pub(super) use selection::UiDeclaredSelectionIdentityContract;
pub use selection::{UiSelectionMode, UiSelectionPolicy};

#[cfg(test)]
mod tests;
