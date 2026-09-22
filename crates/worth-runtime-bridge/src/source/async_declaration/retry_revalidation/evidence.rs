use std::sync::Arc;

use worth_signal::facade::{
    AsyncNodeRevalidationReport, ResourceCancellationReport, ResourceRetryAdmissionReport,
    ResourceRetryScheduleReport, ResourceRevalidationReport, ResourceTimeoutReport,
};

use super::super::request_identity::{
    AdmittedBridgeAsyncRequestIdentity, BridgeAsyncRequestSubscriptionInstance,
    BridgeAsyncRequestTruthViewBasis,
};

#[derive(Debug, Clone)]
pub struct BridgeAsyncRetryLineageRequest {
    pub(crate) prior: AdmittedBridgeAsyncRequestIdentity,
    pub(crate) newer_request: Option<AdmittedBridgeAsyncRequestIdentity>,
    pub(crate) timeout_report: Option<ResourceTimeoutReport>,
    pub(crate) cancellation_report: Option<ResourceCancellationReport>,
    pub(crate) retry_schedule_report: Option<ResourceRetryScheduleReport>,
    pub(crate) retry_admission_report: Option<ResourceRetryAdmissionReport>,
}

impl BridgeAsyncRetryLineageRequest {
    pub(crate) fn validate_runtime(
        &self,
        key: u64,
    ) -> Result<(), super::BridgeAsyncForwardCausalityRejection> {
        super::runtime_binding::validate_lineage_runtime(key, &self.prior)?;
        if let Some(newer) = &self.newer_request {
            super::runtime_binding::validate_lineage_runtime(key, newer)?;
        }
        Ok(())
    }

    pub fn after_timeout(
        prior: &AdmittedBridgeAsyncRequestIdentity,
        timeout_report: &ResourceTimeoutReport,
        retry_schedule_report: &ResourceRetryScheduleReport,
        retry_admission_report: &ResourceRetryAdmissionReport,
    ) -> Self {
        Self {
            prior: prior.clone(),
            newer_request: None,
            timeout_report: Some(timeout_report.clone()),
            cancellation_report: None,
            retry_schedule_report: Some(retry_schedule_report.clone()),
            retry_admission_report: Some(retry_admission_report.clone()),
        }
    }

    pub fn after_cancellation(
        prior: &AdmittedBridgeAsyncRequestIdentity,
        cancellation_report: &ResourceCancellationReport,
        newer_request: &AdmittedBridgeAsyncRequestIdentity,
    ) -> Self {
        Self {
            prior: prior.clone(),
            newer_request: Some(newer_request.clone()),
            timeout_report: None,
            cancellation_report: Some(cancellation_report.clone()),
            retry_schedule_report: None,
            retry_admission_report: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BridgeAsyncRevalidationSignalReport {
    ResourceOnly(ResourceRevalidationReport),
    SubscriptionBacked {
        async_report: AsyncNodeRevalidationReport,
        resource_report: ResourceRevalidationReport,
    },
}

impl BridgeAsyncRevalidationSignalReport {
    pub(crate) fn resource_report(&self) -> &ResourceRevalidationReport {
        match self {
            Self::ResourceOnly(report) => report,
            Self::SubscriptionBacked {
                resource_report, ..
            } => resource_report,
        }
    }

    pub(crate) fn async_decision_digest(&self) -> Option<Arc<str>> {
        match self {
            Self::ResourceOnly(_) => None,
            Self::SubscriptionBacked { async_report, .. } => Some(Arc::from(
                async_report
                    .classification()
                    .decision_digest()
                    .as_str()
                    .to_owned(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BridgeAsyncRevalidationLineageRequest {
    pub(crate) prior: AdmittedBridgeAsyncRequestIdentity,
    pub(crate) current_truth_view_basis: BridgeAsyncRequestTruthViewBasis,
    pub(crate) current_subscription_instance: Option<BridgeAsyncRequestSubscriptionInstance>,
    pub(crate) signal_report: BridgeAsyncRevalidationSignalReport,
}

impl BridgeAsyncRevalidationLineageRequest {
    pub fn request_response(
        prior: &AdmittedBridgeAsyncRequestIdentity,
        current_truth_view_basis: BridgeAsyncRequestTruthViewBasis,
        resource_report: &ResourceRevalidationReport,
    ) -> Self {
        Self {
            prior: prior.clone(),
            current_truth_view_basis,
            current_subscription_instance: None,
            signal_report: BridgeAsyncRevalidationSignalReport::ResourceOnly(
                resource_report.clone(),
            ),
        }
    }

    pub fn subscription_backed(
        prior: &AdmittedBridgeAsyncRequestIdentity,
        current_truth_view_basis: BridgeAsyncRequestTruthViewBasis,
        current_subscription_instance: BridgeAsyncRequestSubscriptionInstance,
        async_report: &AsyncNodeRevalidationReport,
    ) -> Self {
        let resource_report = async_report
            .resource_revalidation()
            .expect("subscription-backed revalidation should retain resource revalidation")
            .clone();
        Self {
            prior: prior.clone(),
            current_truth_view_basis,
            current_subscription_instance: Some(current_subscription_instance),
            signal_report: BridgeAsyncRevalidationSignalReport::SubscriptionBacked {
                async_report: async_report.clone(),
                resource_report,
            },
        }
    }

    pub fn subscription_backed_resource_only(
        prior: &AdmittedBridgeAsyncRequestIdentity,
        current_truth_view_basis: BridgeAsyncRequestTruthViewBasis,
        current_subscription_instance: BridgeAsyncRequestSubscriptionInstance,
        resource_report: &ResourceRevalidationReport,
    ) -> Self {
        Self {
            prior: prior.clone(),
            current_truth_view_basis,
            current_subscription_instance: Some(current_subscription_instance),
            signal_report: BridgeAsyncRevalidationSignalReport::ResourceOnly(
                resource_report.clone(),
            ),
        }
    }
}
