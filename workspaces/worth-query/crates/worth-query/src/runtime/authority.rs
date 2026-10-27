pub use worth_query_execution::facade::evidence_identity::WorthQueryAuthorityLane;

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryEffectAction {
    Derive,
    Deliver,
    WriteIntent,
}

impl WorthQueryEffectAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Derive => "derive",
            Self::Deliver => "deliver",
            Self::WriteIntent => "write-intent",
        }
    }
}

impl std::fmt::Display for WorthQueryEffectAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryEffectPolicy {
    #[default]
    DeriveOnly,
    Muted,
    Redirected,
    SandboxedWriteIntent,
    AuthoritativeAllowed,
}

impl WorthQueryEffectPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeriveOnly => "derive-only",
            Self::Muted => "muted",
            Self::Redirected => "redirected",
            Self::SandboxedWriteIntent => "sandboxed-write-intent",
            Self::AuthoritativeAllowed => "authoritative-allowed",
        }
    }

    pub fn admit(
        self,
        action: WorthQueryEffectAction,
        target_lane: WorthQueryAuthorityLane,
    ) -> Result<WorthQueryEffectAdmission, WorthQueryEffectPolicyDenial> {
        let admitted = match self {
            Self::DeriveOnly => {
                action == WorthQueryEffectAction::Derive
                    && target_lane == WorthQueryAuthorityLane::DerivedRuntimeState
            }
            Self::Muted => false,
            Self::Redirected => match action {
                WorthQueryEffectAction::Derive => {
                    target_lane == WorthQueryAuthorityLane::DerivedRuntimeState
                }
                WorthQueryEffectAction::Deliver => matches!(
                    target_lane,
                    WorthQueryAuthorityLane::EffectDeliveryState
                        | WorthQueryAuthorityLane::PreviewTruth
                        | WorthQueryAuthorityLane::BranchLocalTruth
                ),
                WorthQueryEffectAction::WriteIntent => false,
            },
            Self::SandboxedWriteIntent => match action {
                WorthQueryEffectAction::Derive => {
                    target_lane == WorthQueryAuthorityLane::DerivedRuntimeState
                }
                WorthQueryEffectAction::WriteIntent => matches!(
                    target_lane,
                    WorthQueryAuthorityLane::PreviewTruth
                        | WorthQueryAuthorityLane::BranchLocalTruth
                        | WorthQueryAuthorityLane::PendingWriteIntent
                ),
                WorthQueryEffectAction::Deliver => false,
            },
            Self::AuthoritativeAllowed => true,
        };

        if admitted {
            Ok(WorthQueryEffectAdmission {
                policy: self,
                action,
                target_lane,
            })
        } else {
            Err(WorthQueryEffectPolicyDenial {
                policy: self,
                action,
                target_lane,
            })
        }
    }
}

impl std::fmt::Display for WorthQueryEffectPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryEffectAdmission {
    policy: WorthQueryEffectPolicy,
    action: WorthQueryEffectAction,
    target_lane: WorthQueryAuthorityLane,
}

impl WorthQueryEffectAdmission {
    pub fn policy(&self) -> WorthQueryEffectPolicy {
        self.policy
    }

    pub fn action(&self) -> WorthQueryEffectAction {
        self.action
    }

    pub fn target_lane(&self) -> WorthQueryAuthorityLane {
        self.target_lane
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryEffectPolicyDenial {
    policy: WorthQueryEffectPolicy,
    action: WorthQueryEffectAction,
    target_lane: WorthQueryAuthorityLane,
}

impl WorthQueryEffectPolicyDenial {
    pub fn policy(&self) -> WorthQueryEffectPolicy {
        self.policy
    }

    pub fn action(&self) -> WorthQueryEffectAction {
        self.action
    }

    pub fn target_lane(&self) -> WorthQueryAuthorityLane {
        self.target_lane
    }
}

impl std::fmt::Display for WorthQueryEffectPolicyDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "effect policy `{}` does not admit `{}` into `{}`",
            self.policy, self.action, self.target_lane
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryPreviewOptions {
    effect_policy: WorthQueryEffectPolicy,
}

impl WorthQueryPreviewOptions {
    pub fn derive_only() -> Self {
        Self::default()
    }

    pub fn muted() -> Self {
        Self {
            effect_policy: WorthQueryEffectPolicy::Muted,
        }
    }

    pub fn redirected_delivery() -> Self {
        Self {
            effect_policy: WorthQueryEffectPolicy::Redirected,
        }
    }

    pub fn sandboxed_write_intent() -> Self {
        Self {
            effect_policy: WorthQueryEffectPolicy::SandboxedWriteIntent,
        }
    }

    #[cfg(test)]
    pub(in crate::runtime) fn with_effect_policy(
        mut self,
        effect_policy: WorthQueryEffectPolicy,
    ) -> Self {
        self.effect_policy = effect_policy;
        self
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryBranchOptions {
    effect_policy: WorthQueryEffectPolicy,
}

impl WorthQueryBranchOptions {
    pub fn derive_only() -> Self {
        Self::default()
    }

    pub fn muted() -> Self {
        Self {
            effect_policy: WorthQueryEffectPolicy::Muted,
        }
    }

    pub fn redirected_delivery() -> Self {
        Self {
            effect_policy: WorthQueryEffectPolicy::Redirected,
        }
    }

    pub fn sandboxed_write_intent() -> Self {
        Self {
            effect_policy: WorthQueryEffectPolicy::SandboxedWriteIntent,
        }
    }

    pub fn effect_policy(&self) -> WorthQueryEffectPolicy {
        self.effect_policy
    }
}
