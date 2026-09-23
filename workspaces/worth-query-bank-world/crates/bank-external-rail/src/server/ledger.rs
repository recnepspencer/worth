//! Atomic idempotency admission for external-rail dispatches.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::protocol::correlation::RailCorrelation;
use crate::protocol::notice::{EstateDeathNotice, RailDomainEffect};
use crate::protocol::payload::RailEffectPayload;
use crate::protocol::response::LedgerStatus;

/// Result of atomically comparing and reserving one correlation.
pub enum RailAdmission {
    Reserved(RailReservation),
    Replay(LedgerStatus),
    MeaningDrift,
    DisappearedBeforeAdmission,
}

/// Sole capability to complete a newly reserved correlation.
///
/// Replays never receive this value, so they cannot repeat the physical
/// consequence even when many connections race on the same key.
pub struct RailReservation {
    correlation: RailCorrelation,
    effect: RailDomainEffect,
}

impl RailReservation {
    pub fn correlation(&self) -> &RailCorrelation {
        &self.correlation
    }

    pub fn effect(&self) -> &RailDomainEffect {
        &self.effect
    }
}

struct RailRecord {
    status: LedgerStatus,
    request_fingerprint: RailRequestFingerprint,
    effect: RailDomainEffect,
}

/// Exact immutable request meaning bound to one idempotency correlation.
///
/// The payload is retained rather than reduced through a process-local hasher,
/// so equality cannot collide or drift across compiler versions.
#[derive(Clone, Eq, PartialEq)]
struct RailRequestFingerprint(RailEffectPayload);

impl RailRequestFingerprint {
    fn from_payload(payload: &RailEffectPayload) -> Self {
        Self(payload.clone())
    }
}

/// The rail's authoritative idempotency and protocol-status ledger.
#[derive(Default)]
pub struct Ledger {
    records: Mutex<HashMap<RailCorrelation, RailRecord>>,
    admissions: AtomicU64,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Atomically compares a correlation's immutable request and reserves it.
    ///
    /// `reserve_new == false` is the disappear-before-admission fault. Existing
    /// correlations are still compared first, so that fault cannot conceal a
    /// same-key/different-payload attack.
    pub fn admit(
        &self,
        correlation: &RailCorrelation,
        payload: &RailEffectPayload,
        effect: RailDomainEffect,
        reserve_new: bool,
    ) -> RailAdmission {
        let request_fingerprint = RailRequestFingerprint::from_payload(payload);
        let mut records = self.lock();
        match records.entry(correlation.clone()) {
            Entry::Occupied(entry) => {
                let record = entry.get();
                if record.request_fingerprint != request_fingerprint || record.effect != effect {
                    RailAdmission::MeaningDrift
                } else {
                    RailAdmission::Replay(record.status)
                }
            }
            Entry::Vacant(_) if !reserve_new => RailAdmission::DisappearedBeforeAdmission,
            Entry::Vacant(entry) => {
                entry.insert(RailRecord {
                    status: LedgerStatus::Acknowledged,
                    request_fingerprint,
                    effect: effect.clone(),
                });
                self.admissions.fetch_add(1, Ordering::SeqCst);
                RailAdmission::Reserved(RailReservation {
                    correlation: correlation.clone(),
                    effect,
                })
            }
        }
    }

    pub fn record_completed(&self, reservation: &RailReservation) {
        let mut records = self.lock();
        let record = records
            .get_mut(reservation.correlation())
            .expect("a completion reservation names an admitted record");
        assert_eq!(&record.effect, reservation.effect());
        assert_eq!(record.status, LedgerStatus::Acknowledged);
        record.status = LedgerStatus::Completed;
    }

    pub fn status_of(&self, correlation: &RailCorrelation) -> LedgerStatus {
        self.lock()
            .get(correlation)
            .map(|record| record.status)
            .unwrap_or(LedgerStatus::NoRecord)
    }

    pub fn notice_of(&self, correlation: &RailCorrelation) -> Option<EstateDeathNotice> {
        self.lock()
            .get(correlation)
            .and_then(|record| match record.effect {
                RailDomainEffect::EstateDeathNotice(notice) => Some(notice),
                RailDomainEffect::ApprovedPaymentSettlement(_) => None,
            })
    }

    pub fn admission_count(&self) -> u64 {
        self.admissions.load(Ordering::SeqCst)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<RailCorrelation, RailRecord>> {
        self.records
            .lock()
            .expect("rail ledger mutex is never poisoned")
    }
}
