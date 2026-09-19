use super::{WorthUiAuthoredServiceDeclaration, WorthUiSemanticHandoffEvidence};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorthUiAuthoredServicePolicy {
    Focus(crate::declaration::UiFocusPolicy),
    Motion(crate::declaration::UiMotionPolicy),
    Scroll(crate::declaration::UiScrollPolicy),
    Selection(crate::declaration::UiSelectionPolicy),
}

impl WorthUiSemanticHandoffEvidence {
    pub(crate) fn authored_service_policy_defaults(
        &self,
    ) -> crate::declaration::UiServicePolicyDefaults {
        self.service_declarations()
            .iter()
            .filter_map(WorthUiAuthoredServiceDeclaration::authored_policy)
            .fold(
                crate::declaration::UiServicePolicyDefaults::default(),
                |defaults, policy| policy.apply(defaults),
            )
    }
}

impl WorthUiAuthoredServiceDeclaration {
    pub(super) fn policy_digest(&self) -> Option<u64> {
        self.authored_policy()
            .map(WorthUiAuthoredServicePolicy::digest_basis)
    }

    fn authored_policy(&self) -> Option<WorthUiAuthoredServicePolicy> {
        Some(match self.meaning() {
            // Portal declarations retain exact policy at their live Portal row.
            // They cannot collapse into one application-wide family default.
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Portal(_) => return None,
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Focus(focus) => {
                let policy = match focus.scope() {
                    worth_ui_dsl::WorthUiFocusScope::Workbench => {
                        crate::declaration::UiFocusPolicy::workbench()
                    }
                    worth_ui_dsl::WorthUiFocusScope::Portal => {
                        crate::declaration::UiFocusPolicy::portal_scope()
                    }
                    worth_ui_dsl::WorthUiFocusScope::Composite => {
                        crate::declaration::UiFocusPolicy::composite_scope()
                    }
                }
                .with_scope_restoration(focus.restores())
                .with_focus_reveal(focus.reveals());
                WorthUiAuthoredServicePolicy::Focus(policy)
            }
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Motion(_) => {
                WorthUiAuthoredServicePolicy::Motion(
                    crate::declaration::UiMotionPolicy::system_respecting(),
                )
            }
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Command(_) => return None,
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Scroll(scroll) => {
                let anchor = match scroll.anchor() {
                    worth_ui_dsl::WorthUiScrollAnchorPolicy::StableKey => {
                        crate::declaration::UiScrollAnchorBehavior::RebaseStableAnchor
                    }
                    worth_ui_dsl::WorthUiScrollAnchorPolicy::Clamp => {
                        crate::declaration::UiScrollAnchorBehavior::ClampOffset
                    }
                };
                WorthUiAuthoredServicePolicy::Scroll(
                    crate::declaration::UiScrollPolicy::nested_region()
                        .with_remainder_bubbling(scroll.nested())
                        .with_anchor_behavior(anchor),
                )
            }
            worth_ui_dsl::WorthUiServiceDeclarationMeaning::Selection(selection) => {
                let policy = match selection.mode() {
                    worth_ui_dsl::WorthUiSelectionMode::Single => {
                        crate::declaration::UiSelectionPolicy::single()
                    }
                    worth_ui_dsl::WorthUiSelectionMode::Multiple => {
                        crate::declaration::UiSelectionPolicy::multiple()
                    }
                    worth_ui_dsl::WorthUiSelectionMode::Range => {
                        crate::declaration::UiSelectionPolicy::range()
                    }
                }
                .with_stable_key_preservation(selection.preserves_stable_key());
                WorthUiAuthoredServicePolicy::Selection(policy)
            }
        })
    }
}

pub(super) fn portal_policy(
    portal: &worth_ui_dsl::WorthUiPortalDeclaration,
) -> crate::declaration::UiPortalPolicy {
    match portal.layer() {
        worth_ui_dsl::WorthUiPortalLayer::Transient => {
            crate::declaration::UiPortalPolicy::dropdown()
        }
        worth_ui_dsl::WorthUiPortalLayer::Modal => {
            crate::declaration::UiPortalPolicy::modal_dialog()
        }
    }
    .with_focus_restoration(portal.restores_focus())
    .with_escape_dismissal(portal.dismissal().escape())
    .with_outside_press_dismissal(portal.dismissal().outside_press())
    .with_accepted_selection_dismissal(portal.dismissal().accepted_selection())
    .with_anchor_loss_dismissal(portal.dismissal().anchor_gone())
}

impl WorthUiAuthoredServicePolicy {
    fn apply(
        self,
        defaults: crate::declaration::UiServicePolicyDefaults,
    ) -> crate::declaration::UiServicePolicyDefaults {
        match self {
            Self::Focus(policy) => defaults.with_focus(policy),
            Self::Motion(policy) => defaults.with_motion(policy),
            Self::Scroll(policy) => defaults.with_scroll(policy),
            Self::Selection(policy) => defaults.with_selection(policy),
        }
    }

    fn digest_basis(self) -> u64 {
        match self {
            Self::Focus(policy) => policy.digest_basis(),
            Self::Motion(policy) => policy.digest_basis(),
            Self::Scroll(policy) => policy.digest_basis(),
            Self::Selection(policy) => policy.digest_basis(),
        }
    }
}
