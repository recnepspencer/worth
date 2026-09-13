use super::{RuntimeWorldIdentityExhaustion, RuntimeWorldOwnerIdentity};
use crate::branch::ProductBranchName;

/// Owner-issued identity of one product branch, keyed by its normalized name.
/// Retire and recreate of the same name yields the SAME identity with a NEW
/// incarnation; the incarnation, not the identity, distinguishes occurrences.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProductBranchIdentity {
    owner: RuntimeWorldOwnerIdentity,
    name: ProductBranchName,
}

impl ProductBranchIdentity {
    pub(crate) const fn issued(owner: RuntimeWorldOwnerIdentity, name: ProductBranchName) -> Self {
        Self { owner, name }
    }

    pub const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    pub const fn name(&self) -> &ProductBranchName {
        &self.name
    }
}

/// Occurrence of a product branch between creation and retirement. An
/// incarnation value is never reused, so retire-then-recreate of one name is
/// always distinguishable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProductBranchIncarnation {
    owner: RuntimeWorldOwnerIdentity,
    ordinal: u64,
}

impl ProductBranchIncarnation {
    pub(super) const fn issued(owner: RuntimeWorldOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }

    pub const fn owner_identity(self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    /// Descriptive lifecycle ordinal; this value cannot create a branch.
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}

/// Generation of one mutable product reference cell. It advances only after
/// a successful product-reference movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProductBranchReferenceGeneration(u64);

impl ProductBranchReferenceGeneration {
    pub(crate) const fn initial() -> Self {
        Self(0)
    }

    /// Descriptive reference generation; this value cannot issue or advance a head.
    pub const fn get(self) -> u64 {
        self.0
    }

    pub(crate) fn advance(self) -> Result<Self, RuntimeWorldIdentityExhaustion> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(RuntimeWorldIdentityExhaustion::new(
                super::RuntimeWorldIdentityFamily::ProductBranchReferenceGeneration,
            ))
    }

    pub(crate) fn retreat(self, steps: usize) -> Option<Self> {
        let steps = u64::try_from(steps).ok()?;
        self.0.checked_sub(steps).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::ProductBranchReferenceGeneration;
    use crate::identity::RuntimeWorldIdentityFamily;

    #[test]
    fn generation_advances_only_as_a_checked_transition() {
        let initial = ProductBranchReferenceGeneration::initial();
        assert_eq!(initial.get(), 0);
        assert_eq!(initial.advance().unwrap().get(), 1);

        let terminal = ProductBranchReferenceGeneration(u64::MAX);
        let denial = terminal.advance().expect_err("generation must not wrap");
        assert_eq!(
            denial.family(),
            RuntimeWorldIdentityFamily::ProductBranchReferenceGeneration
        );
    }
}
