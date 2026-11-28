use super::BridgeManagedTemporalDenial;
use super::BridgeManagedTemporalDenialKind;
use worth_execution::report::{
    DenialClass::{self, *},
    Disposition::{self, *},
    ReportCode, Reportable,
};

impl std::fmt::Display for BridgeManagedTemporalDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Bridge denied the managed clock operation.")
    }
}

impl std::error::Error for BridgeManagedTemporalDenial {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.signal_cause
            .as_deref()
            .map(|cause| cause as &dyn std::error::Error)
    }
}

impl Reportable for BridgeManagedTemporalDenial {
    fn denial_class(&self) -> DenialClass {
        report(self).0
    }

    fn disposition(&self) -> Disposition {
        report(self).1
    }

    fn report_code(&self) -> ReportCode {
        ReportCode::new(report(self).2).expect("valid static Bridge report code")
    }

    fn diagnostic_summary(&self) -> &'static str {
        "Bridge denied the managed clock operation."
    }
}

fn report(denial: &BridgeManagedTemporalDenial) -> (DenialClass, Disposition, &'static str) {
    match denial.kind() {
        BridgeManagedTemporalDenialKind::InvalidContract => (
            InvalidInput,
            CorrectInput,
            "bridge.managed_time.invalid_contract",
        ),
        BridgeManagedTemporalDenialKind::DuplicateClockBinding => (
            Conflict,
            Final,
            "bridge.managed_time.duplicate_clock_binding",
        ),
        BridgeManagedTemporalDenialKind::ForeignClockBinding => (
            Authorization,
            Reauthorize,
            "bridge.managed_time.foreign_clock_binding",
        ),
        BridgeManagedTemporalDenialKind::ClosedClockBinding => (
            Unavailable,
            Escalate,
            "bridge.managed_time.closed_clock_binding",
        ),
        BridgeManagedTemporalDenialKind::ForeignClockSource => (
            Authorization,
            Reauthorize,
            "bridge.managed_time.foreign_clock_source",
        ),
        BridgeManagedTemporalDenialKind::ForeignClockTimeline => (
            Authorization,
            Reauthorize,
            "bridge.managed_time.foreign_clock_timeline",
        ),
        BridgeManagedTemporalDenialKind::IntentCapacityExhausted => (
            Exhausted,
            Escalate,
            "bridge.managed_time.intent_capacity_exhausted",
        ),
        BridgeManagedTemporalDenialKind::RetentionCapacityExhausted => (
            Exhausted,
            Escalate,
            "bridge.managed_time.retention_capacity_exhausted",
        ),
        BridgeManagedTemporalDenialKind::RetentionQuarantined => (
            Indeterminate,
            Escalate,
            "bridge.managed_time.retention_quarantined",
        ),
        BridgeManagedTemporalDenialKind::LaneQuarantined => (
            Indeterminate,
            Escalate,
            "bridge.managed_time.lane_quarantined",
        ),
        BridgeManagedTemporalDenialKind::IntentRevisionConflict => (
            Conflict,
            Final,
            "bridge.managed_time.intent_revision_conflict",
        ),
        BridgeManagedTemporalDenialKind::SignalTemporalFailure => (
            Invariant,
            Escalate,
            "bridge.managed_time.signal_temporal_failure",
        ),
    }
}
