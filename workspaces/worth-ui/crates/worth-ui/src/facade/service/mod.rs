//! Canonical public runtime-service declarations.

mod command_routing;
mod focus;
mod motion;
mod portal;
mod scroll;
mod selection;

pub use command_routing::{
    UiCommandContextConsumption, UiCommandKeyCode, UiCommandLogicalKey, UiCommandModifierSet,
    UiCommandPhysicalKey, UiCommandRegistrationGeneration, UiCommandRegistrationOwner,
    UiCommandRegistrationOwnerIdentity, UiCommandRepeatPolicy, UiCommandRouteDeclaration,
    UiCommandRouteDestination, UiCommandRoutePriority, UiCommandRouteScope, UiCommandRoutingPolicy,
    UiCommandShortcutKey, UiCommandShortcutPlatform, UiCommandShortcutSequence,
    UiCommandShortcutStroke, UiCommandTextInputPolicy,
};
pub use focus::{UiFocusPolicy, UiFocusScopePolicy};
pub use motion::{UiMotionPolicy, UiReducedMotionBehavior};
pub use portal::{UiPortalPolicy, UiPortalPolicyKind};
pub use scroll::{
    UiScrollAnchorBehavior, UiScrollPolicy, UiScrollRevealAlignment, UiScrollWheelBehavior,
    UiScrollWheelBehaviorDenial, UI_SCROLL_WHEEL_SETTLE_TICK_CEILING,
};
pub use selection::{UiSelectionMode, UiSelectionPolicy};

pub use worth_ui_runtime::facade::service::{
    UiNormalizedServicePolicyPlan, UiServicePolicyNormalizationDenial,
};
