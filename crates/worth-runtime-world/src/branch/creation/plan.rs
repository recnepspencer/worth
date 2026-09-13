use worth_relational::facade::history::BranchId;
use worth_signal::facade::branch::ValidatedSignalBranchName;

/// Relational posture for one product-branch creation. Fork carries its own
/// owner-issued destination; there is no third advance variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchCreationPlan {
    ReuseExact,
    ForkExact { target: BranchId },
}

impl RelationalBranchCreationPlan {
    pub const fn is_reuse_exact(&self) -> bool {
        matches!(self, Self::ReuseExact)
    }

    pub const fn requires_owner_effect(&self) -> bool {
        !self.is_reuse_exact()
    }

    pub const fn fork_target(&self) -> Option<&BranchId> {
        match self {
            Self::ReuseExact => None,
            Self::ForkExact { target } => Some(target),
        }
    }
}

/// Signal posture for one product-branch creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalBranchCreationPlan {
    ReuseExact,
    ForkExact { target: ValidatedSignalBranchName },
}

impl SignalBranchCreationPlan {
    pub const fn is_reuse_exact(&self) -> bool {
        matches!(self, Self::ReuseExact)
    }

    pub const fn requires_owner_effect(&self) -> bool {
        !self.is_reuse_exact()
    }

    pub const fn fork_target(&self) -> Option<&ValidatedSignalBranchName> {
        match self {
            Self::ReuseExact => None,
            Self::ForkExact { target } => Some(target),
        }
    }
}

/// Complete two-by-two creation cell. Omission is not representable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductBranchCreationPlans {
    relational: RelationalBranchCreationPlan,
    signal: SignalBranchCreationPlan,
}

impl ProductBranchCreationPlans {
    pub const fn new(
        relational: RelationalBranchCreationPlan,
        signal: SignalBranchCreationPlan,
    ) -> Self {
        Self { relational, signal }
    }

    pub const fn relational(&self) -> &RelationalBranchCreationPlan {
        &self.relational
    }

    pub const fn signal(&self) -> &SignalBranchCreationPlan {
        &self.signal
    }

    /// Reuse on both owners selects the existing commit without any owner
    /// movement.
    pub const fn is_exact_reuse(&self) -> bool {
        self.relational.is_reuse_exact() && self.signal.is_reuse_exact()
    }

    pub const fn requires_relational_owner_effect(&self) -> bool {
        self.relational.requires_owner_effect()
    }

    pub const fn requires_signal_owner_effect(&self) -> bool {
        self.signal.requires_owner_effect()
    }
}
