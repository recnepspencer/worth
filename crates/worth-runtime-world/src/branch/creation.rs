#[path = "creation/plan.rs"]
mod plan;

#[path = "creation/lowered.rs"]
mod lowered;

pub use plan::{
    ProductBranchCreationPlans, RelationalBranchCreationPlan, SignalBranchCreationPlan,
};

pub(crate) use lowered::LoweredBranchCreationPlan;

use super::name::{ProductBranchName, ProductBranchNameDenial};

/// Explicit branch-creation meaning. A name is never promoted to a branch
/// identity without the Runtime World owner issuing that identity.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "a creation intent is submitted or dropped"]
pub struct ProductBranchCreationIntent {
    name: ProductBranchName,
    plans: Option<ProductBranchCreationPlans>,
}

impl ProductBranchCreationIntent {
    /// Bootstrap form: names the root product reference. It carries no source
    /// and no component postures, so it cannot create a branch from a source.
    pub fn named(name: impl Into<String>) -> Result<Self, ProductBranchNameDenial> {
        Ok(Self {
            name: ProductBranchName::try_new(name)?,
            plans: None,
        })
    }

    /// Creation-from-source form: exactly one explicit posture per component.
    pub fn from_source(
        name: impl Into<String>,
        plans: ProductBranchCreationPlans,
    ) -> Result<Self, ProductBranchNameDenial> {
        Ok(Self {
            name: ProductBranchName::try_new(name)?,
            plans: Some(plans),
        })
    }

    pub fn name(&self) -> &ProductBranchName {
        &self.name
    }

    pub fn plans(&self) -> Option<&ProductBranchCreationPlans> {
        self.plans.as_ref()
    }

    pub(crate) fn into_parts(self) -> (ProductBranchName, Option<ProductBranchCreationPlans>) {
        (self.name, self.plans)
    }
}

#[cfg(test)]
#[path = "creation_tests.rs"]
mod tests;
