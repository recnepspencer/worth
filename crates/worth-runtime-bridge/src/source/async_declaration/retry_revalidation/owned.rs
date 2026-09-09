use worth_signal::facade::{
    ResourceRetryAdmissionReport, ResourceRetryReason, ResourceRetryScheduleReport,
    ResourceRevalidationDenialClass, ResourceRevalidationReport, ResourceTimeoutReport,
};

use super::class::BridgeAsyncForwardCausalityClass;
use super::classification::finalize_revalidation_lineage;
use super::lineage::{BridgeAsyncRetryLineage, BridgeAsyncRevalidationLineage};
use super::rejection::{
    rejected, BridgeAsyncForwardCausalityRejection, BridgeAsyncForwardCausalityRejectionKind,
};
use super::retry_finalization::{finalize_retry_lineage, BridgeAsyncRetryLineageCandidate};
use crate::source::AdmittedBridgeAsyncRequestIdentity;

pub(crate) fn admit_owned_retry_lineage(
    prior: AdmittedBridgeAsyncRequestIdentity,
    newer: AdmittedBridgeAsyncRequestIdentity,
    timeout_report: &ResourceTimeoutReport,
    retry_schedule_report: &ResourceRetryScheduleReport,
    retry_admission_report: &ResourceRetryAdmissionReport,
) -> Result<BridgeAsyncRetryLineage, BridgeAsyncForwardCausalityRejection> {
    let scheduled = retry_schedule_report
        .scheduled_retry()
        .cloned()
        .ok_or_else(|| {
            rejected(
                BridgeAsyncForwardCausalityRejectionKind::RetryScheduleMissing,
                "timeout-triggered retry requires one admitted retry schedule",
            )
        })?;
    let admitted_retry = retry_admission_report
        .admitted_retry()
        .cloned()
        .ok_or_else(|| {
            rejected(
                BridgeAsyncForwardCausalityRejectionKind::RetryAdmissionMissing,
                "timeout-triggered retry requires one admitted retry request",
            )
        })?;
    let timed_out = timeout_report.timed_out_request().ok_or_else(|| {
        rejected(
            BridgeAsyncForwardCausalityRejectionKind::TimeoutEvidenceMissing,
            "timeout-triggered retry requires an admitted timeout report",
        )
    })?;
    if scheduled.previous() != prior.request_handle()
        || admitted_retry.scheduled().previous() != prior.request_handle()
        || timed_out.handle() != prior.request_handle()
        || admitted_retry.admitted_request().handle() != newer.request_handle()
    {
        return Err(rejected(
            BridgeAsyncForwardCausalityRejectionKind::PriorAndNewerSignalHandleMismatch,
            "owned retry evidence must retain the exact prior and newer Signal handles",
        ));
    }
    if scheduled.reason() != ResourceRetryReason::TimedOut {
        return Err(rejected(
            BridgeAsyncForwardCausalityRejectionKind::RetryScheduleMissing,
            "timeout-triggered retry must retain a timed-out retry reason",
        ));
    }
    finalize_retry_lineage(BridgeAsyncRetryLineageCandidate {
        prior,
        newer,
        class: BridgeAsyncForwardCausalityClass::RetryAfterTimeout,
        retry_reason: scheduled.reason(),
        retry_ordinal: scheduled.retry_ordinal().get(),
        next_attempt: admitted_retry.admitted_request().attempt().get(),
        ready_wake: admitted_retry.ready_wake().id().get(),
        policy_digest: scheduled.policy_decision_digest().as_str(),
        timeout_trigger: timed_out.ready_wake().id().get().to_string(),
        cancellation_trigger: "-".to_owned(),
    })
}

pub(crate) fn admit_owned_revalidation_lineage(
    prior: AdmittedBridgeAsyncRequestIdentity,
    newer: AdmittedBridgeAsyncRequestIdentity,
    resource_report: &ResourceRevalidationReport,
) -> Result<BridgeAsyncRevalidationLineage, BridgeAsyncForwardCausalityRejection> {
    if let Some(denied) = resource_report.denied_revalidation() {
        return match denied.class() {
            ResourceRevalidationDenialClass::ExpectedActiveRequestMismatch
            | ResourceRevalidationDenialClass::ActiveHandleProofMismatch => Err(rejected(
                BridgeAsyncForwardCausalityRejectionKind::StaleSignalGenerationRejected,
                "revalidation lineage cannot admit from stale expected-active signal generation",
            )),
            _ => Err(rejected(
                BridgeAsyncForwardCausalityRejectionKind::RevalidationAdmissionMissing,
                "revalidation report denied before bridge lineage could classify it",
            )),
        };
    }
    finalize_revalidation_lineage(prior, newer, resource_report)
}
