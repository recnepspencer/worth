use worth_signal::facade::ResourceCompletionAdmissionReport;

use super::super::request_identity::state::BridgeSignalRuntime;
use super::super::{AdmittedBridgeAsyncRequestIdentity, BridgeAsyncRequestFamilyAdmission};
use super::admitted::AdmittedBridgeAsyncCompletion;
use super::completion::BridgeAsyncCompletionAdmissionReport;
use super::counters::BridgeAsyncCompletionCounters;
use super::denied::BridgeAsyncDeniedCompletion;
use super::envelope::ValidatedBridgeAsyncCompletionEnvelope;
use super::rejection::{BridgeAsyncCompletionRejection, BridgeAsyncCompletionRejectionKind};

impl ValidatedBridgeAsyncCompletionEnvelope {
    pub fn admit(
        &self,
        signal_runtime: &mut BridgeSignalRuntime,
        request_identity: &AdmittedBridgeAsyncRequestIdentity,
    ) -> Result<BridgeAsyncCompletionAdmissionReport, BridgeAsyncCompletionRejection> {
        if request_identity.request_identity().as_str() != self.request_identity() {
            return Err(BridgeAsyncCompletionRejection::new(
                BridgeAsyncCompletionRejectionKind::EnvelopeHandleMismatch,
                format!(
                    "bridge async completion envelope `{}` was validated for request `{}` and cannot admit against `{}`",
                    self.envelope().envelope_identity().as_str(),
                    self.request_identity(),
                    request_identity.request_identity().as_str(),
                ),
            ));
        }

        let signal_report = signal_runtime.admit_resource_completion(self.raw());
        Ok(map_owned_signal_report(
            request_identity.clone(),
            self.clone(),
            signal_report,
        ))
    }
}

pub(crate) fn map_owned_signal_report(
    request_identity: AdmittedBridgeAsyncRequestIdentity,
    validated_envelope: ValidatedBridgeAsyncCompletionEnvelope,
    signal_report: ResourceCompletionAdmissionReport,
) -> BridgeAsyncCompletionAdmissionReport {
    match (
        signal_report.admitted_completion(),
        signal_report.denied_completion(),
        request_identity.family_admission(),
    ) {
        (Some(admitted), None, BridgeAsyncRequestFamilyAdmission::RequestResponse) => {
            BridgeAsyncCompletionAdmissionReport::admitted(AdmittedBridgeAsyncCompletion::new(
                request_identity,
                validated_envelope,
                admitted,
                BridgeAsyncCompletionCounters::admitted_request_response(),
            ))
        }
        (Some(admitted), None, BridgeAsyncRequestFamilyAdmission::SubscriptionBacked { .. }) => {
            BridgeAsyncCompletionAdmissionReport::admitted(AdmittedBridgeAsyncCompletion::new(
                request_identity,
                validated_envelope,
                admitted,
                BridgeAsyncCompletionCounters::admitted_subscription_backed(),
            ))
        }
        (None, Some(denied), BridgeAsyncRequestFamilyAdmission::RequestResponse) => {
            BridgeAsyncCompletionAdmissionReport::denied(BridgeAsyncDeniedCompletion::new(
                request_identity,
                validated_envelope,
                denied,
                BridgeAsyncCompletionCounters::denied_request_response(),
            ))
        }
        (None, Some(denied), BridgeAsyncRequestFamilyAdmission::SubscriptionBacked { .. }) => {
            BridgeAsyncCompletionAdmissionReport::denied(BridgeAsyncDeniedCompletion::new(
                request_identity,
                validated_envelope,
                denied,
                BridgeAsyncCompletionCounters::denied_subscription_backed(),
            ))
        }
        _ => unreachable!("signal resource completion admission must return exactly one outcome"),
    }
}
