use worth_foundational::facade::{
    performance, FoundationalPerformanceAccessPatternPosture, FoundationalPerformanceBoundary,
    FoundationalPerformanceBreadthLocalityPosture, FoundationalPerformanceBudgetKind,
    FoundationalPerformanceEvidenceStrength, FoundationalPerformanceExecutionTemperature,
    FoundationalPerformanceFallbackDebtPosture, FoundationalPerformanceFreshnessRetentionPosture,
    FoundationalPerformanceWorkClass, FoundationalPolicyAdmissionReceipt,
};

use super::PolicyValueLookupFailure;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PolicyValueLookupCounters {
    pub source_state_hit: u64,
    pub target_state_hit: u64,
    pub base_state_hit: u64,
    pub base_patch_authority_hit: u64,
    pub missing_ancestor_basis: u64,
    pub missing_visible_state: u64,
    pub invalid_shape: u64,
}

impl PolicyValueLookupCounters {
    pub(crate) fn accumulate(&mut self, work: &Self) {
        self.source_state_hit += work.source_state_hit;
        self.target_state_hit += work.target_state_hit;
        self.base_state_hit += work.base_state_hit;
        self.base_patch_authority_hit += work.base_patch_authority_hit;
        self.missing_ancestor_basis += work.missing_ancestor_basis;
        self.missing_visible_state += work.missing_visible_state;
        self.invalid_shape += work.invalid_shape;
    }

    pub(crate) fn record_source(&mut self, result: Result<(), PolicyValueLookupFailure>) {
        self.record_visible_state(result, true);
    }

    pub(crate) fn record_target(&mut self, result: Result<(), PolicyValueLookupFailure>) {
        self.record_visible_state(result, false);
    }

    pub(crate) fn record_base_state(&mut self, result: Result<(), PolicyValueLookupFailure>) {
        match result {
            Ok(()) => self.base_state_hit += 1,
            Err(PolicyValueLookupFailure::MissingRecordBasis)
            | Err(PolicyValueLookupFailure::MissingField) => {}
            Err(PolicyValueLookupFailure::InvalidValueShape) => self.invalid_shape += 1,
        }
    }

    pub(crate) fn record_base_patch_authority(&mut self, hit: bool) {
        if hit {
            self.base_patch_authority_hit += 1;
        } else {
            self.missing_ancestor_basis += 1;
        }
    }

    fn record_visible_state(&mut self, result: Result<(), PolicyValueLookupFailure>, source: bool) {
        match result {
            Ok(()) if source => self.source_state_hit += 1,
            Ok(()) => self.target_state_hit += 1,
            Err(PolicyValueLookupFailure::MissingRecordBasis) => self.missing_visible_state += 1,
            Err(PolicyValueLookupFailure::MissingField) => self.missing_visible_state += 1,
            Err(PolicyValueLookupFailure::InvalidValueShape) => self.invalid_shape += 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PolicyValueLookupReceipt {
    counters: PolicyValueLookupCounters,
    admission_receipt: Option<FoundationalPolicyAdmissionReceipt>,
}

impl PolicyValueLookupReceipt {
    pub(crate) fn from_counters(counters: PolicyValueLookupCounters) -> Self {
        Self {
            admission_receipt: build_foundational_policy_admission_receipt(),
            counters,
        }
    }

    pub(crate) fn counters(&self) -> &PolicyValueLookupCounters {
        &self.counters
    }

    pub(crate) fn admission_receipt(&self) -> Option<&FoundationalPolicyAdmissionReceipt> {
        self.admission_receipt.as_ref()
    }
}

fn build_foundational_policy_admission_receipt() -> Option<FoundationalPolicyAdmissionReceipt> {
    let claim = performance()
        .claim()
        .policy_admission()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::DeltaBound)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::WarmPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .ok()?;

    performance()
        .policy_admission_receipt(claim)
        .budget_decision(FoundationalPerformanceBudgetKind::Breadth, 1, 1)
        .finish()
        .ok()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn value_lookup_counters_distinguish_base_state_and_patch_authority() {
        let mut counters = PolicyValueLookupCounters::default();

        counters.record_base_state(Ok(()));
        counters.record_base_patch_authority(true);

        assert_eq!(counters.base_state_hit, 1);
        assert_eq!(counters.base_patch_authority_hit, 1);
        assert_eq!(counters.missing_ancestor_basis, 0);
    }

    #[test]
    fn value_lookup_counters_do_not_label_missing_visible_as_shape_error() {
        let mut counters = PolicyValueLookupCounters::default();

        counters.record_source(Err(PolicyValueLookupFailure::MissingField));

        assert_eq!(counters.missing_visible_state, 1);
        assert_eq!(counters.invalid_shape, 0);
    }

    #[test]
    fn value_lookup_receipt_carries_foundational_policy_admission_claim() {
        let receipt = PolicyValueLookupReceipt::from_counters(PolicyValueLookupCounters::default());

        assert!(receipt.admission_receipt().is_some());
    }
}
