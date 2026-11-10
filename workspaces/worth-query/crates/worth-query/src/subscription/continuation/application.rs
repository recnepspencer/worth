use super::super::active_counters::ActiveSubscriptionCounters;
use super::super::continuation_error::{
    SubscriptionContinuationDenialKind, SubscriptionContinuationError,
};
use super::super::delivery_dimensions::MaintenanceDeltaWidth;
use super::super::delivery_window::QueryDeliveryWindow;
use super::super::maintenance_delta::{
    QuerySubscriptionMaintenanceDelta, QuerySubscriptionMaintenanceDeltaKind,
};
use super::class::SubscriptionContinuationClass;
use super::evidence::SubscriptionContinuationEvidence;
use super::report::SubscriptionContinuationReport;

pub fn apply_subscription_continuation(
    window: QueryDeliveryWindow,
    evidence: SubscriptionContinuationEvidence,
) -> Result<(QueryDeliveryWindow, SubscriptionContinuationReport), SubscriptionContinuationError> {
    let mut counters = ActiveSubscriptionCounters::default();
    if window.active_lane_digest() != evidence.active_lane_digest() {
        counters.continuation_remap_denial_count = 1;
        return Err(SubscriptionContinuationError::new(
            SubscriptionContinuationDenialKind::ContinuationEvidenceMismatch,
            "continuation evidence must target the delivery window lane",
            evidence.evidence_identity().clone(),
            counters,
        ));
    }
    let report = SubscriptionContinuationReport::new(&evidence);
    let window = window.apply_continuation(&report);
    Ok((window, report))
}

pub fn lower_subscription_continuation_report(
    report: &SubscriptionContinuationReport,
) -> (
    QuerySubscriptionMaintenanceDelta,
    ActiveSubscriptionCounters,
) {
    let mut counters = ActiveSubscriptionCounters::default();
    counters.continuation_remap_width = report.remap_width();
    match report.continuation_class() {
        SubscriptionContinuationClass::CorrespondenceAdvisory => {
            counters.continuation_advisory_count = 1;
        }
        SubscriptionContinuationClass::IdentityBreak => {
            counters.continuation_identity_break_count = 1;
        }
        SubscriptionContinuationClass::IdentityRemap
        | SubscriptionContinuationClass::CollectionMembershipRemap
        | SubscriptionContinuationClass::GroupedMembershipRemap
        | SubscriptionContinuationClass::PreviewPromotionRemap => {
            counters.continuation_remap_count = 1;
        }
        SubscriptionContinuationClass::UnsupportedContinuation => {
            counters.continuation_remap_denial_count = 1;
        }
    }
    let delta = QuerySubscriptionMaintenanceDelta::admitted(
        QuerySubscriptionMaintenanceDeltaKind::ContinuationDelta,
        report.active_lane_digest().clone(),
        report.evidence_identity(),
        MaintenanceDeltaWidth::measured(report.remap_width()),
    );
    (delta, counters)
}
