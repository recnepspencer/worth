use crate::branch::observation::{ProductBranchObservation, RuntimeWorldBranchAdmissionDenial};

use super::plan::{RelationalBranchCreationPlan, SignalBranchCreationPlan};
use super::ProductBranchCreationIntent;

/// Owner-admitted creation plan: the source composite basis pinned against the
/// two owner postures. Distinct from `LoweredOwnerComponentPlan`; it never
/// carries a prepared candidate or a mutation closure.
#[derive(Debug)]
#[must_use = "a lowered creation plan is executed or dropped"]
pub(crate) struct LoweredBranchCreationPlan {
    expected: ProductBranchObservation,
    relational: RelationalBranchCreationPlan,
    signal: SignalBranchCreationPlan,
}

impl LoweredBranchCreationPlan {
    pub(crate) fn lower(
        source: ProductBranchObservation,
        intent: ProductBranchCreationIntent,
    ) -> Result<Self, RuntimeWorldBranchAdmissionDenial> {
        let (_, plans) = intent.into_parts();
        let plans = plans.ok_or(RuntimeWorldBranchAdmissionDenial::PlansOmitted)?;
        let relational = plans.relational().clone();
        let signal = plans.signal().clone();
        Ok(Self {
            expected: source,
            relational,
            signal,
        })
    }

    pub(crate) const fn expected(&self) -> &ProductBranchObservation {
        &self.expected
    }

    pub(crate) const fn relational(&self) -> &RelationalBranchCreationPlan {
        &self.relational
    }

    pub(crate) const fn signal(&self) -> &SignalBranchCreationPlan {
        &self.signal
    }
}
