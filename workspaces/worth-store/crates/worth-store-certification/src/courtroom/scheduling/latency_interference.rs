use worth_foundational::{
    performance, performance_api, FoundationalAuthoritativePerformanceClaim,
    FoundationalCounterBackedPerformanceReceipt,
    FoundationalCounterBackedPerformanceReceiptConstructionDenial,
    FoundationalPerformanceAccessPatternPosture, FoundationalPerformanceBoundary,
    FoundationalPerformanceBreadthLocalityPosture, FoundationalPerformanceContractName,
    FoundationalPerformanceCounterName, FoundationalPerformanceCounterRow,
    FoundationalPerformanceCounterSpec, FoundationalPerformanceEvidenceStrength,
    FoundationalPerformanceExecutionTemperature, FoundationalPerformanceFallbackDebtPosture,
    FoundationalPerformanceFreshnessRetentionPosture, FoundationalPerformanceWorkClass,
};
use worth_store_budgets::CounterEvidenceStrength;
use worth_store_io_scheduler::{
    InterferenceCounterDenial, InterferenceCounterName, InterferenceCounterRow,
    LatencyEnvelopeAssessment, LatencyEnvelopeAssessmentStatus,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct S6LatencyInterferenceEvidence {
    status: LatencyEnvelopeAssessmentStatus,
    rows: Vec<InterferenceCounterRow>,
    counter_backed_receipt:
        FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum S6LatencyInterferenceCertificationDenial {
    MissingRequiredCounter(InterferenceCounterName),
    InsufficientCounterStrength {
        counter: InterferenceCounterName,
        required: CounterEvidenceStrength,
        actual: CounterEvidenceStrength,
    },
    MissingCausalAttribution(InterferenceCounterName),
    WallClockReplayClaim,
    SchedulerEvidence(InterferenceCounterDenial),
    CounterBackedReceipt(FoundationalCounterBackedPerformanceReceiptConstructionDenial),
    FoundationalPerformanceClaim,
    FoundationalPerformanceAttachment,
}

impl S6LatencyInterferenceEvidence {
    pub fn from_assessment(
        assessment: &LatencyEnvelopeAssessment,
    ) -> Result<Self, S6LatencyInterferenceCertificationDenial> {
        if !assessment.replay_scope().excludes_wall_clock_timing() {
            return Err(S6LatencyInterferenceCertificationDenial::WallClockReplayClaim);
        }
        let rows = assessment.counter_rows().to_vec();
        require_declared_strengths(&rows)?;
        let counter_backed_receipt = build_counter_backed_receipt(&rows)?;
        Ok(Self {
            status: assessment.status(),
            rows,
            counter_backed_receipt,
        })
    }

    pub const fn status(&self) -> LatencyEnvelopeAssessmentStatus {
        self.status
    }

    pub fn rows(&self) -> &[InterferenceCounterRow] {
        &self.rows
    }

    pub const fn counter_backed_receipt(
        &self,
    ) -> &FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim>
    {
        &self.counter_backed_receipt
    }

    pub fn require_counter_strength(
        &self,
        counter: InterferenceCounterName,
        required: CounterEvidenceStrength,
    ) -> Result<(), S6LatencyInterferenceCertificationDenial> {
        let row = self
            .rows
            .iter()
            .find(|row| row.name() == counter)
            .ok_or(S6LatencyInterferenceCertificationDenial::MissingRequiredCounter(counter))?;
        if !row.strength().satisfies(required) {
            return Err(
                S6LatencyInterferenceCertificationDenial::InsufficientCounterStrength {
                    counter,
                    required,
                    actual: row.strength(),
                },
            );
        }
        Ok(())
    }
}

impl From<InterferenceCounterDenial> for S6LatencyInterferenceCertificationDenial {
    fn from(denial: InterferenceCounterDenial) -> Self {
        Self::SchedulerEvidence(denial)
    }
}

fn require_declared_strengths(
    rows: &[InterferenceCounterRow],
) -> Result<(), S6LatencyInterferenceCertificationDenial> {
    for row in rows {
        if !row.strength().is_declared() && (row.value() != 0 || row.attribution().is_some()) {
            return Err(
                S6LatencyInterferenceCertificationDenial::InsufficientCounterStrength {
                    counter: row.name(),
                    required: CounterEvidenceStrength::Exact,
                    actual: row.strength(),
                },
            );
        }
        if row.value() > 0
            && matches!(
                row.name(),
                InterferenceCounterName::QueueBackpressureEvents
                    | InterferenceCounterName::QueueForegroundWaitEvents
                    | InterferenceCounterName::QueueViolationEvents
                    | InterferenceCounterName::FlushDelayEvents
                    | InterferenceCounterName::SyncDebtUnits
                    | InterferenceCounterName::BackendContradictionEvents
                    | InterferenceCounterName::EnvelopeExceededEvents
                    | InterferenceCounterName::PolicyDebtEvents
                    | InterferenceCounterName::BackgroundYieldEvents
                    | InterferenceCounterName::BackgroundDebtUnits
                    | InterferenceCounterName::BackgroundViolationEvents
            )
            && row.attribution().is_none()
        {
            return Err(
                S6LatencyInterferenceCertificationDenial::MissingCausalAttribution(row.name()),
            );
        }
    }
    Ok(())
}

fn build_counter_backed_receipt(
    rows: &[InterferenceCounterRow],
) -> Result<
    FoundationalCounterBackedPerformanceReceipt<FoundationalAuthoritativePerformanceClaim>,
    S6LatencyInterferenceCertificationDenial,
> {
    let claim = performance()
        .claim()
        .authoritative_execution()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::TraversalLocal)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
        .finish()
        .map_err(|_| S6LatencyInterferenceCertificationDenial::FoundationalPerformanceClaim)?;
    let mut bundle = performance_api::lower_lane::basis::performance_bundle(claim)
        .attach_contract_name(
            FoundationalPerformanceContractName::new("worth-store.s6.latency-interference")
                .map_err(|_| {
                    S6LatencyInterferenceCertificationDenial::FoundationalPerformanceAttachment
                })?,
        );
    for row in rows
        .iter()
        .filter(|row| row.strength() == CounterEvidenceStrength::Exact)
    {
        bundle = bundle.attach_counter_spec(FoundationalPerformanceCounterSpec::new(
            foundational_counter_name(row.name())?,
            FoundationalPerformanceWorkClass::ValidationPlanning,
            row.value(),
        ));
    }
    let bundle = bundle
        .finish()
        .map_err(|_| S6LatencyInterferenceCertificationDenial::FoundationalPerformanceAttachment)?;
    let mut receipt =
        performance_api::lower_lane::receipts::counter_backed_performance_receipt(bundle);
    for row in rows
        .iter()
        .filter(|row| row.strength() == CounterEvidenceStrength::Exact)
    {
        receipt = receipt.attach_counter_row(FoundationalPerformanceCounterRow::new(
            foundational_counter_name(row.name())?,
            row.value(),
        ));
    }
    receipt
        .finish()
        .map_err(S6LatencyInterferenceCertificationDenial::CounterBackedReceipt)
}

fn foundational_counter_name(
    name: InterferenceCounterName,
) -> Result<FoundationalPerformanceCounterName, S6LatencyInterferenceCertificationDenial> {
    FoundationalPerformanceCounterName::new(format!("store.s6.{}", name.as_str()))
        .map_err(|_| S6LatencyInterferenceCertificationDenial::FoundationalPerformanceAttachment)
}

#[cfg(test)]
mod tests {
    use worth_store_io_scheduler::{
        foreground_reservation::ForegroundIoLaneKind, InterferenceAttribution,
        InterferenceCounterName, InterferenceCounterRow, QueueWorkClass,
    };

    use super::*;

    #[test]
    fn certification_rejects_insufficient_counter_strength() {
        let evidence = evidence_from_rows(vec![InterferenceCounterRow::new(
            InterferenceCounterName::QueuePeakDepth,
            3,
            CounterEvidenceStrength::Sampled,
            "s6-profile/posix-file",
            test_lane(),
            Some(InterferenceAttribution::Queueing),
        )]);

        let denial = evidence
            .require_counter_strength(
                InterferenceCounterName::QueuePeakDepth,
                CounterEvidenceStrength::Exact,
            )
            .expect_err("sampled queue depth must not certify as exact");

        assert_eq!(
            denial,
            S6LatencyInterferenceCertificationDenial::InsufficientCounterStrength {
                counter: InterferenceCounterName::QueuePeakDepth,
                required: CounterEvidenceStrength::Exact,
                actual: CounterEvidenceStrength::Sampled,
            }
        );
    }

    #[test]
    fn certification_publishes_counter_backed_performance_receipt() {
        let evidence = evidence_from_rows(vec![InterferenceCounterRow::new(
            InterferenceCounterName::QueueForegroundWaitEvents,
            1,
            CounterEvidenceStrength::Exact,
            "s6-profile/posix-file",
            test_lane(),
            Some(InterferenceAttribution::ForegroundWait),
        )]);

        assert_eq!(evidence.counter_backed_receipt().counter_rows().len(), 1);
        assert_eq!(
            evidence.counter_backed_receipt().counter_rows()[0].observed_count(),
            1
        );
    }

    #[test]
    fn sampled_rows_stay_out_of_exact_counter_backed_receipt() {
        let evidence = evidence_from_rows(vec![InterferenceCounterRow::new(
            InterferenceCounterName::QueuePeakDepth,
            3,
            CounterEvidenceStrength::Sampled,
            "s6-profile/posix-file",
            test_lane(),
            Some(InterferenceAttribution::Queueing),
        )]);

        assert_eq!(
            evidence.rows()[0].strength(),
            CounterEvidenceStrength::Sampled
        );
        assert!(evidence.counter_backed_receipt().counter_rows().is_empty());
    }

    #[test]
    fn unavailable_zero_rows_remain_explicit_non_claims() {
        let evidence = evidence_from_rows(vec![InterferenceCounterRow::new(
            InterferenceCounterName::PageCacheWaitEvents,
            0,
            CounterEvidenceStrength::Unavailable,
            "s6-profile/posix-file",
            test_lane(),
            None,
        )]);

        assert_eq!(
            evidence.rows()[0].strength(),
            CounterEvidenceStrength::Unavailable
        );
        assert!(evidence.counter_backed_receipt().counter_rows().is_empty());
        assert_eq!(
            evidence
                .require_counter_strength(
                    InterferenceCounterName::PageCacheWaitEvents,
                    CounterEvidenceStrength::Exact,
                )
                .unwrap_err(),
            S6LatencyInterferenceCertificationDenial::InsufficientCounterStrength {
                counter: InterferenceCounterName::PageCacheWaitEvents,
                required: CounterEvidenceStrength::Exact,
                actual: CounterEvidenceStrength::Unavailable,
            }
        );
    }

    #[test]
    fn scheduler_denial_maps_without_strength_upgrade() {
        let denial = InterferenceCounterDenial::InsufficientCounterStrength {
            counter: InterferenceCounterName::QueuePeakDepth,
            required: CounterEvidenceStrength::Exact,
            actual: CounterEvidenceStrength::Sampled,
        };

        assert_eq!(
            S6LatencyInterferenceCertificationDenial::from(denial),
            S6LatencyInterferenceCertificationDenial::SchedulerEvidence(denial)
        );
    }

    #[test]
    fn certification_rejects_unattributed_positive_violation_rows() {
        let rows = vec![InterferenceCounterRow::new(
            InterferenceCounterName::QueueViolationEvents,
            1,
            CounterEvidenceStrength::Exact,
            "s6-profile/posix-file",
            test_lane(),
            None,
        )];

        let denial = require_declared_strengths(&rows)
            .expect_err("positive violation rows need causal attribution");

        assert_eq!(
            denial,
            S6LatencyInterferenceCertificationDenial::MissingCausalAttribution(
                InterferenceCounterName::QueueViolationEvents
            )
        );
    }

    const fn test_lane() -> QueueWorkClass {
        QueueWorkClass::Foreground(ForegroundIoLaneKind::PointRead)
    }

    fn evidence_from_rows(rows: Vec<InterferenceCounterRow>) -> S6LatencyInterferenceEvidence {
        S6LatencyInterferenceEvidence {
            status: LatencyEnvelopeAssessmentStatus::Held,
            counter_backed_receipt: build_counter_backed_receipt(&rows)
                .expect("test rows should build counter-backed receipt"),
            rows,
        }
    }
}
