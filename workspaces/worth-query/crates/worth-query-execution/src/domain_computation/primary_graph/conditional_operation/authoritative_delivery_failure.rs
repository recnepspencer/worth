//! Native refusals while delivering authoritative commits for reconsideration.

/// The native outcome that stopped authoritative change delivery during reconsideration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryConditionalAuthoritativeDeliveryFailure {
    Admission(worth_runtime_bridge::facade::BridgeConditionalDenial),
    Denied(worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryDenial),
    Failed(worth_runtime_bridge::facade::BridgeCorrespondenceAdmissionFailure),
    Deferred(worth_runtime_bridge::facade::BridgeCorrespondenceDeferred),
    Stale(worth_runtime_bridge::facade::BridgeCorrespondenceStale),
    RebindRequired(worth_runtime_bridge::facade::BridgeCorrespondenceRebindRequired),
}

pub(super) fn delivered_receipt(
    outcome: worth_runtime_bridge::facade::CorrespondenceDeliveryOutcome,
) -> Result<
    worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
    WorthQueryConditionalAuthoritativeDeliveryFailure,
> {
    use worth_proof::TransitionOutcome;
    use WorthQueryConditionalAuthoritativeDeliveryFailure as Failure;
    match outcome {
        TransitionOutcome::Success(receipt) => Ok(receipt),
        TransitionOutcome::Denied(cause) => Err(Failure::Denied(cause)),
        TransitionOutcome::Failed(cause) => Err(Failure::Failed(cause)),
        TransitionOutcome::Deferred(cause) => Err(Failure::Deferred(cause)),
        TransitionOutcome::Stale(cause) => Err(Failure::Stale(cause)),
        TransitionOutcome::RebindRequired(cause) => Err(Failure::RebindRequired(cause)),
    }
}
