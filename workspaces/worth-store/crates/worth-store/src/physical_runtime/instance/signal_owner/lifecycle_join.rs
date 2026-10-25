use super::PhysicalWorkSignalOwner;
use crate::physical_runtime::{
    PhysicalSignalAspectBindingDigest, PhysicalSignalSettlementOutcome, PhysicalWorkConsumerHandle,
    PhysicalWorkEffectFate, PhysicalWorkRetrySchedule, PhysicalWorkRetryScheduleOutcome,
    SettledPhysicalWork,
};
use std::collections::HashMap;

// Store supplies completion identity through the envelope fields and retains
// physical byte counts in settlement evidence. No payload bytes are attached
// to the derived Signal completion.
const SIGNAL_COMPLETION_PAYLOAD_BYTES: u64 = 0;

impl PhysicalWorkSignalOwner {
    pub(in crate::physical_runtime) fn record_settlement(
        &self,
        settled: &SettledPhysicalWork,
    ) -> PhysicalSignalSettlementOutcome {
        if !completion_fate(settled.evidence().fate()) {
            if !settled.retry_is_physically_safe()
                && let Some(route) = self.route(settled.signal_binding())
            {
                route.release(settled.intent().identity());
            }
            return PhysicalSignalSettlementOutcome::ReconciledFromPhysicalTruth;
        }
        if !self.retain_settlement_obligation(settled) {
            self.admission_status.revoke();
            return PhysicalSignalSettlementOutcome::DerivedStateUnavailable;
        }
        let route_digest = settled.signal_binding();
        let envelope = settlement_envelope(settled);
        if self.require_available().is_err() {
            return PhysicalSignalSettlementOutcome::DerivedStateUnavailable;
        }
        let outcome = self.route(route_digest).map_or(
            PhysicalSignalSettlementOutcome::DerivedStateUnavailable,
            |route| route.record_settlement(envelope.clone()),
        );
        if outcome != PhysicalSignalSettlementOutcome::DerivedStateUnavailable {
            self.reconciliation.resolve(settled.intent().identity());
        }
        outcome
    }

    pub(in crate::physical_runtime) fn retain_settlement_obligation(
        &self,
        settled: &SettledPhysicalWork,
    ) -> bool {
        if !completion_fate(settled.evidence().fate()) {
            return true;
        }
        self.reconciliation.retain(
            settled.intent().identity(),
            settled.signal_binding(),
            settlement_envelope(settled),
        )
    }

    pub(in crate::physical_runtime) fn settlement_requires_derived_completion(
        &self,
        settled: &SettledPhysicalWork,
    ) -> bool {
        completion_fate(settled.evidence().fate())
    }

    pub(in crate::physical_runtime) fn reconcile_settlements(
        &self,
    ) -> Vec<(
        crate::physical_runtime::PhysicalWorkIdentity,
        PhysicalSignalSettlementOutcome,
    )> {
        self.reconciliation.reconcile(self)
    }

    pub(in crate::physical_runtime) fn record_settlement_batch(
        &self,
        settled: &[SettledPhysicalWork],
    ) -> Box<[PhysicalSignalSettlementOutcome]> {
        let mut outcomes =
            vec![PhysicalSignalSettlementOutcome::ReconciledFromPhysicalTruth; settled.len()];
        if self.require_available().is_err() {
            for (index, work) in settled.iter().enumerate() {
                if !completion_fate(work.evidence().fate()) {
                    continue;
                }
                outcomes[index] = PhysicalSignalSettlementOutcome::DerivedStateUnavailable;
                if !self.reconciliation.retain(
                    work.intent().identity(),
                    work.signal_binding(),
                    settlement_envelope(work),
                ) {
                    self.admission_status.revoke();
                }
            }
            return outcomes.into_boxed_slice();
        }
        let mut route_order = Vec::new();
        let mut route_groups = HashMap::<_, Vec<_>>::new();
        for (index, work) in settled.iter().enumerate() {
            if !completion_fate(work.evidence().fate()) {
                if !work.retry_is_physically_safe()
                    && let Some(route) = self.route(work.signal_binding())
                {
                    route.release(work.intent().identity());
                }
                continue;
            }
            outcomes[index] = PhysicalSignalSettlementOutcome::DerivedStateUnavailable;
            let route = work.signal_binding();
            if !route_groups.contains_key(&route) {
                route_order.push(route);
            }
            route_groups
                .entry(route)
                .or_default()
                .push((index, settlement_envelope(work)));
        }
        for route_digest in route_order {
            let Some(group) = route_groups.remove(&route_digest) else {
                continue;
            };
            let Some(route) = self.route(route_digest) else {
                continue;
            };
            let (indexes, envelopes): (Vec<_>, Vec<_>) = group.into_iter().unzip();
            let retained = envelopes.clone();
            for ((index, outcome), envelope) in indexes
                .into_iter()
                .zip(route.record_settlement_batch(envelopes))
                .zip(retained)
            {
                outcomes[index] = outcome;
                if outcome == PhysicalSignalSettlementOutcome::DerivedStateUnavailable {
                    if !self.reconciliation.retain(
                        settled[index].intent().identity(),
                        route_digest,
                        envelope,
                    ) {
                        self.admission_status.revoke();
                    }
                } else {
                    self.reconciliation
                        .resolve(settled[index].intent().identity());
                }
            }
        }
        outcomes.into_boxed_slice()
    }

    pub(in crate::physical_runtime) fn cancel(
        &self,
        consumer: PhysicalWorkConsumerHandle,
    ) -> Result<worth_signal::facade::ResourceCancellationReport, ()> {
        self.require_available().map_err(|_| ())?;
        self.route(consumer.route())
            .ok_or(())?
            .cancel(consumer.signal_request())
    }

    pub(in crate::physical_runtime) fn schedule_retry(
        &self,
        settled: &SettledPhysicalWork,
    ) -> Result<PhysicalWorkRetryScheduleOutcome, ()> {
        self.require_available().map_err(|_| ())?;
        let report = self
            .route(settled.signal_binding())
            .ok_or(())?
            .schedule_retry(settled.signal_request())?;
        let Some(scheduled) = report.scheduled_retry().cloned() else {
            return Ok(PhysicalWorkRetryScheduleOutcome::Denied(report));
        };
        Ok(PhysicalWorkRetryScheduleOutcome::Scheduled(
            PhysicalWorkRetrySchedule::new(
                settled.intent().identity(),
                settled.signal_binding(),
                scheduled,
            ),
        ))
    }

    pub(in crate::physical_runtime) fn admit_retry(
        &self,
        retry: &PhysicalWorkRetrySchedule,
    ) -> Result<worth_signal::facade::ResourceRetryAdmissionReport, ()> {
        self.require_available().map_err(|_| ())?;
        self.route(retry.route()).ok_or(())?.admit_retry(
            retry.scheduled().previous(),
            retry.scheduled().backoff_wake_id(),
        )
    }

    pub(in crate::physical_runtime) fn advance_clock(
        &self,
        route: PhysicalSignalAspectBindingDigest,
        request: worth_signal::facade::ClockAdvanceRequest,
    ) -> Result<worth_signal::facade::ValidatedClockAdvance, ()> {
        self.require_available().map_err(|_| ())?;
        self.route(route).ok_or(())?.advance_clock(request)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn advance_clock_for_certification(
        &self,
        request: worth_signal::facade::ClockAdvanceRequest,
    ) -> Result<worth_signal::facade::ValidatedClockAdvance, ()> {
        let route = self.bindings.bindings().first().ok_or(())?.digest();
        self.advance_clock(route, request)
    }

    pub(in crate::physical_runtime) fn timeout(
        &self,
        consumer: PhysicalWorkConsumerHandle,
    ) -> Result<worth_signal::facade::ResourceTimeoutReport, ()> {
        self.require_available().map_err(|_| ())?;
        self.route(consumer.route())
            .ok_or(())?
            .timeout(consumer.signal_request())
    }

    pub(in crate::physical_runtime) fn runtime_summary(
        &self,
    ) -> Result<worth_signal::facade::ResourceRuntimeSummary, ()> {
        self.require_available().map_err(|_| ())?;
        let route = self
            .bindings
            .bindings()
            .first()
            .and_then(|binding| self.route(binding.digest()))
            .ok_or(())?;
        route
            .observation()
            .map(|observation| observation.resource())
    }
}

fn completion_fate(fate: PhysicalWorkEffectFate) -> bool {
    matches!(
        fate,
        PhysicalWorkEffectFate::ReadCompleted
            | PhysicalWorkEffectFate::WriteCompleted
            | PhysicalWorkEffectFate::PublicationCompleted
    )
}

fn settlement_envelope(
    settled: &SettledPhysicalWork,
) -> worth_signal::facade::RawCompletionEnvelope {
    let signal = settled.signal_evidence();
    let handle = signal.signal_request;
    worth_signal::facade::RawCompletionEnvelope::new(
        handle.request_id(),
        handle.generation(),
        handle.branch_epoch(),
        signal.attempt,
        signal.payload_contract().clone(),
        SIGNAL_COMPLETION_PAYLOAD_BYTES,
    )
}
