//! Public runtime-service declaration policies and normalized prepared plans.

pub use crate::declaration::{
    UiCommandRoutingPolicy, UiFocusPolicy, UiFocusScopePolicy, UiMotionPolicy,
    UiNormalizedServicePolicyPlan, UiPortalPolicy, UiPortalPolicyKind, UiReducedMotionBehavior,
    UiScrollAnchorBehavior, UiScrollPolicy, UiScrollRevealAlignment, UiScrollWheelBehavior,
    UiScrollWheelBehaviorDenial, UiSelectionMode, UiSelectionPolicy,
    UiServicePolicyNormalizationDenial, UI_SCROLL_WHEEL_SETTLE_TICK_CEILING,
};

pub use crate::facade::registry::descriptor::{
    UiCommandContextConsumption, UiCommandKeyCode, UiCommandLogicalKey, UiCommandModifierSet,
    UiCommandPhysicalKey, UiCommandRegistrationGeneration, UiCommandRegistrationOwner,
    UiCommandRegistrationOwnerIdentity, UiCommandRepeatPolicy, UiCommandRouteDeclaration,
    UiCommandRouteDestination, UiCommandRoutePriority, UiCommandRouteScope, UiCommandShortcutKey,
    UiCommandShortcutPlatform, UiCommandShortcutSequence, UiCommandShortcutStroke,
    UiCommandTextInputPolicy,
};
