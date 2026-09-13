use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

use worth_signal::facade::{
    ClockTick, TemporalCondition, TemporalWakeId, TemporalWakeRetirementReason,
};

use super::contract::{
    BridgeManagedClockLease, BridgeManagedDueWake, BridgeManagedTemporalDenial,
    BridgeManagedTemporalDenialKind, BridgeManagedTemporalIntentIdentity,
    BridgeManagedTemporalIntentLifecycle, BridgeManagedTemporalIntentReconciliation,
};

pub(super) struct BridgeManagedTemporalIntentRecord {
    pub(super) revision: u64,
    pub(super) due_coordinate: u64,
    pub(super) idempotency_identity: Arc<str>,
    pub(super) source_record_identity:
        crate::relational_identity::RelationalBridgeRecordIdentityParts,
    pub(super) wake_id: TemporalWakeId,
    pub(super) observation_baselines:
        Arc<super::super::observation_retention::BridgeObservationBaselines>,
}

pub(in crate::conditional_execution) struct BridgeManagedClockLane {
    pub(in crate::conditional_execution) lifecycle_token: Arc<()>,
    lowering: Arc<super::super::BridgeInstalledConditionalLowering>,
    pub(super) source_identity: Arc<str>,
    pub(super) timeline_identity: Arc<str>,
    pub(super) lease: Arc<BridgeManagedClockLease>,
    pub(super) maximum_active_intents: usize,
    pub(super) maximum_due_wakes_per_observation: NonZeroUsize,
    last_observation: Option<(u64, u64)>,
    pub(super) quarantined: bool,
    #[cfg(test)]
    pub(super) fault: Option<super::quarantine::Fault>,
    pub(super) retention: Arc<super::super::retention::BridgeRetentionLedger>,
    pub(super) signal: worth_signal::facade::branch::SignalConditionalTemporalPartition<(), (), ()>,
    pub(super) intents:
        BTreeMap<BridgeManagedTemporalIntentIdentity, BridgeManagedTemporalIntentRecord>,
    pub(super) wake_to_intent: BTreeMap<TemporalWakeId, BridgeManagedTemporalIntentIdentity>,
    _reservation: super::super::retention::BridgeRetentionReservation,
}

impl BridgeManagedClockLane {
    pub(super) fn new(
        lowering: Arc<super::super::BridgeInstalledConditionalLowering>,
        source_identity: Arc<str>,
        timeline_identity: Arc<str>,
        lease: Arc<BridgeManagedClockLease>,
        maximum_active_intents: usize,
        maximum_due_wakes_per_observation: NonZeroUsize,
        signal: worth_signal::facade::branch::SignalConditionalTemporalPartition<(), (), ()>,
        retention: Arc<super::super::retention::BridgeRetentionLedger>,
        reservation: super::super::retention::BridgeRetentionReservation,
    ) -> Self {
        Self {
            lifecycle_token: Default::default(),
            retention,
            _reservation: reservation,
            lowering,
            source_identity,
            timeline_identity,
            lease,
            maximum_active_intents,
            maximum_due_wakes_per_observation,
            last_observation: None,
            quarantined: false,
            #[cfg(test)]
            fault: None,
            signal,
            intents: BTreeMap::new(),
            wake_to_intent: BTreeMap::new(),
        }
    }

    pub(in crate::conditional_execution) fn retains_due_wake(
        &self,
        lowering: &Arc<super::super::BridgeInstalledConditionalLowering>,
        wake: &BridgeManagedDueWake,
    ) -> bool {
        if !Arc::ptr_eq(&self.lowering, lowering) {
            return false;
        }
        let Some(identity) = self.wake_to_intent.get(&wake.signal_wake_id) else {
            return false;
        };
        let Some(intent) = self.intents.get(identity) else {
            return false;
        };
        identity == &wake.intent_identity
            && intent.wake_id == wake.signal_wake_id
            && intent.revision == wake.revision
            && intent.due_coordinate == wake.due_coordinate
            && intent.idempotency_identity == wake.idempotency_identity
            && intent.source_record_identity == wake.source_record_identity
    }

    pub(super) fn last_observation(&self) -> Option<(u64, u64)> {
        self.last_observation
    }

    pub(super) fn record_observation(&mut self, sequence: u64, coordinate: u64) {
        self.last_observation = Some((sequence, coordinate));
    }

    pub(super) fn reconcile_active_intent(
        &mut self,
        identity: BridgeManagedTemporalIntentIdentity,
        revision: u64,
        due_coordinate: u64,
        idempotency_identity: Arc<str>,
        source_record_identity: crate::relational_identity::RelationalBridgeRecordIdentityParts,
    ) -> Result<BridgeManagedTemporalIntentReconciliation, BridgeManagedTemporalDenial> {
        let Some(existing) = self.intents.get(&identity) else {
            return self.install_new_intent(
                identity,
                revision,
                due_coordinate,
                idempotency_identity,
                source_record_identity,
            );
        };
        if revision < existing.revision {
            return Ok(BridgeManagedTemporalIntentReconciliation::Stale);
        }
        if revision == existing.revision {
            return if existing.due_coordinate == due_coordinate
                && existing.idempotency_identity == idempotency_identity
                && existing.source_record_identity == source_record_identity
            {
                Ok(BridgeManagedTemporalIntentReconciliation::Duplicate)
            } else {
                Err(BridgeManagedTemporalDenial::new(
                    BridgeManagedTemporalDenialKind::IntentRevisionConflict,
                    "one temporal-intent revision changed its due or idempotency meaning",
                ))
            };
        }

        let observation_baselines =
            super::super::observation_retention::BridgeObservationBaselines::new(&self.retention)
                .map_err(|denial| {
                BridgeManagedTemporalDenial::new(
                    BridgeManagedTemporalDenialKind::RetentionCapacityExhausted,
                    denial.detail(),
                )
            })?;
        let old_wake = existing.wake_id;
        self.begin_effect()?;
        let replacement = self.signal.supersede_temporal_wake(
            old_wake,
            TemporalCondition::at_or_after(ClockTick::new(due_coordinate)),
            ClockTick::new(due_coordinate),
        );
        let replacement = self.admit_signal_result(replacement)?;
        self.after_signal_before_publication()?;
        self.wake_to_intent.remove(&old_wake);
        self.wake_to_intent
            .insert(replacement.scheduled().id(), identity.clone());
        self.intents.insert(
            identity,
            BridgeManagedTemporalIntentRecord {
                revision,
                due_coordinate,
                idempotency_identity,
                source_record_identity,
                wake_id: replacement.scheduled().id(),
                observation_baselines,
            },
        );
        self.finish_effect();
        Ok(BridgeManagedTemporalIntentReconciliation::Superseded)
    }

    fn install_new_intent(
        &mut self,
        identity: BridgeManagedTemporalIntentIdentity,
        revision: u64,
        due_coordinate: u64,
        idempotency_identity: Arc<str>,
        source_record_identity: crate::relational_identity::RelationalBridgeRecordIdentityParts,
    ) -> Result<BridgeManagedTemporalIntentReconciliation, BridgeManagedTemporalDenial> {
        if self.intents.len() >= self.maximum_active_intents {
            return Err(BridgeManagedTemporalDenial::new(
                BridgeManagedTemporalDenialKind::IntentCapacityExhausted,
                "managed temporal-intent capacity was exhausted before Signal admission",
            ));
        }
        let observation_baselines =
            super::super::observation_retention::BridgeObservationBaselines::new(&self.retention)
                .map_err(|denial| {
                BridgeManagedTemporalDenial::new(
                    BridgeManagedTemporalDenialKind::RetentionCapacityExhausted,
                    denial.detail(),
                )
            })?;
        self.begin_effect()?;
        let wake = self.signal.schedule_temporal_wake(
            TemporalCondition::at_or_after(ClockTick::new(due_coordinate)),
            ClockTick::new(due_coordinate),
        );
        let wake = self.admit_signal_result(wake)?;
        self.after_signal_before_publication()?;
        self.wake_to_intent.insert(wake.id(), identity.clone());
        self.intents.insert(
            identity,
            BridgeManagedTemporalIntentRecord {
                revision,
                due_coordinate,
                idempotency_identity,
                source_record_identity,
                wake_id: wake.id(),
                observation_baselines,
            },
        );
        self.finish_effect();
        Ok(BridgeManagedTemporalIntentReconciliation::Installed)
    }

    pub(super) fn reconcile_terminal_intent(
        &mut self,
        identity: &BridgeManagedTemporalIntentIdentity,
        revision: u64,
        lifecycle: BridgeManagedTemporalIntentLifecycle,
    ) -> Result<BridgeManagedTemporalIntentReconciliation, BridgeManagedTemporalDenial> {
        let Some(existing) = self.intents.get(identity) else {
            return Ok(BridgeManagedTemporalIntentReconciliation::TerminalNoop);
        };
        if revision < existing.revision {
            return Ok(BridgeManagedTemporalIntentReconciliation::Stale);
        }
        if revision == existing.revision {
            return Err(BridgeManagedTemporalDenial::new(
                BridgeManagedTemporalDenialKind::IntentRevisionConflict,
                "one temporal-intent revision cannot change lifecycle meaning",
            ));
        }
        let reason = match lifecycle {
            BridgeManagedTemporalIntentLifecycle::Cancelled => {
                TemporalWakeRetirementReason::Cancelled
            }
            BridgeManagedTemporalIntentLifecycle::Completed => {
                TemporalWakeRetirementReason::Consumed
            }
            BridgeManagedTemporalIntentLifecycle::Active => {
                return Err(BridgeManagedTemporalDenial::new(
                    BridgeManagedTemporalDenialKind::InvalidContract,
                    "active intent reached terminal reconciliation",
                ));
            }
        };
        let wake_id = existing.wake_id;
        self.begin_effect()?;
        let retirement = self.signal.retire_temporal_wake(wake_id, reason);
        self.admit_signal_result(retirement)?;
        self.after_signal_before_publication()?;
        self.intents.remove(identity);
        self.wake_to_intent.remove(&wake_id);
        self.finish_effect();
        Ok(BridgeManagedTemporalIntentReconciliation::Retired)
    }

    pub(in crate::conditional_execution) fn revoke_liveness(&self) {
        self.lease.revoke();
    }

    pub(super) fn closure_counts(
        &self,
    ) -> Result<(usize, usize, usize), BridgeManagedTemporalDenial> {
        let wake_summary = self.signal.temporal_wake_summary().map_err(signal_denial)?;
        Ok((
            self.intents.len(),
            wake_summary.scheduled_count() as usize,
            wake_summary.ready_count() as usize,
        ))
    }
}

pub(super) fn signal_denial(
    error: worth_signal::facade::branch::SignalConditionalTemporalPartitionDenial,
) -> BridgeManagedTemporalDenial {
    BridgeManagedTemporalDenial::new(
        BridgeManagedTemporalDenialKind::SignalTemporalFailure,
        format!("Signal temporal authority denied managed time: {error:?}"),
    )
}
