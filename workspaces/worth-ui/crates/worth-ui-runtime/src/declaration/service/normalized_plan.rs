#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiNormalizedServicePolicyPlan {
    portal: Option<super::UiPortalPolicy>,
    focus: Option<super::UiFocusPolicy>,
    motion: Option<super::UiMotionPolicy>,
    command_routing: Option<super::UiCommandRoutingPolicy>,
    scroll: Option<super::UiScrollPolicy>,
    selection: Option<super::UiSelectionPolicy>,
}

impl UiNormalizedServicePolicyPlan {
    pub(crate) fn normalize(
        defaults: super::UiServicePolicyDefaults,
        authored: super::UiServicePolicyDefaults,
        support: crate::capability::UiRuntimeServiceSupport,
    ) -> Result<Self, super::UiServicePolicyNormalizationDenial> {
        use crate::capability::{UiRuntimeServiceFamily as Family, UiRuntimeServiceSupportPosture};
        let installed =
            |family| support.posture(family) == UiRuntimeServiceSupportPosture::Installed;
        let portal = installed(Family::Portal).then(|| {
            authored
                .portal()
                .or(defaults.portal())
                .unwrap_or(super::UiPortalPolicy::dropdown())
        });
        let focus = installed(Family::Focus).then(|| {
            authored
                .focus()
                .or(defaults.focus())
                .unwrap_or(super::UiFocusPolicy::workbench())
        });
        let motion = installed(Family::Motion).then(|| {
            authored
                .motion()
                .or(defaults.motion())
                .unwrap_or(super::UiMotionPolicy::system_respecting())
        });
        let command_routing = installed(Family::CommandRouting).then(|| {
            authored
                .command_routing()
                .or(defaults.command_routing())
                .unwrap_or(super::UiCommandRoutingPolicy::desktop())
        });
        let scroll = installed(Family::Scroll).then(|| {
            authored
                .scroll()
                .or(defaults.scroll())
                .unwrap_or(super::UiScrollPolicy::nested_region())
        });
        let selection = installed(Family::Selection).then(|| {
            authored
                .selection()
                .or(defaults.selection())
                .unwrap_or(super::UiSelectionPolicy::single())
        });
        if let Some(settle_ticks) = scroll
            .map(super::UiScrollPolicy::wheel_behavior)
            .and_then(super::UiScrollWheelBehavior::settle_ticks)
        {
            if motion.is_none() {
                return Err(
                    super::UiServicePolicyNormalizationDenial::SmoothWheelWithoutMotionOwner {
                        settle_ticks,
                    },
                );
            }
        }
        Ok(Self {
            portal,
            focus,
            motion,
            command_routing,
            scroll,
            selection,
        })
    }

    pub const fn portal(self) -> Option<super::UiPortalPolicy> {
        self.portal
    }

    pub const fn focus(self) -> Option<super::UiFocusPolicy> {
        self.focus
    }

    pub const fn motion(self) -> Option<super::UiMotionPolicy> {
        self.motion
    }

    pub const fn command_routing(self) -> Option<super::UiCommandRoutingPolicy> {
        self.command_routing
    }

    pub const fn scroll(self) -> Option<super::UiScrollPolicy> {
        self.scroll
    }

    pub const fn selection(self) -> Option<super::UiSelectionPolicy> {
        self.selection
    }

    pub const fn installed_family_count(self) -> usize {
        self.portal.is_some() as usize
            + self.focus.is_some() as usize
            + self.motion.is_some() as usize
            + self.command_routing.is_some() as usize
            + self.scroll.is_some() as usize
            + self.selection.is_some() as usize
    }

    pub(crate) const fn runtime_service_support(
        self,
    ) -> crate::capability::UiRuntimeServiceSupport {
        use crate::capability::UiRuntimeServiceFamily as Family;
        let mut support = crate::capability::UiRuntimeServiceSupport::none_installed();
        if self.portal.is_some() {
            support = support.with_installed(Family::Portal);
        }
        if self.focus.is_some() {
            support = support.with_installed(Family::Focus);
        }
        if self.motion.is_some() {
            support = support.with_installed(Family::Motion);
        }
        if self.command_routing.is_some() {
            support = support.with_installed(Family::CommandRouting);
        }
        if self.scroll.is_some() {
            support = support.with_installed(Family::Scroll);
        }
        if self.selection.is_some() {
            support = support.with_installed(Family::Selection);
        }
        support
    }

    pub(crate) fn digest_basis(self) -> u64 {
        optional_digest(self.portal.map(super::UiPortalPolicy::digest_basis), 3)
            ^ optional_digest(self.focus.map(super::UiFocusPolicy::digest_basis), 13)
            ^ optional_digest(self.motion.map(super::UiMotionPolicy::digest_basis), 23)
            ^ optional_digest(
                self.command_routing
                    .map(super::UiCommandRoutingPolicy::digest_basis),
                31,
            )
            ^ optional_digest(self.scroll.map(super::UiScrollPolicy::digest_basis), 41)
            ^ optional_digest(
                self.selection.map(super::UiSelectionPolicy::digest_basis),
                53,
            )
    }
}

fn optional_digest(value: Option<u64>, rotation: u32) -> u64 {
    match value {
        Some(value) => value
            .wrapping_add(0x9e37_79b9_7f4a_7c15)
            .rotate_left(rotation),
        None => 0,
    }
}
