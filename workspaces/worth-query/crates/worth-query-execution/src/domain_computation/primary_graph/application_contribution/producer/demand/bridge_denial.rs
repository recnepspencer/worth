use worth_runtime_bridge::facade::{BridgeConditionalDenial, BridgeConditionalDenialKind};

use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};

pub(super) fn bridge_denial(
    producer_identity: &str,
    failure: BridgeConditionalDenial,
) -> WorthQueryOutputDemandDenial {
    let kind = bridge_denial_kind(&failure);
    WorthQueryOutputDemandDenial::new(
        kind,
        format!(
            "{producer_identity}: conditional {:?}: {}",
            failure.kind(),
            failure.detail()
        ),
    )
}

pub(super) fn bridge_denial_kind(
    failure: &BridgeConditionalDenial,
) -> WorthQueryOutputDemandDenialKind {
    if matches!(
        failure.kind(),
        BridgeConditionalDenialKind::ConditionalRetentionCapacity
            | BridgeConditionalDenialKind::ConditionalEvaluationBusy
            | BridgeConditionalDenialKind::ConditionalEvaluationUnwindPending
            | BridgeConditionalDenialKind::ConditionalEvaluationAdmissionCapacity
    ) {
        WorthQueryOutputDemandDenialKind::SchedulingDeferred
    } else {
        WorthQueryOutputDemandDenialKind::SchedulingRejected
    }
}
