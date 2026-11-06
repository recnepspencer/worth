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
    source_kind: super::BranchCreationSourceKind,
    retained_native: Option<worth_relational::facade::branch::AdmittedRelationalRetainedForkSource>,
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
            source_kind: super::BranchCreationSourceKind::Current,
            retained_native: None,
        })
    }

    pub(crate) fn from_retained(
        mut self,
        source: worth_relational::facade::branch::AdmittedRelationalRetainedForkSource,
    ) -> Self {
        self.source_kind = super::BranchCreationSourceKind::Retained;
        self.retained_native = Some(source);
        self
    }
    pub(crate) fn source_kind(&self) -> super::BranchCreationSourceKind {
        self.source_kind
    }
    pub(crate) fn take_retained_native(
        &mut self,
    ) -> Option<worth_relational::facade::branch::AdmittedRelationalRetainedForkSource> {
        self.retained_native.take()
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
