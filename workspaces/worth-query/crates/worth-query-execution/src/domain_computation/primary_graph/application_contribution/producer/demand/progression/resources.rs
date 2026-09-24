use super::super::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandRecoveryPosture,
};
use super::denial;

pub(super) fn validate_demand_resources<Schema>(
    executor: &dyn super::super::super::InstalledProducerExecutor<Schema>,
    source: &dyn std::any::Any,
    producer_identity: &str,
    maximum_work: usize,
    maximum_retained_bytes: usize,
) -> Result<super::super::super::WorthQueryProducerDemandResources, WorthQueryOutputDemandDenial> {
    let resources = executor.resources(source).ok_or_else(|| {
        denial(
            WorthQueryOutputDemandDenialKind::ForeignSource,
            producer_identity,
        )
    })?;
    validate_retained_resources(
        resources,
        producer_identity,
        maximum_work,
        maximum_retained_bytes,
    )?;
    Ok(resources)
}

pub(super) fn validate_retained_resources(
    resources: super::super::super::WorthQueryProducerDemandResources,
    producer_identity: &str,
    maximum_work: usize,
    maximum_retained_bytes: usize,
) -> Result<(), WorthQueryOutputDemandDenial> {
    if resources.work() > maximum_work {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            producer_identity,
        )
        .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable));
    }
    if resources.retained_bytes() > maximum_retained_bytes {
        return Err(denial(
            WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            producer_identity,
        )
        .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable));
    }
    Ok(())
}
