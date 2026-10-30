use std::sync::Arc;

use crate::runtime::intent::{
    UiAdmittedIntent, UiIntentAdmissionLease, UiIntentAdmissionSettlementPosture,
    UiIntentAdmissionSettlementReceipt, UiIntentAdmissionSlotIdentity,
    UiIntentOccupancyReservationDenial,
};

mod census;
#[cfg(any(test, feature = "certification-support"))]
mod certification;
mod consequence;
mod lifecycle;
mod progression;
mod recovery;
mod reservation;
mod reservation_authority;
mod shutdown;

pub(crate) use consequence::{
    UiIntentConsequenceBeginOutcome, UiIntentConsequenceCurrentnessContext,
};

pub(crate) use census::UiIntentExecutionAdmissionCensus;

use super::{
    UiIntentExecutionCapacity, UiIntentExecutionDeadline, UiIntentExecutionDispatchOutcome,
    UiIntentExecutionDispatchReceipt, UiIntentExecutionDispatchStop,
    UiIntentExecutionDispatchStopReason, UiIntentExecutionReservationDenial,
    UI_INTENT_MAXIMUM_APPLICATION_ATTEMPTS,
};
use reservation_authority::{
    UiActiveIntentExecutionReservation, UiIntentConsequenceBasis, UiIntentExecutionReservationCore,
    UiReservedIntentExecutionReservation,
};

pub(crate) struct UiIntentExecutionState {
    slots: [UiIntentExecutionSlot; UI_INTENT_MAXIMUM_APPLICATION_ATTEMPTS],
    occupancy: crate::runtime::intent::UiIntentOccupancyState,
    capacity: UiIntentExecutionCapacity,
    last_tick: Option<u64>,
}

struct UiIntentExecutionSlot {
    generation: u64,
    phase: Option<UiIntentExecutionSlotPhase>,
}

enum UiIntentExecutionSlotPhase {
    Admitted(UiReservedIntentAdmission),
    AttemptPrepared(UiPreparedFrameworkIntentAttempt),
    Running(UiRunningFrameworkIntentAttempt),
    Recovery(UiRecoveringFrameworkIntentAttempt),
    ConsequencePending(UiSettledFrameworkIntentAttempt),
    ConsequenceReady(UiPreparedFrameworkIntentConsequence),
    ConsequenceHandoff(UiFrameworkIntentConsequenceHandoffMarker),
}

struct UiReservedIntentAdmission {
    candidate: crate::runtime::intent::UiCurrentIntentAdmissionCandidate,
    reservation: UiReservedIntentExecutionReservation,
}

struct UiPreparedFrameworkIntentAttempt {
    reservation: UiActiveIntentExecutionReservation,
    execution: super::UiPreparedIntentExecution,
    attempt: super::UiIntentExecutionAttemptIdentity,
    idempotency: super::UiIntentExecutionIdempotencyIdentity,
    deadline: UiIntentExecutionDeadline,
}

struct UiRunningFrameworkIntentAttempt {
    reservation: UiActiveIntentExecutionReservation,
    execution: Box<dyn super::provider::UiManagedIntentExecution>,
    attempt: super::UiIntentExecutionAttemptIdentity,
    idempotency: super::UiIntentExecutionIdempotencyIdentity,
    deadline: UiIntentExecutionDeadline,
    cancellation: Option<super::UiIntentExecutionCancellationReason>,
}

struct UiRecoveringFrameworkIntentAttempt {
    reservation: UiActiveIntentExecutionReservation,
    recovery: Box<dyn super::provider::UiManagedIntentRecovery>,
    attempt: super::UiIntentExecutionAttemptIdentity,
    idempotency: super::UiIntentExecutionIdempotencyIdentity,
    lease: Arc<super::UiIntentRecoveryLease>,
    partial: Option<super::provider::UiManagedIntentPartialEffect>,
}

struct UiSettledFrameworkIntentAttempt {
    attempt: super::UiIntentExecutionAttemptIdentity,
    idempotency: super::UiIntentExecutionIdempotencyIdentity,
    outcome: Box<dyn super::provider::UiManagedIntentOutcomeMaterial>,
    consequence_lease: Arc<super::UiIntentConsequenceLease>,
    basis: UiIntentConsequenceBasis,
}

struct UiPreparedFrameworkIntentConsequence {
    attempt: super::UiIntentExecutionAttemptIdentity,
    idempotency: super::UiIntentExecutionIdempotencyIdentity,
    consequence_lease: Arc<super::UiIntentConsequenceLease>,
    basis: UiIntentConsequenceBasis,
    batch: UiPreparedIntentConsequenceBatch,
}

struct UiFrameworkIntentConsequenceHandoffMarker {
    attempt: super::UiIntentExecutionAttemptIdentity,
    idempotency: super::UiIntentExecutionIdempotencyIdentity,
    consequence_lease: Arc<super::UiIntentConsequenceLease>,
}

struct UiPreparedIntentConsequenceBatch {
    mounted_posture: bool,
    runtime_service: Option<crate::capability::UiIntentRuntimeServiceDestination>,
    query_collection_change: Option<worth_ui_query_binding::WorthUiCollectionChangeConsequence>,
    query_projection: Option<worth_ui_query_binding::UiProjectionObservation>,
}

pub(crate) struct UiIntentConsequenceHandoff {
    slot: usize,
    attempt: super::UiIntentExecutionAttemptIdentity,
    idempotency: super::UiIntentExecutionIdempotencyIdentity,
    consequence_lease: Arc<super::UiIntentConsequenceLease>,
    basis: UiIntentConsequenceBasis,
    batch: UiPreparedIntentConsequenceBatch,
}

pub(crate) struct UiIntentExecutionAdmissionCommit {
    identity: crate::runtime::intent::UiAdmittedIntentIdentity,
    lease: Arc<UiIntentAdmissionLease>,
    slots_inspected: usize,
    occupancy_slots_inspected: usize,
}

pub(crate) struct UiIntentExecutionAdmissionReservationFailure {
    reason: UiIntentExecutionAdmissionReservationFailureReason,
    slots_inspected: usize,
    occupancy_slots_inspected: usize,
}

pub(crate) enum UiIntentExecutionAdmissionReservationFailureReason {
    Capacity(UiIntentExecutionReservationDenial),
    Occupancy(UiIntentOccupancyReservationDenial),
    ReservationIdentityExhausted,
}

impl UiIntentExecutionState {
    pub(crate) fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| UiIntentExecutionSlot {
                generation: 0,
                phase: None,
            }),
            occupancy: crate::runtime::intent::UiIntentOccupancyState::new(),
            capacity: UiIntentExecutionCapacity::production(),
            last_tick: None,
        }
    }

    pub(crate) const fn occupancy(&self) -> &crate::runtime::intent::UiIntentOccupancyState {
        &self.occupancy
    }

    fn admit_monotonic_tick(&mut self, tick: u64) -> Result<(), (u64, u64)> {
        if let Some(previous) = self.last_tick
            && tick < previous
        {
            return Err((previous, tick));
        }
        self.last_tick = Some(tick);
        Ok(())
    }

    fn release_reservation(&mut self, reservation: UiActiveIntentExecutionReservation) {
        let _ = self.occupancy.release(reservation.core.occupancy);
    }

    pub(crate) fn release_admission<I: crate::capability::UiIntent>(
        &mut self,
        admitted: UiAdmittedIntent<I>,
    ) -> UiIntentAdmissionSettlementReceipt {
        let (identity, lease) = admitted.into_parts();
        let exact = self.exact_admitted(identity, &lease);
        let posture = if exact {
            let reserved = self.take_reserved(identity.slot());
            let _ = self.occupancy.release(reserved.reservation.core.occupancy);
            lease.mark_released();
            UiIntentAdmissionSettlementPosture::Released
        } else {
            lease.settlement_posture()
        };
        UiIntentAdmissionSettlementReceipt::new(posture, self.active_count())
    }

    pub(crate) fn dispatch<I: crate::capability::UiIntent>(
        &mut self,
        admitted: UiAdmittedIntent<I>,
        context: crate::runtime::intent::UiIntentAdmissionCurrentnessContext<'_>,
        session_identity: u64,
        deadline_tick: u64,
    ) -> UiIntentExecutionDispatchOutcome {
        let (identity, lease) = admitted.into_parts();
        if !self.exact_admitted(identity, &lease) {
            return self.dispatch_stopped(UiIntentExecutionDispatchStopReason::AdmissionSettled(
                lease.settlement_posture(),
            ));
        }
        let currentness = {
            let reserved = self.reserved(identity.slot());
            crate::runtime::intent::revalidate_typed_candidate_for_execution::<I>(
                &reserved.candidate,
                context,
            )
        };
        let currentness = match currentness {
            Ok(currentness) => currentness,
            Err(stop) => {
                let reserved = self.take_reserved(identity.slot());
                let _ = self.occupancy.release(reserved.reservation.core.occupancy);
                lease.mark_released();
                return self
                    .dispatch_stopped(UiIntentExecutionDispatchStopReason::Currentness(stop));
            }
        };
        let reserved = self.take_reserved(identity.slot());
        let execution = reserved.candidate.into_execution();
        let (currentness_checks, target_affinity) = currentness.into_parts();
        let reservation = reserved.reservation.activate(target_affinity);
        let attempt =
            super::UiIntentExecutionAttemptIdentity::issued(identity.slot(), identity.generation());
        let idempotency = super::UiIntentExecutionIdempotencyIdentity::issued(
            session_identity,
            reservation.core.lineage.diagnostic_value(),
        );
        let deadline = UiIntentExecutionDeadline::at_tick(deadline_tick);
        reservation.core.lease.mark_transferred_to_execution();
        self.slots[identity.slot() as usize].phase = Some(
            UiIntentExecutionSlotPhase::AttemptPrepared(UiPreparedFrameworkIntentAttempt {
                reservation,
                execution,
                attempt,
                idempotency,
                deadline,
            }),
        );
        UiIntentExecutionDispatchOutcome::AttemptPrepared(UiIntentExecutionDispatchReceipt::new(
            attempt,
            idempotency,
            deadline,
            currentness_checks,
        ))
    }

    fn exact_admitted(
        &self,
        identity: UiIntentAdmissionSlotIdentity,
        lease: &Arc<UiIntentAdmissionLease>,
    ) -> bool {
        let slot = &self.slots[identity.slot() as usize];
        slot.generation == identity.generation()
            && matches!(
                slot.phase.as_ref(),
                Some(UiIntentExecutionSlotPhase::Admitted(reserved))
                    if Arc::ptr_eq(&reserved.reservation.core.lease, lease)
            )
    }

    fn reserved(&self, slot: u8) -> &UiReservedIntentAdmission {
        match self.slots[slot as usize].phase.as_ref() {
            Some(UiIntentExecutionSlotPhase::Admitted(reserved)) => reserved,
            _ => unreachable!("exact admitted identity points at admitted execution phase"),
        }
    }

    fn take_reserved(&mut self, slot: u8) -> UiReservedIntentAdmission {
        match self.slots[slot as usize].phase.take() {
            Some(UiIntentExecutionSlotPhase::Admitted(reserved)) => reserved,
            _ => unreachable!("exact admitted identity consumes admitted execution phase"),
        }
    }

    fn dispatch_stopped(
        &self,
        reason: UiIntentExecutionDispatchStopReason,
    ) -> UiIntentExecutionDispatchOutcome {
        UiIntentExecutionDispatchOutcome::Stopped(UiIntentExecutionDispatchStop::new(
            reason,
            self.active_count(),
        ))
    }

    fn active_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| {
                slot.phase
                    .as_ref()
                    .is_some_and(|phase| phase.reservation().is_some())
            })
            .count()
    }
}

impl UiIntentExecutionSlotPhase {
    fn reservation(&self) -> Option<&UiIntentExecutionReservationCore> {
        match self {
            Self::Admitted(reserved) => Some(&reserved.reservation.core),
            Self::AttemptPrepared(attempt) => Some(&attempt.reservation.core),
            Self::Running(attempt) => Some(&attempt.reservation.core),
            Self::Recovery(attempt) => Some(&attempt.reservation.core),
            Self::ConsequencePending(_)
            | Self::ConsequenceReady(_)
            | Self::ConsequenceHandoff(_) => None,
        }
    }
}

impl UiIntentExecutionAdmissionCommit {
    pub(crate) fn into_parts(
        self,
    ) -> (
        crate::runtime::intent::UiAdmittedIntentIdentity,
        Arc<UiIntentAdmissionLease>,
        usize,
        usize,
    ) {
        (
            self.identity,
            self.lease,
            self.slots_inspected,
            self.occupancy_slots_inspected,
        )
    }
}

impl UiIntentExecutionAdmissionReservationFailure {
    pub(crate) fn into_parts(
        self,
    ) -> (
        UiIntentExecutionAdmissionReservationFailureReason,
        usize,
        usize,
    ) {
        (
            self.reason,
            self.slots_inspected,
            self.occupancy_slots_inspected,
        )
    }
}
