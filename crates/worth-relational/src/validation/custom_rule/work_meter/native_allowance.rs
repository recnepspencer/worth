use super::CustomInvariantWorkMeter;

impl CustomInvariantWorkMeter {
    pub(crate) fn try_debit_native_work(&self, units: u64) -> bool {
        self.native_debit().try_debit_native_work(units)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::custom_rule::work_meter::native_rule_debit::CustomPreparationStop;
    use crate::validation::custom_rule::CustomPreparationBudget;
    use std::num::{NonZeroU64, NonZeroUsize};
    use worth_execution::{CancellationToken, LeaseRequest};
    use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };

    #[test]
    fn native_debit_preserves_rule_limit_without_rebilling_leased_custom_counter() {
        let lease = crate::tests::support::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    ExecutionPosture::Serial,
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(NonZeroUsize::MIN, Some(256 * 1024), Some(100)),
                ),
                cancellation: CancellationToken::new(),
                deadline: None,
            })
            .unwrap();
        let request = CustomPreparationBudget::tracked(&lease).unwrap();
        let rule = CustomInvariantWorkMeter::new(NonZeroU64::new(60)).with_lease(&request);
        assert!(rule.try_debit_native_work(60));
        assert_eq!(rule.charged_work(), 60);
        assert_eq!(request.charged_work(), 0);
        assert!(!rule.try_debit_native_work(1));
        assert_eq!(
            rule.leased_stop(),
            Some(CustomPreparationStop::WorkExhausted)
        );
        assert_eq!(rule.charged_work(), 60);
        assert_eq!(request.charged_work(), 0);
        drop(rule);
        drop(request);
        assert_eq!(lease.charged_memory_bytes(), 0);
    }
}
