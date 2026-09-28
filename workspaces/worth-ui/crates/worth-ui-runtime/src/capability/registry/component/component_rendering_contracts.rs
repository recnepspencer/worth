/// Accessibility support declared by a renderable component capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentAccessibilitySupport {
    Semantic,
    DecorativeOnly,
}

impl ComponentAccessibilitySupport {
    pub fn semantic() -> Self {
        Self::Semantic
    }

    pub fn decorative_only() -> Self {
        Self::DecorativeOnly
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Semantic => "semantic",
            Self::DecorativeOnly => "decorative_only",
        }
    }
}

/// Focus behavior declared by a renderable component capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentFocusSupport {
    NotFocusable,
    Focusable,
    FocusContainer(ComponentFocusContainerPolicy),
}

/// Keyboard movement admitted by one declared focus container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentFocusContainerPolicy {
    Roving {
        axis: ComponentFocusNavigationAxis,
        wrap: bool,
    },
    ActiveDescendant {
        axis: ComponentFocusNavigationAxis,
        wrap: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentFocusNavigationAxis {
    Horizontal,
    Vertical,
    Both,
}

impl ComponentFocusSupport {
    pub fn not_focusable() -> Self {
        Self::NotFocusable
    }

    pub fn focusable() -> Self {
        Self::Focusable
    }

    pub fn roving_focus_container(axis: ComponentFocusNavigationAxis, wrap: bool) -> Self {
        Self::FocusContainer(ComponentFocusContainerPolicy::Roving { axis, wrap })
    }

    pub fn active_descendant_focus_container(
        axis: ComponentFocusNavigationAxis,
        wrap: bool,
    ) -> Self {
        Self::FocusContainer(ComponentFocusContainerPolicy::ActiveDescendant { axis, wrap })
    }

    pub(crate) const fn container_policy(self) -> Option<ComponentFocusContainerPolicy> {
        match self {
            Self::FocusContainer(policy) => Some(policy),
            Self::NotFocusable | Self::Focusable => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::NotFocusable => "not_focusable",
            Self::Focusable => "focusable",
            Self::FocusContainer(ComponentFocusContainerPolicy::Roving {
                axis: ComponentFocusNavigationAxis::Horizontal,
                wrap: false,
            }) => "roving_horizontal_bounded",
            Self::FocusContainer(ComponentFocusContainerPolicy::Roving {
                axis: ComponentFocusNavigationAxis::Horizontal,
                wrap: true,
            }) => "roving_horizontal_wrapped",
            Self::FocusContainer(ComponentFocusContainerPolicy::Roving {
                axis: ComponentFocusNavigationAxis::Vertical,
                wrap: false,
            }) => "roving_vertical_bounded",
            Self::FocusContainer(ComponentFocusContainerPolicy::Roving {
                axis: ComponentFocusNavigationAxis::Vertical,
                wrap: true,
            }) => "roving_vertical_wrapped",
            Self::FocusContainer(ComponentFocusContainerPolicy::Roving {
                axis: ComponentFocusNavigationAxis::Both,
                wrap: false,
            }) => "roving_both_bounded",
            Self::FocusContainer(ComponentFocusContainerPolicy::Roving {
                axis: ComponentFocusNavigationAxis::Both,
                wrap: true,
            }) => "roving_both_wrapped",
            Self::FocusContainer(ComponentFocusContainerPolicy::ActiveDescendant {
                axis: ComponentFocusNavigationAxis::Horizontal,
                wrap: false,
            }) => "active_descendant_horizontal_bounded",
            Self::FocusContainer(ComponentFocusContainerPolicy::ActiveDescendant {
                axis: ComponentFocusNavigationAxis::Horizontal,
                wrap: true,
            }) => "active_descendant_horizontal_wrapped",
            Self::FocusContainer(ComponentFocusContainerPolicy::ActiveDescendant {
                axis: ComponentFocusNavigationAxis::Vertical,
                wrap: false,
            }) => "active_descendant_vertical_bounded",
            Self::FocusContainer(ComponentFocusContainerPolicy::ActiveDescendant {
                axis: ComponentFocusNavigationAxis::Vertical,
                wrap: true,
            }) => "active_descendant_vertical_wrapped",
            Self::FocusContainer(ComponentFocusContainerPolicy::ActiveDescendant {
                axis: ComponentFocusNavigationAxis::Both,
                wrap: false,
            }) => "active_descendant_both_bounded",
            Self::FocusContainer(ComponentFocusContainerPolicy::ActiveDescendant {
                axis: ComponentFocusNavigationAxis::Both,
                wrap: true,
            }) => "active_descendant_both_wrapped",
        }
    }
}

impl ComponentFocusContainerPolicy {
    pub const fn axis(self) -> ComponentFocusNavigationAxis {
        match self {
            Self::Roving { axis, .. } | Self::ActiveDescendant { axis, .. } => axis,
        }
    }

    pub const fn wraps(self) -> bool {
        match self {
            Self::Roving { wrap, .. } | Self::ActiveDescendant { wrap, .. } => wrap,
        }
    }
}

/// Execution lane hint consumed by later runtime lowering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentExecutionLane {
    Passive,
    Interactive,
    Virtualized,
    CanvasSpatial,
    RealtimeOverlay,
}

impl ComponentExecutionLane {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passive => "passive",
            Self::Interactive => "interactive",
            Self::Virtualized => "virtualized",
            Self::CanvasSpatial => "canvas_spatial",
            Self::RealtimeOverlay => "realtime_overlay",
        }
    }
}

/// Bounded source-independent scale policy for one canvas component.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentCanvasSpatialContract {
    visible_primitive_limit: u32,
    overlay_row_limit: u16,
    tool_state_row_limit: u16,
}

impl ComponentCanvasSpatialContract {
    pub fn new(
        visible_primitive_limit: u32,
        overlay_row_limit: u16,
        tool_state_row_limit: u16,
    ) -> Option<Self> {
        (visible_primitive_limit > 0).then_some(Self {
            visible_primitive_limit,
            overlay_row_limit,
            tool_state_row_limit,
        })
    }

    pub fn visible_primitive_limit(self) -> u32 {
        self.visible_primitive_limit
    }

    pub fn overlay_row_limit(self) -> u16 {
        self.overlay_row_limit
    }

    pub fn tool_state_row_limit(self) -> u16 {
        self.tool_state_row_limit
    }

    pub(crate) fn digest_basis(self) -> u64 {
        u64::from(self.visible_primitive_limit)
            ^ u64::from(self.overlay_row_limit).rotate_left(21)
            ^ u64::from(self.tool_state_row_limit).rotate_left(43)
    }
}

/// Immutable priority carried by one realtime overlay declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentRealtimeOverlayPriority {
    HudOverlay,
    CriticalOverlay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentRealtimeOverlayContractDenialReason {
    ZeroOverlayRowLimit,
    ZeroDeclaredFrameCost,
    ZeroFrameBudget,
    FrameBudgetOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentRealtimeOverlayContractDenial {
    reason: ComponentRealtimeOverlayContractDenialReason,
}

/// Bounded, source-independent execution policy for one realtime overlay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentRealtimeOverlayContract {
    overlay_row_limit: u16,
    declared_frame_cost_millis: u16,
    frame_budget_millis: u16,
    priority: ComponentRealtimeOverlayPriority,
}

impl ComponentRealtimeOverlayContract {
    pub fn new(
        overlay_row_limit: u16,
        declared_frame_cost_millis: u16,
        frame_budget_millis: u32,
        priority: ComponentRealtimeOverlayPriority,
    ) -> Result<Self, ComponentRealtimeOverlayContractDenial> {
        let reason = if overlay_row_limit == 0 {
            Some(ComponentRealtimeOverlayContractDenialReason::ZeroOverlayRowLimit)
        } else if declared_frame_cost_millis == 0 {
            Some(ComponentRealtimeOverlayContractDenialReason::ZeroDeclaredFrameCost)
        } else if frame_budget_millis == 0 {
            Some(ComponentRealtimeOverlayContractDenialReason::ZeroFrameBudget)
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(ComponentRealtimeOverlayContractDenial { reason });
        }
        let frame_budget_millis = u16::try_from(frame_budget_millis).map_err(|_| {
            ComponentRealtimeOverlayContractDenial {
                reason: ComponentRealtimeOverlayContractDenialReason::FrameBudgetOverflow,
            }
        })?;
        Ok(Self {
            overlay_row_limit,
            declared_frame_cost_millis,
            frame_budget_millis,
            priority,
        })
    }

    pub fn overlay_row_limit(self) -> u16 {
        self.overlay_row_limit
    }
    pub fn declared_frame_cost_millis(self) -> u16 {
        self.declared_frame_cost_millis
    }
    pub fn frame_budget_millis(self) -> u16 {
        self.frame_budget_millis
    }
    pub fn priority(self) -> ComponentRealtimeOverlayPriority {
        self.priority
    }

    pub(crate) fn digest_basis(self) -> u64 {
        u64::from(self.overlay_row_limit)
            ^ u64::from(self.declared_frame_cost_millis).rotate_left(17)
            ^ u64::from(self.frame_budget_millis).rotate_left(37)
            ^ self.priority.digest_tag().rotate_left(53)
    }
}

impl ComponentRealtimeOverlayContractDenial {
    pub fn reason(self) -> ComponentRealtimeOverlayContractDenialReason {
        self.reason
    }
}

impl ComponentRealtimeOverlayPriority {
    pub(crate) fn digest_tag(self) -> u64 {
        match self {
            Self::HudOverlay => 1,
            Self::CriticalOverlay => 2,
        }
    }
}
