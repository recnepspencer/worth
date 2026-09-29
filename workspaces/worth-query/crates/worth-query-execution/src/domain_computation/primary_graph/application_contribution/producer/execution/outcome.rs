use super::denial::{denial, failed};
use super::{WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationNoEffectCause,
};

/// Maps a producer commit to its receipt or to the demand denial it settles as.
pub(super) fn commit_receipt(
    identity: &str,
    outcome: WorthQueryApplicationCommitOutcome,
) -> Result<WorthQueryApplicationCommitReceipt, WorthQueryOutputDemandDenial> {
    match outcome {
        WorthQueryApplicationCommitOutcome::Committed(receipt)
        | WorthQueryApplicationCommitOutcome::AlreadyCommitted(receipt) => Ok(receipt),
        WorthQueryApplicationCommitOutcome::Stale(_)
        | WorthQueryApplicationCommitOutcome::ProductStale(_) => Err(denial(
            WorthQueryOutputDemandDenialKind::PublicationStale,
            identity,
        )),
        WorthQueryApplicationCommitOutcome::Cancelled => Err(denial(
            WorthQueryOutputDemandDenialKind::Cancelled,
            identity,
        )),
        WorthQueryApplicationCommitOutcome::TimedOut => {
            Err(denial(WorthQueryOutputDemandDenialKind::TimedOut, identity))
        }
        WorthQueryApplicationCommitOutcome::NoEffect(no_effect)
            if no_effect.cause() == WorthQueryApplicationNoEffectCause::CapacityExhausted =>
        {
            Err(denial(
                WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded,
                identity,
            ))
        }
        WorthQueryApplicationCommitOutcome::Denied(commit_denial)
            if commit_denial.kind() == WorthQueryApplicationCommitDenialKind::ProductBasisStale =>
        {
            Err(denial(
                WorthQueryOutputDemandDenialKind::PublicationStale,
                identity,
            ))
        }
        WorthQueryApplicationCommitOutcome::Denied(commit_denial)
            if commit_denial.kind()
                == WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift =>
        {
            Err(denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                format!("{}: {commit_denial:?}", identity),
            ))
        }
        outcome => Err(failed(identity, outcome)),
    }
}
