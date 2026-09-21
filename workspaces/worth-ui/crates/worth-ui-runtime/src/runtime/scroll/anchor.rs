#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollAnchorIdentity {
    ApplicationItem(crate::runtime::UiApplicationItemKey),
    Mounted(worth_ui_host_contract::UiMountedInstanceIdentity),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollAnchor {
    identity: UiScrollAnchorIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    inline_subpixels: i64,
    block_subpixels: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollAnchorPolicy {
    Preserve,
    Rebase,
    Clamp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollAnchorReconciliationOutcome {
    Preserved,
    Rebased,
    Clamped,
    Replaced,
    Dropped,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollRebindRequest {
    registration: super::UiScrollOwnerRegistration,
    successor_anchor: Option<UiScrollAnchor>,
    policy: UiScrollAnchorPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollAnchorReconciliationReceipt {
    outcome: UiScrollAnchorReconciliationOutcome,
    offset: super::UiScrollOffset,
}

impl UiScrollAnchorIdentity {
    pub(crate) const fn application_item(value: crate::runtime::UiApplicationItemKey) -> Self {
        Self::ApplicationItem(value)
    }

    pub(crate) const fn mounted(
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Self {
        Self::Mounted(instance)
    }
}

impl UiScrollAnchor {
    pub(crate) const fn new(
        identity: UiScrollAnchorIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
        inline_subpixels: i64,
        block_subpixels: i64,
    ) -> Option<Self> {
        if inline_subpixels < 0 || block_subpixels < 0 {
            None
        } else {
            Some(Self {
                identity,
                binding,
                inline_subpixels,
                block_subpixels,
            })
        }
    }

    pub(super) fn same_identity(self, other: Self) -> bool {
        self.identity == other.identity
    }

    pub(crate) const fn mounted_identity(
        self,
    ) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        match self.identity {
            UiScrollAnchorIdentity::Mounted(identity) => Some(identity),
            UiScrollAnchorIdentity::ApplicationItem(_) => None,
        }
    }

    pub(super) fn exact_basis(self, other: Self) -> bool {
        self == other
    }

    pub(super) const fn inline_subpixels(self) -> i64 {
        self.inline_subpixels
    }

    pub(super) const fn block_subpixels(self) -> i64 {
        self.block_subpixels
    }
}

impl UiScrollRebindRequest {
    pub(crate) const fn new(
        registration: super::UiScrollOwnerRegistration,
        successor_anchor: Option<UiScrollAnchor>,
        policy: UiScrollAnchorPolicy,
    ) -> Self {
        Self {
            registration,
            successor_anchor,
            policy,
        }
    }

    pub(super) const fn registration(self) -> super::UiScrollOwnerRegistration {
        self.registration
    }

    pub(super) const fn successor_anchor(self) -> Option<UiScrollAnchor> {
        self.successor_anchor
    }

    pub(super) const fn policy(self) -> UiScrollAnchorPolicy {
        self.policy
    }
}

impl UiScrollAnchorReconciliationReceipt {
    pub(super) const fn new(
        outcome: UiScrollAnchorReconciliationOutcome,
        offset: super::UiScrollOffset,
    ) -> Self {
        Self { outcome, offset }
    }

    #[cfg(test)]
    pub(crate) const fn outcome(self) -> UiScrollAnchorReconciliationOutcome {
        self.outcome
    }

    #[cfg(test)]
    pub(crate) const fn offset(self) -> super::UiScrollOffset {
        self.offset
    }
}
