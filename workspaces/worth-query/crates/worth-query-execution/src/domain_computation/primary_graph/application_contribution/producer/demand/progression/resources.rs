use super::super::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandRecoveryPosture,
};
use super::denial;

pub(super) fn validate_demand_resources<Schema>(
    executor: &dyn super::super::super::InstalledProducerExecutor<Schema>,
    source: &dyn std::any::Any,
    producer_identity: &str,
    limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
) -> Result<super::super::super::WorthQueryProducerDemandResources, WorthQueryOutputDemandDenial> {
    let resources = executor.resources(source).ok_or_else(|| {
        denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            producer_identity,
        )
    })?;
    validate_retained_resources(resources, producer_identity, limits)?;
    Ok(resources)
}

pub(super) fn validate_retained_resources(
    resources: super::super::super::WorthQueryProducerDemandResources,
    producer_identity: &str,
    limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
) -> Result<(), WorthQueryOutputDemandDenial> {
    if resources.work() > limits.producer_work() {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            producer_identity,
        )
        .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable));
    }
    if resources.retained_bytes() > limits.producer_retained_bytes() {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            producer_identity,
        )
        .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::{
        primary_graph::WorthQueryProducerDemandResources, WorthQueryOutputDemandResourceProfile,
    };

    #[test]
    fn child_resource_admission_uses_producer_allowance_not_source_scan_allowance() {
        let child = WorthQueryOutputDemandResourceProfile::standard()
            .limits()
            .for_artifact(7, 11);
        assert!(validate_retained_resources(
            WorthQueryProducerDemandResources::new(7, 11),
            "child",
            child,
        )
        .is_ok());
        assert_eq!(
            validate_retained_resources(
                WorthQueryProducerDemandResources::new(8, 11),
                "child",
                child,
            )
            .unwrap_err()
            .kind(),
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        );
        assert_eq!(
            validate_retained_resources(
                WorthQueryProducerDemandResources::new(7, 12),
                "child",
                child,
            )
            .unwrap_err()
            .kind(),
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded
        );
        assert_eq!(child.source_currentness_work(), 4_194_304);
    }
}
