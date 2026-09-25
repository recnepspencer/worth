use std::sync::Arc;

use worth_signal::facade::ResourceRetryReason;

use super::class::BridgeAsyncForwardCausalityClass;
use super::counters::BridgeAsyncForwardCausalityCounters;
use super::lineage::BridgeAsyncRetryLineage;
use super::rejection::{
    rejected, BridgeAsyncForwardCausalityRejection, BridgeAsyncForwardCausalityRejectionKind,
};
use crate::source::async_declaration::request_identity::{
    AdmittedBridgeAsyncRequestIdentity, BridgeAsyncRequestFamilyAdmission,
};

pub(super) struct BridgeAsyncRetryLineageCandidate<'a> {
    pub prior: AdmittedBridgeAsyncRequestIdentity,
    pub newer: AdmittedBridgeAsyncRequestIdentity,
    pub class: BridgeAsyncForwardCausalityClass,
    pub retry_reason: ResourceRetryReason,
    pub retry_ordinal: u64,
    pub next_attempt: u64,
    pub ready_wake: u64,
    pub policy_digest: &'a str,
    pub timeout_trigger: String,
    pub cancellation_trigger: String,
}

pub(super) fn finalize_retry_lineage(
    candidate: BridgeAsyncRetryLineageCandidate<'_>,
) -> Result<BridgeAsyncRetryLineage, BridgeAsyncForwardCausalityRejection> {
    let BridgeAsyncRetryLineageCandidate {
        prior,
        newer,
        class,
        retry_reason,
        retry_ordinal,
        next_attempt,
        ready_wake,
        policy_digest,
        timeout_trigger,
        cancellation_trigger,
    } = candidate;
    let newer = newer
        .inherit_runtime(&prior)
        .ok_or_else(super::runtime_binding::runtime_mismatch)?;
    if prior.lowered().declaration_identity() != newer.lowered().declaration_identity() {
        return Err(rejected(
            BridgeAsyncForwardCausalityRejectionKind::PriorAndNewerDeclarationMismatch,
            "retry lineage must stay within one bridge async declaration identity",
        ));
    }
    if prior.lowered().lowering_identity() != newer.lowered().lowering_identity() {
        return Err(rejected(
            BridgeAsyncForwardCausalityRejectionKind::PriorAndNewerLoweringMismatch,
            "retry lineage must stay within one lowered bridge async source identity",
        ));
    }
    if prior.family_admission() != newer.family_admission()
        && !matches!(
            (prior.family_admission(), newer.family_admission()),
            (
                BridgeAsyncRequestFamilyAdmission::SubscriptionBacked { .. },
                BridgeAsyncRequestFamilyAdmission::SubscriptionBacked { .. }
            )
        )
    {
        return Err(rejected(
            BridgeAsyncForwardCausalityRejectionKind::PriorAndNewerFamilyMismatch,
            "retry lineage must stay within one bridge async family",
        ));
    }
    if prior.basis_binding().truth_view_basis().digest()
        != newer.basis_binding().truth_view_basis().digest()
    {
        return Err(rejected(
            BridgeAsyncForwardCausalityRejectionKind::BasisDriftForbiddenForRetry,
            "retry lineage cannot rebind to a different truth-view basis",
        ));
    }
    if prior.subscription_instance_digest() != newer.subscription_instance_digest() {
        return Err(rejected(
            BridgeAsyncForwardCausalityRejectionKind::SubscriptionInstanceDriftForbiddenForRetry,
            "retry lineage cannot drift subscription instance identity",
        ));
    }
    let counters = match class {
        BridgeAsyncForwardCausalityClass::RetryAfterTimeout => {
            BridgeAsyncForwardCausalityCounters::one_retry_after_timeout()
        }
        BridgeAsyncForwardCausalityClass::RetryAfterCancellation => {
            BridgeAsyncForwardCausalityCounters::one_retry_after_cancellation()
        }
        _ => unreachable!(),
    };
    Ok(BridgeAsyncRetryLineage::new(
        prior.clone(),
        newer.clone(),
        class,
        counters,
        Arc::from(format!(
            "bridge-async-forward-causality|class={class:?}|prior={}|newer={}|prior-basis={}|newer-basis={}|subscription-instance={}|retry-reason={:?}|retry-ordinal={}|next-attempt={}|ready-wake={}|policy={}|timeout-trigger={}|cancellation-trigger={}",
            prior.request_identity().as_str(),
            newer.request_identity().as_str(),
            prior.basis_binding().truth_view_basis().digest(),
            newer.basis_binding().truth_view_basis().digest(),
            prior.subscription_instance_digest().unwrap_or("-"),
            retry_reason,
            retry_ordinal,
            next_attempt,
            ready_wake,
            policy_digest,
            timeout_trigger,
            cancellation_trigger,
        )),
    ))
}
